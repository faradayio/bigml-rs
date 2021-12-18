//! Declare our error types using error-chain.

// Unforunately, error_chain does not generate docs for all the types it
// defines.
#![allow(missing_docs, unused_doc_comments)]

use reqwest::StatusCode;
use std::collections::BTreeMap;
use std::error::Error as StdError;
use std::io;
use std::path::PathBuf;
use std::result;
use thiserror::Error;
use url::Url;

/// A custom `Result`, for convenience.
pub type Result<T, E = Error> = result::Result<T, E>;

/// A BigML-related error.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// We could not access the specified URL.
    ///
    /// **WARNING:** Do not construct this directly, but use
    /// `Error::could_not_access_url` to handle various URL sanitization and
    /// security issues.
    #[non_exhaustive]
    #[error("error accessing '{url}': {source}")]
    CouldNotAccessUrl { url: Url, source: Box<Error> },

    /// We could not get an output value from a WhizzML script.
    #[non_exhaustive]
    #[error("could not get WhizzML output '{name}': {source}")]
    CouldNotGetOutput { name: String, source: Box<Error> },

    /// We could not parse the specified URL.
    ///
    /// **WARNING:** This takes a domain, not the full URL that we couldn't
    /// parse, because we want to be careful to exclude credentials from error
    /// messages, and we can't remove credentials from a URL we can't parse.
    #[non_exhaustive]
    #[error("could not parse a URL with the domain '{domain}': {source}")]
    CouldNotParseUrlWithDomain {
        domain: String,
        source: Box<url::ParseError>,
    },

    /// We could not read a file.
    #[non_exhaustive]
    #[error("could not read file {path:?}: {source}")]
    CouldNotReadFile { path: PathBuf, source: Box<Error> },

    /// The user must specify the environment variable `var`.
    #[non_exhaustive]
    #[error("must specify {var}")]
    MissingEnvVar { var: String },

    /// We could not access an output value of a WhizzML script.
    #[non_exhaustive]
    #[error("WhizzML output is not (yet?) available")]
    OutputNotAvailable {},

    /// BigML says that payment is required for this request, perhaps because
    /// we have hit plan limits.
    #[non_exhaustive]
    #[error("BigML payment required for {url} ({body})")]
    PaymentRequired { url: Url, body: String },

    /// A request timed out.
    #[non_exhaustive]
    #[error("The operation timed out")]
    Timeout {},

    /// We received an unexpected HTTP status code.
    #[non_exhaustive]
    #[error("{status} for {url} ({body})")]
    UnexpectedHttpStatus {
        url: Url,
        status: StatusCode,
        body: String,
    },

    /// We encountered an unknown BigML value type.
    #[non_exhaustive]
    #[error("unknown BigML type {type_name:?}")]
    UnknownBigMlType { type_name: String },

    /// We tried to create a BigML resource, but we failed. Display a dashboard
    /// URL to make it easy to look up the actual error.
    #[non_exhaustive]
    #[error("https://bigml.com/dashboard/{id} failed ({message})")]
    WaitFailed {
        /// The ID of the resource that we were waiting on.
        id: String,
        /// The message that was returned.
        message: String,
    },

    /// We found a type mismatch deserializing a BigML resource ID.
    #[non_exhaustive]
    #[error("Expected BigML resource ID starting with '{expected}', found '{found}'")]
    WrongResourceType {
        expected: &'static str,
        found: String,
    },

    /// Another kind of error occurred.
    #[non_exhaustive]
    #[error("{source}")]
    Other {
        /// The original error.
        ///
        /// We add `Send + Sync` to make it easy to use in the presence of threads, and
        /// `'static` to make sure it depends on no borrowed data.
        #[from]
        source: Box<dyn StdError + Send + Sync + 'static>,
    },
}

impl Error {
    /// Construct an `Error::CouldNotAccessUrl` value, taking care to
    /// sanitize the URL query.
    pub(crate) fn could_not_access_url<E>(url: &Url, error: E) -> Error
    where
        E: Into<Error>,
    {
        Error::CouldNotAccessUrl {
            url: url_without_api_key(url),
            source: Box::new(error.into()),
        }
    }

    pub(crate) fn could_not_get_output<E>(name: &str, error: E) -> Error
    where
        E: Into<Error>,
    {
        Error::CouldNotGetOutput {
            name: name.to_owned(),
            source: Box::new(error.into()),
        }
    }

    /// Construct an `Error::CouldNotParseUrlWithDomain` value.
    pub(crate) fn could_not_parse_url_with_domain<S>(
        domain: S,
        error: url::ParseError,
    ) -> Error
    where
        S: Into<String>,
    {
        Error::CouldNotParseUrlWithDomain {
            domain: domain.into(),
            source: Box::new(error),
        }
    }

    pub(crate) fn could_not_read_file<P, E>(path: P, error: E) -> Error
    where
        P: Into<PathBuf>,
        E: Into<Error>,
    {
        Error::CouldNotReadFile {
            path: path.into(),
            source: Box::new(error.into()),
        }
    }

    /// Is this error likely to be temporary?
    pub fn might_be_temporary(&self) -> bool {
        match self {
            Error::CouldNotAccessUrl { source, .. } => source.might_be_temporary(),
            Error::CouldNotGetOutput { source, .. } => source.might_be_temporary(),
            Error::CouldNotReadFile { source, .. } => source.might_be_temporary(),
            // This error occurs when all your BigML "slots" are used and
            // they're suggesting you upgrade. Backing off may free up slots.
            Error::PaymentRequired { .. } => true,
            // Some HTTP status codes also tend to correspond to temporary errors.
            Error::UnexpectedHttpStatus { status, .. } => matches!(
                *status,
                StatusCode::INTERNAL_SERVER_ERROR // I'm not so sure about this one.
                | StatusCode::SERVICE_UNAVAILABLE
                | StatusCode::GATEWAY_TIMEOUT
            ),
            _ => false,
        }
    }

    /// Construct a `MissingEnvVar` value.
    pub(crate) fn missing_env_var<S: Into<String>>(var: S) -> Self {
        Error::MissingEnvVar { var: var.into() }
    }

    /// Return the original `bigml::Error` that caused this error, without any
    /// wrapper errors.
    pub fn original_bigml_error(&self) -> &Error {
        match self {
            Error::CouldNotAccessUrl { source, .. } => source.original_bigml_error(),
            Error::CouldNotGetOutput { source, .. } => source.original_bigml_error(),
            Error::CouldNotReadFile { source, .. } => source.original_bigml_error(),

            Error::CouldNotParseUrlWithDomain { .. }
            | Error::MissingEnvVar { .. }
            | Error::Other { .. }
            | Error::OutputNotAvailable { .. }
            | Error::PaymentRequired { .. }
            | Error::Timeout { .. }
            | Error::UnexpectedHttpStatus { .. }
            | Error::UnknownBigMlType { .. }
            | Error::WaitFailed { .. }
            | Error::WrongResourceType { .. } => self,
        }
    }
}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Error {
        Error::Other {
            source: error.into(),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Error {
        // TODO: We might be able to classify more `serde` errors as temporary
        // now.
        Error::Other {
            source: error.into(),
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Error {
        Error::Other {
            source: error.into(),
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
