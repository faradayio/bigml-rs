//! BigML dataset support.

use super::id::*;
use super::status::*;
use super::Resource;

resource! {
    api_name "dataset";

    /// A BigML dataset. Basically a table of data with named columns.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Dataset {
        /// The current status of this execution.
        pub status: GenericStatus,
    }
}
