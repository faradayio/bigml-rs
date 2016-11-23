// Included directly into client.rs after pre-processing by serde.

use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::result;
use std::str::FromStr;

use errors::*;

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

/// This trait allows access to common properties shared by all resource
/// types.
pub trait ResourceProperties: fmt::Debug + Deserialize {
    /// The status code for this resource.
    fn status_code(&self) -> ResourceStatusCode;
    /// A human-readable message describing the status of this resource.
    fn status_message(&self) -> &str;
}

/// A trait representing a BigML data type.  Caution!  This is a very
/// abstract trait and implementations are not expected to carry any actual
/// data.  Rather, this mostly exists to be used as a "tag" and to create
/// associations between related types.
pub trait Resource {
    /// The properties of resources of this type.
    type Properties: ResourceProperties;

    /// The prefix used for all IDs of this type.
    fn id_prefix() -> &'static str;
}

/// A strongly-typed "resource ID" used to identify many different kinds of
/// BigML resources.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ResourceId<R: Resource> {
    /// The ID of the resource.
    id: String,
    /// A special 0-byte field which exists just to mention the type `R`
    /// inside the struct, and thus avoid compiler errors about unused type
    /// parameters.
    _phantom: PhantomData<R>,
}

impl<R: Resource> ResourceId<R> {
    /// Get this resource as a string.
    pub fn as_str(&self) -> &str {
        &self.id
    }
}

impl<R: Resource> FromStr for ResourceId<R> {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        if !id.starts_with(R::id_prefix()) {
            Ok(ResourceId {
                id: id.to_owned(),
                _phantom: PhantomData,
            })
        } else {
            Err(ErrorKind::WrongResourceType(R::id_prefix(), id.to_owned()).into())
        }
    }
}

impl<R: Resource> fmt::Debug for ResourceId<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> fmt::Display for ResourceId<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> Deserialize for ResourceId<R> {
    fn deserialize<D>(deserializer: &mut D) -> result::Result<Self, D::Error>
        where D: Deserializer
    {
        let id: String = String::deserialize(deserializer)?;
        if !id.starts_with(R::id_prefix()) {
            Ok(ResourceId {
                id: id,
                _phantom: PhantomData,
            })
        } else {
            let err: Error =
                ErrorKind::WrongResourceType(R::id_prefix(), id).into();
            Err(<D::Error as serde::Error>::invalid_value(&format!("{}", err)))
        }
    }
}

impl<R: Resource> Serialize for ResourceId<R> {
    fn serialize<S>(&self, serializer: &mut S) -> result::Result<(), S::Error>
        where S: Serializer
    {
        self.id.serialize(serializer)
    }
}

/// A data source used by BigML.
pub struct Source;

impl Resource for Source {
    type Properties = SourceProperties;

    fn id_prefix() -> &'static str {
        "source/"
    }
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
    pub resource: ResourceId<Source>,

    /// The number of bytes of the source.
    pub size: u64,

    /// The status of this source.
    pub status: SourceStatus,

    /// A hidden field to allow future extensibility.
    #[serde(default)]
    _hidden: (),
}

impl ResourceProperties for SourceProperties {
    fn status_code(&self) -> ResourceStatusCode {
        self.status.code
    }

    fn status_message(&self) -> &str {
        &self.status.message
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
