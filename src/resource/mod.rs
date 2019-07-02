//! Resource types manipulated by the BigML API.

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt};

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
///
/// ### Implementing `Resource` (internal only)
///
/// Normally you want to implement this using `#[derive(Resource)]`, which will
/// look something like:
///
/// ```
/// # #[macro_use] extern crate bigml_derive;
/// # extern crate bigml;
/// # use serde::{Deserialize, Serialize};
/// # use bigml::resource::{GenericStatus, Id, Resource, ResourceCommon, Status, Updatable};
/// #[derive(Clone, Debug, Deserialize, Resource, Serialize, Updatable)]
/// #[api_name = "exampleresource"]
/// pub struct ExampleResource {
///     /// Common resource information. These fields will be serialized at the
///     /// top-level of this structure by `serde`.
///     #[serde(flatten)]
///     #[updatable(flatten)]
///     pub common: ResourceCommon,
///
///     /// The ID of this resource.
///     pub resource: Id<ExampleResource>,
///
///     /// The status of this resource. (Must be a type which implements
///     /// `Status`, normally `GenericStatus` for most resources, except those
///     /// with extended status data.)
///     pub status: GenericStatus,
///
///     // Resource-specific fields here.
///
///     /// Placeholder to allow extensibility without breaking the API.
///     #[serde(skip)]
///     _placeholder: (),
/// }
/// ```
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
    fn status(&self) -> &dyn Status;
}

/// A value which can be updated using the BigML API. May be a `Resource` or a
/// piece of data contained in `Resource`. This is normally passed to
/// `Client::update`.
///
/// ### Implementing `Updatable` (internal only)
///
/// For primitive types like `String` or `bool`, you should add them to the
/// `primitive_updatable_types!` macro, which will define `type Update = self`.
/// You can also do this manually for simple `enum` types, and other values
/// which can only be updated as a whole.
///
/// For struct types, you should use `#[derive(Updatable)]` and mark updatable
/// fields with `#[updatable]`. For a struct `Foo`, this will generate a
/// corresponding `FooUpdate` type, containing only those fields marked as
/// `#[updatable]` (with appropriate types).
pub trait Updatable {
    /// The type of the data used to update this value.
    type Update: Serialize + fmt::Debug;
}

/// Primitive types are updated using plain values of the same type.
macro_rules! primitive_updatable_types {
    ( $( $ty:ty ),* ) => {
        $(
            impl Updatable for $ty {
                type Update = Self;
            }
        )*
    };
}

primitive_updatable_types!(bool, i64, String, u16);

/// `HashMap<String, T>` can be updated using `HashMap<String, T::Update>`.
impl<T: Updatable> Updatable for HashMap<String, T> {
    type Update = HashMap<String, <T as Updatable>::Update>;
}

/// `Option<T>` can be updated using `Option<T::Update>`.
impl<T: Updatable> Updatable for Option<T> {
    type Update = Option<<T as Updatable>::Update>;
}

/// `Vec<T>` can be updated using `Vec<T::Update>`.
impl<T: Updatable> Updatable for Vec<T> {
    type Update = Vec<<T as Updatable>::Update>;
}

/// Arguments which can be used to create a resource.
pub trait Args: fmt::Debug + Serialize {
    /// The resource type these arguments create.
    type Resource: Resource;
}

/// Fields which are present on all resources. This struct is "flattened" into
/// all types which implement `Resource` using `#[serde(flatten)]`, giving us a
/// sort of inheritence.
#[derive(Clone, Debug, Deserialize, Serialize, Updatable)]
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
    #[updatable]
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
    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
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
