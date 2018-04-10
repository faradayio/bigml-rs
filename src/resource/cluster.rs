//! https://bigml.com/api/clusters

use serde_json;

use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "cluster";

    /// An cluster of multiple predictive models.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Clone, Debug, Deserialize, Serialize)]
    pub struct Cluster {
        /// The current status of this cluster.
        pub status: GenericStatus,

        /// Details about the clusters that BigML found.
        ///
        /// TODO: Convert to a strongly-typed struct.
        pub clusters: Option<serde_json::Value>,

        // The dataset used to create this cluster.
        //pub dataset: Id<Dataset>,
    }
}
