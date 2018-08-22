//! An ensemble of multiple predictive models.

use std::collections::HashMap;

use super::{Resource, ResourceCommon};
use super::id::*;
use super::status::*;

/// An ensemble of multiple predictive models.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize)]
#[api_name = "ensemble"]
pub struct Ensemble {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Ensemble>,

    /// The current status of this ensemble.
    pub status: GenericStatus,

    /// Extra information about this ensemble. Does not appear to be
    /// documented in the official API.
    ///
    /// TODO: This may need to be wrapped in `Option` to handle the early
    /// stages of resource creation, when not all fields are present.
    pub ensemble: EnsembleInfo,

    /// Maps BigML field IDs to average importance per field.
    ///
    /// TODO: This may need to be wrapped in `Option` to handle the early
    /// stages of resource creation, when not all fields are present.
    pub importance: HashMap<String, f64>,

    // The dataset used to create this ensemble.
    //pub dataset: Id<Dataset>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// Information about this ensemble.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnsembleInfo {
    /// Information about this ensemble's fields. Keyed by BigML field ID.
    pub fields: HashMap<String, EnsembleField>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// List of field codes mapped to input fields
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnsembleField {
    /// The original name of this field (not the BigML field ID).
    pub name: String,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}
