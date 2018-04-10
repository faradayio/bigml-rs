//! A batch prediction of missing values from a data set.


use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "batchprediction";

    /// A batch prediction generated by BigML.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct BatchPrediction {
        /// The status of this source.
        pub status: GenericStatus,

        /// Does this prediction include all the fields in the input?
        pub all_fields: bool,

        // Our output dataset.
        //pub output_dataset_resource: Option<Id<Dataset>>,

        /// Is our output dataset currently available?
        pub output_dataset_status: bool,
    }
}
