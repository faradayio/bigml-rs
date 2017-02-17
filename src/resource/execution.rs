//! An execution of a WhizzML script.

use serde::{Deserialize, Deserializer, Serialize};
use serde::de;
use serde_json;
use std::error;
use std::fmt;
use std::result;

use errors::*;
use super::id::*;
use super::status::*;
use super::Resource;
use super::Script;

resource! {
    api_name "execution";

    /// An execution of a WhizzML script.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize, Clone)]
    pub struct Execution {
        /// The current status of this execution.
        pub status: GenericStatus,

        /// Further information about this execution.
        pub execution: Data,
    }
}

/// Data about a script execution.
///
/// TODO: Lots of missing fields.
#[derive(Debug, Deserialize, Clone)]
pub struct Data {
    /// Outputs from this script.
    #[serde(default)]
    pub outputs: Vec<Output>,

    /// Result values from the script.  This is literally whatever value is
    /// returned at the end of the WhizzML script.
    pub result: Option<serde_json::Value>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl Data {
    /// Get a named output of this execution.
    pub fn get<D: Deserialize>(&self, name: &str) -> Result<D> {
        for output in &self.outputs {
            if output.name == name {
                return output.get();
            }
        }
        Err(ErrorKind::CouldNotGetOutput(name.to_owned()).into())
    }
}

/// Arguments for creating a script execution.
///
/// TODO: Lots of missing fields.
#[derive(Debug, Default, Serialize)]
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

impl super::Args for Args {
    type Resource = Execution;
}

/// A named output value from an execution.
#[derive(Debug, Clone)]
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
    pub fn get<D: Deserialize>(&self) -> Result<D> {
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

impl Deserialize for Output {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer,
    {
        struct OutputVisitor;

        // Do a whole bunch of annoying work to deal with all the different
        // formats this might have.
        impl de::Visitor for OutputVisitor {
            type Value = Output;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "either a string or an array of three strings")
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
                where V: de::SeqVisitor
            {
                use serde::de::Error;

                let name = visitor.visit()?
                    .ok_or_else(|| V::Error::custom("no name field in output"))?;
                let value = visitor.visit()?
                    .ok_or_else(|| V::Error::custom("no value field in output"))?;
                let type_ = visitor.visit()?
                    .ok_or_else(|| V::Error::custom("no type field in output"))?;

                Ok(Output {
                    name: name,
                    value: Some(value),
                    type_: if type_ == "" { None } else { Some(type_) },
                    _hidden: (),
                })
            }
        }

        deserializer.deserialize_i32(OutputVisitor)
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
