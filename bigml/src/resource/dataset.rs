//! BigML dataset support.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::id::*;
use super::source::Field;
use super::status::*;
use super::{Resource, ResourceCommon, Source};

/// A BigML dataset. Basically a table of data with named columns.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[api_name = "dataset"]
pub struct Dataset {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Dataset>,

    /// The current status of this execution.
    pub status: GenericStatus,

    /// The number of columns in the dataset.
    pub columns: usize,

    /// Field IDs excluded when building this dataset.
    pub excluded_fields: Vec<String>,

    /// The number of fields of each type. This includes a few odd things
    /// like "preferred", so we represent it as a string.
    pub field_types: HashMap<String, u64>,

    /// Metadata describing each field. Will be empty while object is being
    /// created.
    #[serde(default)]
    pub fields: HashMap<String, Field>,

    /// Field IDs included when building this dataset.
    pub input_fields: Vec<String>,

    /// The number of rows in this dataset.
    pub rows: usize,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// Arguments used to create a dataset.
#[derive(Debug, Serialize)]
pub struct Args {
    /// The ID of the BigML `Source` from which to import data.
    pub source: Id<Source>,

    /// The name of this dataset.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// User-defined tags.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

impl Args {
    /// Create a new `Args`.
    pub fn from_source(source: Id<Source>) -> Args {
        Args {
            source,
            name: None,
            tags: vec![],
            _placeholder: (),
        }
    }
}

impl super::Args for Args {
    type Resource = Dataset;
}
