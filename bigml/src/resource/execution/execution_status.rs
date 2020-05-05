use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::resource::status::*;

/// Execution-specific status information.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[non_exhaustive]
pub struct ExecutionStatus {
    /// Status code.
    pub code: StatusCode,

    /// Human-readable status message.
    pub message: String,

    /// Number of milliseconds which were needed to create this resource.
    pub elapsed: Option<u64>,

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    pub progress: Option<f32>,

    /// The call stack, if one is present.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        with = "call_stack_repr"
    )]
    pub call_stack: Option<Vec<SourceLocation>>,

    /// The cause of the error.
    pub cause: Option<Cause>,

    /// The number of milliseconds elapsed during different phases of execution.
    #[serde(default)]
    pub elapsed_times: HashMap<String, u64>,

    /// Extra information about errors, typically mapping field names to
    /// field-specific error messages.
    #[serde(default)]
    pub extra: HashMap<String, String>,

    /// The instruction at which an error occurred.
    pub instruction: Option<Instruction>,

    /// (Undocumented) Where are we in the script's execution? This is
    /// particularly useful when an error occurs.
    pub source_location: Option<SourceLocation>,
}

impl ExecutionStatus {
    /// The `message` for this status, plus the `cause` and any other useful
    /// information that might be present.
    pub fn full_message(&self) -> String {
        if let Some(ref cause) = self.cause {
            format!("{} ({})", self.message, cause)
        } else {
            self.message.clone()
        }
    }
}

impl Status for ExecutionStatus {
    fn code(&self) -> StatusCode {
        self.code
    }

    fn message(&self) -> &str {
        &self.message
    }

    fn elapsed(&self) -> Option<u64> {
        self.elapsed
    }

    fn progress(&self) -> Option<f32> {
        self.progress
    }
}

/// Functions for (de)serializing WhizzML call stacks.
pub(crate) mod call_stack_repr {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::*;

    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Option<Vec<SourceLocation>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[allow(clippy::type_complexity)]
        let raw: Option<Vec<(usize, (u64, u64), (u64, u64))>> =
            Deserialize::deserialize(deserializer)?;
        Ok(raw.map(|vec| {
            vec.into_iter()
                .map(|(origin, lines, columns)| SourceLocation {
                    origin,
                    columns,
                    lines,
                })
                .collect()
        }))
    }

    pub(crate) fn serialize<S>(
        stack: &Option<Vec<SourceLocation>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw: Option<Vec<_>> = stack.as_ref().map(|vec| {
            vec.iter()
                .map(|sloc| (sloc.origin, sloc.lines, sloc.columns))
                .collect()
        });
        raw.serialize(serializer)
    }
}

/// A location in an execution's source code.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SourceLocation {
    /// Start and end column.
    pub columns: (u64, u64),

    /// Start and end line.
    pub lines: (u64, u64),

    /// File in which the error occurred, probably as a position in the
    /// `sources` array?
    pub origin: usize,
}

/// The cause of an error.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Cause {
    /// The error code of the underlying error.
    pub code: i64,

    /// Extra information about the underlying error (may be a list or
    /// hash, possibly other things).
    #[serde(default)]
    pub extra: serde_json::Value,

    /// The HTTP status related to the underlying error.
    pub http_status: Option<u16>,
}

impl fmt::Display for Cause {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "code: {}", self.code)?;
        if let Some(http_status) = self.http_status {
            write!(f, ", HTTP status: {}", http_status)?;
        }
        write!(f, ", extra: {}", self.extra,)
    }
}

/// Information on the instruction where an error occurred.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Instruction {
    /// The name of the instruction.
    pub instruction: String,

    /// The source location where the error occurred.
    pub source: SourceLocation,
}

#[test]
fn deserialize_error_status() {
    let json = r#"{"call_stack": [[1, [109, 109], [14, 65]], [1, [109, 109], [15, 17]]], "code": -1, "elapsed": 62321, "elapsed_times": {"in-progress": 62265, "queued": 140, "started": 56}, "error": -8200, "instruction": {"instruction": "push-procedure", "source": {"columns": [14, 65], "lines": [109, 109], "origin": 1}}, "message": "Problem while executing script:  'get' expects 2 or 3 arguments, 4 given", "progress": 0.195, "source_location": {"columns": [0, 34], "lines": [97, 97], "origin": 1}}"#;
    let _status: ExecutionStatus = serde_json::from_str(json).unwrap();

    let json = r#"{"call_stack": [[1, [32, 47], [15, 1]]], "cause": {"code": -1206, "extra": {"all_fields": "Must be true or false", "fields": "Must be an object"}, "http_status": 400}, "code": -1, "elapsed": 8896, "elapsed_times": {"in-progress": 8834, "queued": 22, "started": 62}, "error": -8200, "instruction": {"instruction": "apply", "source": {"columns": [15, 1], "lines": [32, 47], "origin": 1}}, "message": "Problem while executing script: Error handling resource (Validation error)", "progress": 0.195, "source_location": {"columns": [15, 1], "lines": [32, 47], "origin": 1}}"#;
    let status: ExecutionStatus = serde_json::from_str(json).unwrap();
    assert_eq!(status.cause.unwrap().code, -1206);
}
