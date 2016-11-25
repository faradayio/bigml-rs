//! An execution of a WhizzML script.

use serde::Serialize;
use serde_json;

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
    #[derive(Debug, Deserialize)]
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
#[derive(Debug, Deserialize)]
pub struct Data {
    // The `outputs` field changes type as the script executes.  Don't
    // bother messing with it.  Use `result.outputs` when present instead.

    /// Result values from the script.
    pub result: Option<Vec<serde_json::Value>>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
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
        let mut ser = serde_json::value::Serializer::new();
        value.serialize(&mut ser)?;
        self.inputs.push((name.into(), ser.unwrap()));
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
