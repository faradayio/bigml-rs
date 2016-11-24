//! An execution of a WhizzML script.

use chrono::{DateTime, UTC};
use serde_json;

use super::Resource;
use super::id::*;
use super::status::*;

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
    /// Names, values and types of resources output by the script.
    outputs: Vec<(String, serde_json::Value, String)>,

    /// Result values from the script.
    result: Vec<serde_json::Value>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}
