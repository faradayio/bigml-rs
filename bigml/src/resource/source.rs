//! A data source used by BigML.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::id::*;
use super::status::*;
use super::{Resource, ResourceCommon, Updatable};

/// A data source used by BigML.
///
/// TODO: Still lots of missing fields.
#[derive(Clone, Debug, Deserialize, Resource, Serialize, Updatable)]
#[api_name = "source"]
pub struct Source {
    /// Common resource information. These fields will be serialized at the
    /// top-level of this structure by `serde`.
    #[serde(flatten)]
    #[updatable(flatten)]
    pub common: ResourceCommon,

    /// The ID of this resource.
    pub resource: Id<Source>,

    /// The status of this source.
    pub status: GenericStatus,

    /// The name of the file uploaded.
    pub file_name: Option<String>,

    /// An MD5 hash of the uploaded file.
    pub md5: String,

    /// The number of bytes of the source.
    pub size: u64,

    /// Whether BigML should automatically expand dates into year, day of week, etc.
    #[updatable]
    pub disable_datetime: Option<bool>,

    /// The fields in this source, keyed by BigML internal ID.
    #[updatable]
    pub fields: Option<HashMap<String, Field>>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// Arguments used to create a data source.
///
/// TODO: Add more fields so people need to use `update` less.
#[derive(Debug, Serialize)]
pub struct Args {
    /// The URL of the data source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,

    /// The raw data to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,

    /// Set to true if you want to avoid date expansion into year, day of week, etc.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_datetime: Option<bool>,

    /// The name of this source.
    pub name: Option<String>,

    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

impl Args {
    /// Create a new `Args` from a remote data source.
    pub fn remote<S: Into<String>>(remote: S) -> Args {
        Args {
            remote: Some(remote.into()),
            data: None,
            disable_datetime: None,
            name: None,
            _placeholder: (),
        }
    }

    /// Create a new `Args` from a small amount of inline data.
    pub fn data<S: Into<String>>(data: S) -> Args {
        Args {
            remote: None,
            data: Some(data.into()),
            disable_datetime: None,
            name: None,
            _placeholder: (),
        }
    }
}

impl super::Args for Args {
    type Resource = Source;
}

/// Information about a field in a data source.
#[derive(Clone, Debug, Deserialize, Serialize, Updatable)]
pub struct Field {
    /// The name of this field.
    pub name: String,

    /// The type of data stored in this field.
    #[updatable]
    pub optype: Optype,

    // The locale of this field.
    //pub locale: Option<String>,

    // (This is not well-documented in the BigML API.)
    //pub missing_tokens: Option<Vec<String>>,
    /// Placeholder to allow extensibility without breaking the API.
    #[serde(skip)]
    _placeholder: (),
}

/// The type of a data field.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Serialize)]
pub enum Optype {
    /// Treat this as a date value.
    #[serde(rename = "datetime")]
    DateTime,

    /// Treat this as a numeric value.
    #[serde(rename = "numeric")]
    Numeric,
    /// Threat this as a category with multiple possible values, but not
    /// arbitrary strings.
    #[serde(rename = "categorical")]
    Categorical,
    /// Treat this as text.  This uses different machine learning
    /// algorithms than `Categorical`.
    #[serde(rename = "text")]
    Text,
    /// Treat this as a list of muliple items separated by an auto-detected
    /// separator.
    #[serde(rename = "items")]
    Items,
}

impl Updatable for Optype {
    type Update = Self;
}

#[test]
fn update_source_name() {
    use super::ResourceCommonUpdate;
    use serde_json::json;
    let source_update = SourceUpdate {
        common: Some(ResourceCommonUpdate {
            name: Some("example".to_owned()),
            ..ResourceCommonUpdate::default()
        }),
        ..SourceUpdate::default()
    };
    assert_eq!(json!(source_update), json!({ "name": "example" }));
}
