//! An ensemble of multiple predictive models.

use std::collections::HashMap;

use super::Resource;
use super::id::*;
use super::status::*;

/// List of field codes mapped to input fields
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EnsembleField {
  pub name: String,
}

resource! {
    api_name "ensemble";

    /// An ensemble of multiple predictive models.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize, Clone)]
    pub struct Ensemble {
        /// The current status of this ensemble.
        pub status: GenericStatus,

        /// Maps BigML field codes to named input fields
        /// This is just a single-entry dictionary of "fields" key mapping to
        /// dictionary of fields
        pub ensemble: HashMap<String, HashMap<String, EnsembleField>>,

        /// Maps average importance per field (BigML field id => importance value).
        pub importance: HashMap<String, f64>,

        // The dataset used to create this ensemble.
        //pub dataset: Id<Dataset>,
    }
}
