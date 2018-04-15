use std::collections::HashMap;

use resource::status::*;

/// Execution-specific status information.
#[derive(Clone, Debug, Deserialize, Serialize)]
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

    /// The number of milliseconds elapsed during different phases of execution.
    #[serde(default)]
    pub elapsed_times: HashMap<String, u64>,

    /// (Undocumented) Where are we in the script's execution? This is
    /// particularly useful when an error occurs.
    pub source_location: Option<SourceLocation>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
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

/// A location in an execution's source code.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SourceLocation {
    /// Start and end column.
    pub columns: (u64, u64),

    /// Start and end line.
    pub lines: (u64, u64),

    /// File in which the error occurred, probably as a position in the
    /// `sources` array?
    pub origin: usize,

    /// For extensibility.
    #[serde(default, skip_serializing)]
    _hidden: (),
}
