//! Types represesting the status of a BigML resource.

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

/// Status of a resource.  BigML actually defines many different "status"
/// types, one for each resource, but quite a few of them have are highly
/// similar.  This interface tries to generalize over the most common
/// versions.
pub trait ResourceStatus {
    /// Status code.
    fn code(&self) -> ResourceStatusCode;

    /// Human-readable status message.
    fn message(&self) -> &str;

    /// Number of milliseconds which were needed to create this resource.
    fn elapsed(&self) -> Option<u64>;

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    fn progress(&self) -> Option<f32>;
}

/// Status of a generic resource.
#[derive(Debug, Deserialize)]
pub struct GenericResourceStatus {
    /// Status code.
    pub code: ResourceStatusCode,

    /// Human-readable status message.
    pub message: String,

    /// Number of milliseconds which were needed to create this resource.
    pub elapsed: Option<u64>,

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    pub progress: Option<f32>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

impl ResourceStatus for GenericResourceStatus {
    fn code(&self) -> ResourceStatusCode {
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
