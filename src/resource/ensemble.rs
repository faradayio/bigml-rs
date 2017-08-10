//! An ensemble of multiple predictive models.

use std::collections::HashMap;

use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "ensemble";

    /// An ensemble of multiple predictive models.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize, Clone)]
    pub struct Ensemble {
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
    }
}

/// Information about this ensemble.
#[derive(Clone, Debug, Deserialize)]
pub struct EnsembleInfo {
    /// Information about this ensemble's fields. Keyed by BigML field ID.
    fields: HashMap<String, EnsembleField>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}

/// List of field codes mapped to input fields
#[derive(Clone, Debug, Deserialize)]
pub struct EnsembleField {
    /// The original name of this field (not the BigML field ID).
    pub name: String,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
    #[serde(default, skip_serializing)]
    _hidden: (),
}
