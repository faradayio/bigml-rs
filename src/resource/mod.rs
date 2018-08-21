//! Resource types manipulated by the BigML API.

use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;

// We re-export everything from our support submodules.
pub use self::id::*;
pub use self::status::*;

// We only re-export the main names from our resource submodules.  For any
// other types, use a fully-qualified name.
pub use self::batchcentroid::BatchCentroid;
pub use self::batchprediction::BatchPrediction;
pub use self::cluster::Cluster;
pub use self::dataset::Dataset;
pub use self::ensemble::{Ensemble, EnsembleField};
pub use self::evaluation::Evaluation;
pub use self::execution::Execution;
pub use self::library::Library;
pub use self::script::Script;
pub use self::source::Source;

/// A shared interface to all BigML resource types.
pub trait Resource: fmt::Debug + DeserializeOwned + Serialize + 'static {
    /// The prefix used for all IDs of this type.
    fn id_prefix() -> &'static str;

    /// The URL path used to create a new resource of this type.
    fn create_path() -> &'static str;

    /// Fields shared between all resource types. These are "flattened" into the
    /// top-level of the JSON version of this resource.
    fn common(&self) -> &ResourceCommon;

    /// The ID of this resource.
    fn id(&self) -> &Id<Self>;

    /// The status code for this resource.
    ///
    /// TODO: Does this need to go in a separate trait in order to maintain
    /// trait object support?
    fn status(&self) -> &Status;
}

/// A value which can be updated using the BigML API. May be a `Resource` or a
/// small piece of a `Resource`.
pub trait Updatable {
    /// The type of the data used to update this value.
    type Update: Serialize;
}

/// Arguments which can be used to create a resource.
pub trait Args: fmt::Debug + Serialize {
    /// The resource type these arguments create.
    type Resource: Resource;
}

/// Fields which are present on all resources. This struct is "flattened" into
/// all types which implement `Resource` using `#[serde(flatten)]`, giving us a
/// sort of inheritence.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResourceCommon {
    /// Used to classify by industry or category.  0 is "Miscellaneous".
    pub category: i64,

    /// An HTTP status code, typically either 201 or 200.
    ///
    /// TODO: Deserialize as a `reqwest::StatusCode`?
    pub code: u16,

    // The time this resource was created.
    //
    // TODO: The response is missing the `Z`, which makes chrono sad.
    //pub created: DateTime<UTC>,

    /// Was this created in development mode?
    pub dev: Option<bool>,

    /// Text describing this resource.  May contain limited Markdown.
    pub description: String,

    /// The name of this resource.
    pub name: String,

    // What project is this associated with?
    //
    // TODO: Define `Project` type and then enable this.
    //pub project: Id<Project>,

    /// Has this been shared using a private link?
    pub shared: bool,

    /// Was this created using a subscription plan?
    pub subscription: bool,

    /// User-defined tags.
    pub tags: Vec<String>,

    // The last time this was updated.
    //
    // TODO: The response is missing the `Z`, which makes chrono sad.
    //pub updated: DateTime<UTC>,

    /// Having one hidden field makes it possible to extend this struct
    /// without breaking semver API guarantees.
     #[serde(default, skip_serializing)]
    _hidden: (),
}

// Support modules defining general types.
mod id;
mod status;

// Individual resource types.  These need to go after our `response!` macro
// definition, above, because macros are processed as source is being read.
pub mod batchcentroid;
pub mod batchprediction;
pub mod cluster;
pub mod dataset;
pub mod ensemble;
pub mod evaluation;
pub mod execution;
pub mod library;
pub mod script;
pub mod source;
