//! https://bigml.com/api/clusters

use super::Resource;
use super::id::*;
use super::status::*;

resource! {
    api_name "cluster";

    /// An cluster of multiple predictive models.
    ///
    /// TODO: Still lots of missing fields.
    #[derive(Debug, Deserialize, Clone)]
    pub struct Cluster {
        /// The current status of this cluster.
        pub status: GenericStatus,

        pub clusters: # FIXME this is just a json blob, we'll examine later

        // The dataset used to create this cluster.
        //pub dataset: Id<Dataset>,
    }
}
