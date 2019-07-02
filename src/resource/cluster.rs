//! https://bigml.com/api/clusters

use serde::{Deserialize, Serialize};
use serde_json;

use super::{Resource, ResourceCommon};
use super::id::*;
use super::status::*;

/// An cluster of multiple predictive models.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[api_name = "cluster"]
pub struct Cluster {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Cluster>,

    /// The current status of this cluster.
    pub status: GenericStatus,

    /// Details about the clusters that BigML found.
    ///
    /// TODO: Convert to a strongly-typed struct.
    pub clusters: Option<serde_json::Value>,

    // The dataset used to create this cluster.
    //pub dataset: Id<Dataset>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}
