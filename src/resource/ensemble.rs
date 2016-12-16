//! An ensemble of multiple predictive models.

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

        // The dataset used to create this ensemble.
        //pub dataset: Id<Dataset>,
    }
}
