//! Resource identifiers used by the BigML API.

use serde::de::Unexpected;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use url::Url;

use super::Resource;
use crate::errors::*;

/// A strongly-typed "resource ID" used to identify many different kinds of
/// BigML resources.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Id<R: Resource> {
    /// The ID of the resource.
    id: String,
    /// A special 0-byte field which exists just to mention the type `R`
    /// inside the struct, and thus avoid compiler errors about unused type
    /// parameters.
    _phantom: PhantomData<R>,
}

impl<R: Resource> Id<R> {
    /// Get this resource as a string.
    pub fn as_str(&self) -> &str {
        &self.id
    }

    /// Get a URL pointing at the human-readable version of this resource.
    pub fn dashboard_url(&self) -> Url {
        Url::parse(&format!("https://bigml.com/dashboard/{}", self))
            // This should never fail to parse.
            .expect("dashboard URL unexpectedly failed to parse")
    }
}

impl<R: Resource> FromStr for Id<R> {
    type Err = Error;

    fn from_str(id: &str) -> Result<Self> {
        if id.starts_with(R::id_prefix()) {
            Ok(Id {
                id: id.to_owned(),
                _phantom: PhantomData,
            })
        } else {
            Err(Error::WrongResourceType {
                expected: R::id_prefix(),
                found: id.to_owned(),
            })
        }
    }
}

impl<R: Resource> fmt::Debug for Id<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> fmt::Display for Id<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<'de, R: Resource> Deserialize<'de> for Id<R> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let id: String = String::deserialize(deserializer)?;
        if id.starts_with(R::id_prefix()) {
            Ok(Id {
                id,
                _phantom: PhantomData,
            })
        } else {
            let unexpected = Unexpected::Str(&id);
            let expected =
                format!("a BigML resource ID starting with '{}'", R::id_prefix());
            Err(<D::Error as serde::de::Error>::invalid_value(
                unexpected,
                &&expected[..],
            ))
        }
    }
}

impl<R: Resource> Serialize for Id<R> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.id.serialize(serializer)
    }
}
