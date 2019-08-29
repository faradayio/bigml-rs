//! Declare our error types using error-chain.

// Unforunately, error_chain does not generate docs for all the types it
// defines.
#![allow(missing_docs, unused_doc_comments)]

use failure;
use reqwest;
use serde_json;
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::result;
use url::Url;

/// A custom `Result`, for convenience.
pub type Result<T> = result::Result<T, Error>;

/// A BigML-related error.
#[derive(Debug, Fail)]
pub enum Error {
    /// We could not access the specified URL.
    ///
    /// **WARNING:** Do not construct this directly, but use
    /// `Error::could_not_access_url` to handle various URL sanitization and
    /// security issues.
    #[fail(display = "error accessing '{}': {}", url, error)]
    CouldNotAccessUrl {
        url: Url,
        /*#[cause]*/ error: failure::Error,
    },

    /// We could not get an output value from a WhizzML script.
    #[fail(display = "could not get WhizzML output '{}': {}", name, error)]
    CouldNotGetOutput {
        name: String,
        /*#[cause]*/ error: failure::Error,
    },

    /// We could not read a file.
    #[fail(display = "could not read file {:?}: {}", path, error)]
    CouldNotReadFile {
        path: PathBuf,
        /*#[cause]*/ error: failure::Error,
    },

    /// We could not access an output value of a WhizzML script.
    #[fail(display = "WhizzML output is not (yet?) available")]
    OutputNotAvailable,

    /// BigML says that payment is required for this request, perhaps because
    /// we have hit plan limits.
    #[fail(display = "BigML payment required for {} ({})", url, body)]
    PaymentRequired { url: Url, body: String },

    /// A request timed out.
    #[fail(display = "The operation timed out")]
    Timeout,

    /// We received an unexpected HTTP status code.
    #[fail(display = "{} for {} ({})", status, url, body)]
    UnexpectedHttpStatus {
        url: Url,
        status: reqwest::StatusCode,
        body: String,
    },

    /// We tried to create a BigML resource, but we failed. Display a dashboard
    /// URL to make it easy to look up the actual error.
    #[fail(display = "https://bigml.com/dashboard/{} failed ({})", id, message)]
    WaitFailed {
        /// The ID of the resource that we were waiting on.
        id: String,
        /// The message that was returned.
        message: String,
    },

    /// We found a type mismatch deserializing a BigML resource ID.
    #[fail(
        display = "Expected BigML resource ID starting with '{}', found '{}'",
        expected, found
    )]
    WrongResourceType {
        expected: &'static str,
        found: String,
    },

    /// Another kind of error occurred.
    #[fail(display = "{}", error)]
    Other { /*#[cause]*/ error: failure::Error, },

    /// Add a hidden member for future API extensibility.
    #[doc(none)]
    #[fail(display = "This error should never have occurred")]
    __Nonexclusive,
}

impl Error {
    /// Construct an `Error::CouldNotAccessUrl` value, taking care to
    /// sanitize the URL query.
    pub(crate) fn could_not_access_url<E>(url: &Url, error: E) -> Error
    where
        E: Into<failure::Error>,
    {
        Error::CouldNotAccessUrl {
            url: url_without_api_key(&url),
            error: error.into(),
        }
    }

    pub(crate) fn could_not_get_output<E>(name: &str, error: E) -> Error
    where
        E: Into<failure::Error>,
    {
        Error::CouldNotGetOutput {
            name: name.to_owned(),
            error: error.into(),
        }
    }
}

impl From<failure::Error> for Error {
    fn from(error: failure::Error) -> Error {
        Error::Other { error }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Other {
            error: error.into(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        Error::Other {
            error: error.into(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Other {
            error: error.into(),
        }
    }
}

/// Given a URL with a possible `api_key` parameter, replace the `api_key` with
/// `*****` to minimize the risk of leaking credentials into logs somewhere.
pub(crate) fn url_without_api_key(url: &Url) -> Url {
    // Extract all our query parameters.
    let mut query = BTreeMap::new();
    for (k, v) in url.query_pairs() {
        query.insert(k.into_owned(), v.into_owned());
    }

    // Build a new URL, setting our query parameters manually, and excluding
    // the problematic one.
    let mut new_url = url.to_owned();
    {
        let mut serializer = new_url.query_pairs_mut();
        serializer.clear();
        for (k, v) in query.iter() {
            if k == "api_key" {
                serializer.append_pair(k, "*****");
            } else {
                serializer.append_pair(k, v);
            }
        }
    }
    new_url
}

#[test]
fn url_without_api_key_is_sanitized() {
    let url = Url::parse("https://www.example.com/foo?a=b&api_key=12345")
        .expect("could not parse URL");
    let cleaned = url_without_api_key(&url);
    assert_eq!(
        cleaned.as_str(),
        "https://www.example.com/foo?a=b&api_key=*****"
    );
}
