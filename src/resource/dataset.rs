//! BigML dataset support.

use std::collections::HashMap;

use super::id::*;
use super::status::*;
use super::Resource;
use super::source::Field;

resource! {
    api_name "dataset";

    /// A BigML dataset. Basically a table of data with named columns.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Dataset {
        /// The current status of this execution.
        pub status: GenericStatus,

        /// The number of columns in the dataset.
        pub columns: usize,

        /// Field IDs excluded when building this dataset.
        pub excluded_fields: Vec<String>,

        /// The number of fields of each type. This includes a few odd things
        /// like "preferred", so we represent it as a string.
        pub field_types: HashMap<String, u64>,

        /// Metadata describing each field.
        pub fields: HashMap<String, Field>,

        /// Field IDs included when building this dataset.
        pub input_fields: Vec<String>,

        /// The number of rows in this dataset.
        pub rows: usize,
    }
}
