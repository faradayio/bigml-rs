// Included directly into client.rs after pre-processing by serde.

use serde::{self, Deserialize, Deserializer};
use std::result;

/// A BigML status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResourceStatusCode {
    /// BigML is waiting on another resource before processing this one.
    Waiting,
    /// The processing job has been added to the queue.
    Queued,
    /// Actual processing has started.
    Started,
    /// Part of the job has been performed.
    InProgress,
    /// Summary statistics for a dataset are available.
    Summarized,
    /// The resource is ready.
    Finished,
    /// Something went wrong processing the task.
    Faulty,
    /// Something has gone wrong in BigML, perhaps an outage.
    Unknown,
}

impl ResourceStatusCode {
    /// Is BigML still working on ingesting and processing this resource?
    pub fn is_working(self) -> bool {
        use ResourceStatusCode::*;
        match self {
            Waiting | Queued | Started | InProgress | Summarized => true,
            _ => false,
        }
    }

    /// Has BigML successfully finished processing this resource?
    pub fn is_ready(self) -> bool {
        self == ResourceStatusCode::Finished
    }

    /// Did something go wrong while processing this resource?
    pub fn is_err(self) -> bool {
        self == ResourceStatusCode::Faulty ||
            self == ResourceStatusCode::Unknown
    }
}

impl Deserialize for ResourceStatusCode {
    fn deserialize<D>(deserializer: &mut D) -> result::Result<Self, D::Error>
        where D: Deserializer
    {
        match i64::deserialize(deserializer)? {
            0 => Ok(ResourceStatusCode::Waiting),
            1 => Ok(ResourceStatusCode::Queued),
            2 => Ok(ResourceStatusCode::Started),
            3 => Ok(ResourceStatusCode::InProgress),
            4 => Ok(ResourceStatusCode::Summarized),
            5 => Ok(ResourceStatusCode::Finished),
            -1 => Ok(ResourceStatusCode::Faulty),
            -2 => Ok(ResourceStatusCode::Unknown),
            code => {
                let msg = format!("Unknown BigML resource status code {}", code);
                Err(<D::Error as serde::Error>::invalid_value(&msg))
            }
        }
    }
}

/// The status of a BitML source.
#[derive(Debug, Deserialize)]
pub struct SourceStatus {
    /// A code describing the status.
    pub code: ResourceStatusCode,
    /// A human-readable message explaining the status.
    pub message: String,
    /// The number of milliseconds spent processing the source.
    pub elapsed: Option<u64>,
}

/// Properties of BigML source.
///
/// TODO: Still lots of missing fields.
#[derive(Debug, Deserialize)]
pub struct SourceProperties {
    /// Used to classify data by industry or category.  0 is
    /// "Miscellaneous".
    pub category: i64,

    /// An HTTP status code, typically either 201 or 200.
    ///
    /// TODO: Deserialize as a `reqwest::StatusCode`?
    pub code: u16,

    /// The number of credits it cost to create this source.
    pub credits: f64,

    /// Text describing this source.  May contain limited Markdown.
    pub description: String,

    /// The name of the file uploaded.
    pub file_name: String,

    /// An MD5 hash of the uploaded file.
    pub md5: String,

    /// The name of this data source.
    pub name: String,

    /// The identifier for this source.
    pub resource: String,

    /// The number of bytes of the source.
    pub size: u64,

    /// The status of this source.
    pub status: SourceStatus,

    /// A hidden field to allow future extensibility.
    #[serde(default)]
    _hidden: (),
}
