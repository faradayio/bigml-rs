//! Types represesting the status of a BigML resource.

use serde::de::Unexpected;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::result;

/// A BigML status code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatusCode {
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

impl StatusCode {
    /// Is BigML still working on ingesting and processing this resource?
    pub fn is_working(self) -> bool {
        use self::StatusCode::*;
        match self {
            Waiting | Queued | Started | InProgress | Summarized => true,
            _ => false,
        }
    }

    /// Has BigML successfully finished processing this resource?
    pub fn is_ready(self) -> bool {
        self == StatusCode::Finished
    }

    /// Did something go wrong while processing this resource?
    pub fn is_err(self) -> bool {
        self == StatusCode::Faulty || self == StatusCode::Unknown
    }
}

impl<'de> Deserialize<'de> for StatusCode {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match i64::deserialize(deserializer)? {
            0 => Ok(StatusCode::Waiting),
            1 => Ok(StatusCode::Queued),
            2 => Ok(StatusCode::Started),
            3 => Ok(StatusCode::InProgress),
            4 => Ok(StatusCode::Summarized),
            5 => Ok(StatusCode::Finished),
            -1 => Ok(StatusCode::Faulty),
            -2 => Ok(StatusCode::Unknown),
            code => {
                let unexpected = Unexpected::Signed(code);
                let expected = "a number between -2 and 5";
                Err(<D::Error as serde::de::Error>::invalid_value(
                    unexpected, &expected,
                ))
            }
        }
    }
}

impl Serialize for StatusCode {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let code = match *self {
            StatusCode::Waiting => 0,
            StatusCode::Queued => 1,
            StatusCode::Started => 2,
            StatusCode::InProgress => 3,
            StatusCode::Summarized => 4,
            StatusCode::Finished => 5,
            StatusCode::Faulty => -1,
            StatusCode::Unknown => -2,
        };
        code.serialize(serializer)
    }
}

/// Status of a resource.  BigML actually defines many different "status"
/// types, one for each resource, but quite a few of them have are highly
/// similar.  This interface tries to generalize over the most common
/// versions.
pub trait Status {
    /// Status code.
    fn code(&self) -> StatusCode;

    /// Human-readable status message.
    fn message(&self) -> &str;

    /// Number of milliseconds which were needed to create this resource.
    fn elapsed(&self) -> Option<u64>;

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    fn progress(&self) -> Option<f32>;
}

/// Status of a generic resource.
#[derive(Debug, Deserialize, Clone, Serialize)]
pub struct GenericStatus {
    /// Status code.
    pub code: StatusCode,

    /// Human-readable status message.
    pub message: String,

    /// Number of milliseconds which were needed to create this resource.
    pub elapsed: Option<u64>,

    /// Number between 0.0 and 1.0 representing the progress of creating
    /// this resource.
    pub progress: Option<f32>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

impl Status for GenericStatus {
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
