//! Resource types manipulated by the BigML API.

use serde::Deserialize;
use std::fmt;

// We re-export everything from our support submodules.
pub use self::id::*;
pub use self::status::*;

// We only re-export the main names from our resource submodules.  For any
// other types, use a fully-qualified name.
pub use self::ensemble::Ensemble;
pub use self::evaluation::Evaluation;
pub use self::execution::Execution;
pub use self::source::Source;

macro_rules! resource {
    (
        api_name $string_name:expr;

        // The pattern `$(<$($Ty : $Tr),*>)*` is overly generous.  We want
        // to match an optional set of type parameters of the form `<Name:
        // Trait, ...>`, but Rust macros have no easy "match 0 or 1"
        // mechanism, so we match 0 or more `<...>` patterns instead.

        $(#[ $meta:meta ])*
        pub struct $name:ident $(<$($Ty:ident : $Tr:ident),*>)* {
            $(
                $(#[ $field_type_meta:meta ])*
                pub $field_name:ident: $field_ty:ty,
            )*
        }

    ) => {
        $(#[ $meta ])*
        pub struct $name $(<$($Ty : $Tr),*>)* {
            // Start by declaring the fields which appear on every resource
            // type.  We should theoretically implement this using
            // inheritance, but Rust doesn't have implementation
            // inheritance.  We could also implement this using various
            // other Rust patterns like delegation, but that would mean
            // that serde could no longer assume a simple 1-to-1 mapping
            // between Rust and JSON types. So we just use a macro to do
            // some code gen, and we define a `Resource` trait that we can
            // use to access any duplicated bits using a single API.

            /// Used to classify by industry or category.  0 is "Miscellaneous".
            pub category: i64,

            /// An HTTP status code, typically either 201 or 200.
            ///
            /// TODO: Deserialize as a `reqwest::StatusCode`?
            pub code: u16,

            /// The time this resource was created.
            pub created: DateTime<UTC>,

            /// Was this created in development mode?
            pub dev: bool,

            /// Text describing this resource.  May contain limited Markdown.
            pub description: String,

            /// The name of this resource
            pub name: String,

            // What project is this associated with?
            //
            // TODO: Define `Project` type and then enable this.
            //pub project: ResourceId<Project>,

            /// Has this been shared using a private link?
            pub shared: bool,

            /// Was this created using a subscription plan?
            pub subscription: bool,

            /// User-defined tags.
            pub tags: Vec<String>,

            /// The last time this was updated.
            pub updated: DateTime<UTC>,

            /// The ID of this execution.
            pub resource: ResourceId<$name $(<$($Ty),*>)*>,

            /// Having one hidden field makes it possible to extend this struct
            /// without breaking semver API guarantees.
             #[serde(default, skip_serializing)]
            _hidden: (),

            $(
                $(#[ $field_type_meta ])*
                pub $field_name: $field_ty
            ),*
        }

        impl $(<$($Ty : $Tr),*>)* Resource for $name $(<$($Ty),*>)* {
            fn id_prefix() -> &'static str {
                concat!($string_name, "/")
            }

            fn status(&self) -> &Status {
                &self.status
            }
        }
    };
}

/// A shared interface to all BigML resource types.
pub trait Resource: fmt::Debug + Deserialize {
    /// The prefix used for all IDs of this type.
    fn id_prefix() -> &'static str;

    /// The status code for this resource.
    fn status(&self) -> &Status;
}

// Support modules defining general types.
mod id;
mod status;

// Individual resource types.
pub mod ensemble;
pub mod evaluation;
pub mod execution;
pub mod source;
