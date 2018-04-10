//! BigML dataset support.

use super::id::*;
use super::status::*;
use super::Resource;

resource! {
    api_name "library";

    /// A BigML library for use in a WhizzML script.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Library {
        /// The current status of this execution.
        pub status: GenericStatus,
    }
}
