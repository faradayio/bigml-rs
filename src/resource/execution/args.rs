use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde::de::DeserializeOwned;
use serde::de;
use serde::ser::SerializeSeq;
use serde_json;
use std::error;
use std::fmt;
use std::result;

use errors::*;
use resource;
use resource::id::*;
use resource::Script;
use super::Execution;

/// Arguments for creating a script execution.
///
/// TODO: Lots of missing fields.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Args {
    /// The ID of the script to run.
    pub script: Option<Id<Script>>,

    /// Inputs to our script.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<(String, serde_json::Value)>,

    /// Outputs to place into the `result` field of our `Data`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outputs: Vec<String>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl Args {
    /// Set the script to execute.
    pub fn set_script(&mut self, id: Id<Script>) {
        self.script = Some(id);
    }

    /// Add a named input to our script.
    pub fn add_input<S, V>(&mut self, name: S, value: V) -> Result<()>
        where S: Into<String>, V: Serialize
    {
        let val = serde_json::value::to_value(value)?;
        self.inputs.push((name.into(), val));
        Ok(())
    }

    /// Add a named output parameter that we want place into `result`.
    pub fn add_output<S>(&mut self, name: S)
        where S: Into<String>
    {
        self.outputs.push(name.into());
    }
}

impl resource::Args for Args {
    type Resource = Execution;
}

/// A named output value from an execution.
#[derive(Clone, Debug)]
pub struct Output {
    /// The name of this output.
    pub name: String,

    /// The value of this output, or `None` if it has not yet been computed.
    pub value: Option<serde_json::Value>,

    /// The type of this output, or `None` if we don't know the type.
    pub type_: Option<String>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    _hidden: (),
}

impl Output {
    /// Get this output as the specified type, performing any necessary
    /// conversions.  Returns an error if this output hasn't been computed
    /// yet.
    pub fn get<D: DeserializeOwned>(&self) -> Result<D> {
        let mkerr = || ErrorKind::CouldNotGetOutput(self.name.clone());
        if let Some(ref value) = self.value {
            // We need to be explicit about the error type we want
            // `from_value` to return here.
            let result: result::Result<D, serde_json::error::Error> =
                serde_json::value::from_value(value.to_owned());
            result.chain_err(&mkerr)
        } else {
            let err: Error = ErrorKind::OutputNotAvailable.into();
            Err(err).chain_err(&mkerr)
        }
    }
}

impl<'de> Deserialize<'de> for Output {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        struct OutputVisitor;

        // Do a whole bunch of annoying work to deal with all the different
        // formats this might have.
        impl<'de> de::Visitor<'de> for OutputVisitor {
            type Value = Output;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "either a string or a [name, value, type] sequence")
            }

            fn visit_str<E>(self, v: &str)
                            -> result::Result<Self::Value, E>
                where E: error::Error
            {
                Ok(Output {
                    name: v.to_owned(),
                    value: None,
                    type_: None,
                    _hidden: (),
                })
            }

            fn visit_seq<V>(self, mut visitor: V)
                            -> result::Result<Self::Value, V::Error>
                where V: de::SeqAccess<'de>
            {
                use serde::de::Error;

                let name = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no name field in output"))?;
                let value = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no value field in output"))?;
                let type_ = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no type field in output"))?;

                Ok(Output {
                    name: name,
                    value: Some(value),
                    type_: if type_ == "" { None } else { Some(type_) },
                    _hidden: (),
                })
            }
        }

        deserializer.deserialize_any(OutputVisitor)
    }
}

#[test]
fn deserialize_output_with_only_a_name() {
    let json = r#""name""#;
    let output: Output = serde_json::from_str(&json).unwrap();
    assert_eq!(output.name, "name");
    assert!(output.value.is_none());
    assert!(output.get::<bool>().is_err());
    assert!(output.type_.is_none());
}

#[test]
fn deserialize_output_with_name_and_value_but_no_type() {
    let json = r#"["name", null, ""]"#;
    let output: Output = serde_json::from_str(&json).unwrap();
    assert_eq!(output.name, "name");
    assert_eq!(output.get::<Option<bool>>().unwrap(), None);
    assert!(output.type_.is_none());
}

#[test]
fn deserialize_output_with_everything() {
    use resource::evaluation::{ClassificationResult, Evaluation};

    let json =
        r#"["evaluation", "evaluation/50650d563c19202679000000", "evaluation"]"#;
    let output: Output = serde_json::from_str(&json).unwrap();

    assert_eq!(output.name, "evaluation");
    let value: Id<Evaluation<ClassificationResult>> = output.get().unwrap();
    assert_eq!(value.as_str(), "evaluation/50650d563c19202679000000");
    assert_eq!(output.type_.unwrap(), "evaluation");
}

#[test]
fn deserialize_multiple_outputs() {
    // This _appears_ to be breaking in one caller of `bigml`, so let's put
    // in some tests to ensure that it actually works.
    let json = r#"
    [
      ["evaluation", null, ""],
      ["final-ensemble", null, ""],
      ["fields", ["label", "id"], "list"]
    ]
    "#;
    let outputs: Vec<Output> = serde_json::from_str(&json).unwrap();
    assert_eq!(outputs.len(), 3);
}

impl Serialize for Output {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        // Always serialize in "canonical" form.
        let mut seq = serializer.serialize_seq(Some(3))?;
        seq.serialize_element(&self.name)?;
        seq.serialize_element(&self.value)?;
        if let Some(ref type_) = self.type_ {
            seq.serialize_element(type_)?;
        } else {
            // Gross: This is represented as an empty string instead of NULL.
            seq.serialize_element("")?;
        }
        seq.end()
    }
}

/// The logging level of a log message.
#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
enum LogLevel {
    /// Generated by `log-info`.
    #[serde(rename = "info")]
    Info,

    /// Generated by `log-warn`.
    #[serde(rename = "warn")]
    Warn,

    /// Generated by `log-error`.
    #[serde(rename = "error")]
    Error,
}

/// A log entry output by the script.
#[derive(Clone, Debug)]
pub struct LogEntry {
    log_level: LogLevel,
    timestamp: DateTime<Utc>,
    source_index: u64,
    line_number: u64,
    message: String,
}

impl<'de> Deserialize<'de> for LogEntry {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer<'de>,
    {
        struct LogEntryVisitor;

        // Do a whole bunch of annoying work needed to deserialize mixed-type
        // arrays.
        impl<'de> de::Visitor<'de> for LogEntryVisitor {
            type Value = LogEntry;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "a list containing log entry information")
            }

            fn visit_seq<V>(self, mut visitor: V)
                            -> result::Result<Self::Value, V::Error>
                where V: de::SeqAccess<'de>
            {
                use serde::de::Error;

                let log_level = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no log_level field in log entry"))?;
                let timestamp = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no timestamp field in log entry"))?;
                let source_index = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no source_index field in log entry"))?;
                let line_number = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no line_number field in log entry"))?;
                let message = visitor.next_element()?
                    .ok_or_else(|| V::Error::custom("no message field in log entry"))?;

                Ok(LogEntry {
                    log_level,
                    timestamp,
                    source_index,
                    line_number,
                    message,
                })
            }
        }

        deserializer.deserialize_seq(LogEntryVisitor)
    }
}

impl Serialize for LogEntry {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        let mut seq = serializer.serialize_seq(Some(5))?;
        seq.serialize_element(&self.log_level)?;
        seq.serialize_element(&self.timestamp)?;
        seq.serialize_element(&self.source_index)?;
        seq.serialize_element(&self.line_number)?;
        seq.serialize_element(&self.message)?;
        seq.end()
    }
}

#[test]
fn deserialize_serialize_log_entry() {
    let json =
        r#"["info","2016-04-17T01:13:30.713Z",0,30,"creating model 1"]"#;
    let entry: LogEntry = serde_json::from_str(json).unwrap();
    assert_eq!(entry.log_level, LogLevel::Info);
    assert_eq!(entry.source_index, 0);
    assert_eq!(entry.line_number, 30);
    assert_eq!(entry.message, "creating model 1");

    let ser_json = serde_json::to_string(&entry).unwrap();
    assert_eq!(ser_json, json);
}

/// A resource created by the script.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OutputResource {
    /// The ID of the resource created.
    pub id: String,

    /// The name of the variable in which the resource was stored.
    #[serde(default)]
    pub variable: Option<String>,

    /// The time when this resource was last upgraded.
    pub last_update: i64,

    /// A progress value, probably between 0.0 and 1.0.
    pub progress: f64,

    /// A human-readable description of what's currently happening.
    pub task: Option<String>,

    /// This appears to be a textual representation of a `StatusCode`.
    pub state: String,

    /// For extensibility.
    #[serde(default, skip_serializing)]
    _hidden: (),
}