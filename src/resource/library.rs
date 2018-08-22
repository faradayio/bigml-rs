//! BigML dataset support.

use super::id::*;
use super::status::*;
use super::{Resource, ResourceCommon};

/// A BigML library for use in a WhizzML script.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[api_name = "library"]
pub struct Library {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Library>,

    /// The current status of this execution.
    pub status: GenericStatus,

    /// The source code of this library.
    pub source_code: String,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// Arguments used to create a new BigML script.
#[derive(Debug, Serialize)]
pub struct Args {
    /// The category code which best describes this script.
    #[serde(skip_serializing_if="Option::is_none")]
    pub category: Option<i64>,

    /// A human-readable description of this script.
    #[serde(skip_serializing_if="Option::is_none")]
    pub description: Option<String>,

    /// A list of "library/..." identifiers to import.
    #[serde(skip_serializing_if="Vec::is_empty")]
    pub imports: Vec<Id<Library>>,

    /// A human-readable name for this script.
    #[serde(skip_serializing_if="Option::is_none")]
    pub name: Option<String>,

    /// The source code of this script.
    pub source_code: String,

    /// User-defined tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

impl Args {
    /// Create a new `ScriptNew` value.
    pub fn new<S: Into<String>>(source_code: S) -> Args {
        Args {
            category: Default::default(),
            description: Default::default(),
            imports: Default::default(),
            name: Default::default(),
            source_code: source_code.into(),
            tags: Default::default(),
            _placeholder: (),
        }
    }
}

impl super::Args for Args {
    type Resource = Library;
}
