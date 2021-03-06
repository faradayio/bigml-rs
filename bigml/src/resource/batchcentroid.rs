//! https://bigml.com/api/batchcentroids

use serde::{Deserialize, Serialize};

use super::id::*;
use super::status::*;
use super::{Resource, ResourceCommon};

/// A batch centroid generated by BigML.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[api_name = "batchcentroid"]
#[non_exhaustive]
pub struct BatchCentroid {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<BatchCentroid>,

    /// The status of this source.
    pub status: GenericStatus,

    /// Does this centroid include all the fields in the input?
    pub all_fields: bool,

    // Our output dataset.
    //pub output_dataset_resource: Option<Id<Dataset>>,
    /// Is our output dataset currently available?
    pub output_dataset_status: bool,
}
