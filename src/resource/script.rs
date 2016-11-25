//! A WhizzML script on BigML.

use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "script";

    /// A WhizzML script on BigML.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize)]
    pub struct Script {
        /// The status of this resource.
        pub status: GenericStatus,

        /// The source code of this script.
        pub source_code: String,
    }
}
