//! A data source used by BigML.

use chrono::{DateTime, UTC};

use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "source";

    /// A data source used by BigML.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize)]
    pub struct Source {
        /// The status of this source.
        pub status: GenericStatus,

        /// The name of the file uploaded.
        pub file_name: String,

        /// An MD5 hash of the uploaded file.
        pub md5: String,

        /// The number of bytes of the source.
        pub size: u64,
    }
}
