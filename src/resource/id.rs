//! Resource identifiers used by the BigML API.

#[cfg(feature="postgres")]
use postgres as pg;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use serde::de::Unexpected;
#[cfg(feature="postgres")]
use std::error;
use std::fmt;
use std::marker::PhantomData;
use std::result;
use std::str::FromStr;

use errors::*;
use super::Resource;

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
            Err(ErrorKind::WrongResourceType(R::id_prefix(), id.to_owned()).into())
        }
    }
}

impl<R: Resource> fmt::Debug for Id<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> fmt::Display for Id<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", &self.id)
    }
}

impl<R: Resource> Deserialize for Id<R> {
    fn deserialize<D>(deserializer: D) -> result::Result<Self, D::Error>
        where D: Deserializer
    {
        let id: String = String::deserialize(deserializer)?;
        if id.starts_with(R::id_prefix()) {
            Ok(Id {
                id: id,
                _phantom: PhantomData,
            })
        } else {
            let unexpected = Unexpected::Str(&id);
            let expected = format!("a BigML resource ID starting with '{}'",
                                   R::id_prefix());
            Err(<D::Error as serde::de::Error>::invalid_value(unexpected,
                                                              &&expected[..]))
        }
    }
}

impl<R: Resource> Serialize for Id<R> {
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
        where S: Serializer
    {
        self.id.serialize(serializer)
    }
}

#[cfg(feature="postgres")]
impl<R: Resource> pg::types::ToSql for Id<R> {
    fn to_sql(&self,
              ty: &pg::types::Type,
              out: &mut Vec<u8>,
              ctx: &pg::types::SessionInfo)
              -> result::Result<pg::types::IsNull, Box<error::Error + Sync + Send>>
        where Self: Sized
    {
        self.id.to_sql(ty, out, ctx)
    }

    fn accepts(ty: &pg::types::Type) -> bool where Self: Sized {
        String::accepts(ty)
    }

    to_sql_checked!();
}

#[cfg(feature="postgres")]
impl<R: Resource> pg::types::FromSql for Id<R> {
    fn from_sql(ty: &pg::types::Type, raw: &[u8], ctx: &pg::types::SessionInfo)
                -> result::Result<Self, Box<error::Error + Sync + Send>> {
        String::from_sql(ty, raw, ctx)
            .and_then(|s| {
                // We smash all errors to strings, because `error-chain`
                // doesn't declare errors as `Sync`, which `postgres` wants
                // here.
                Id::from_str(&s).map_err(|e| format!("{}", e).into())
            })
    }

    fn accepts(ty: &pg::types::Type) -> bool {
        String::accepts(ty)
    }
}
