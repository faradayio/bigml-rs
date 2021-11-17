//! Declare our error types using error-chain.

// Unforunately, error_chain does not generate docs for all the types it
// defines.
#![allow(missing_docs, unused_doc_comments)]

use reqwest::StatusCode;
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use std::result;
use url::Url;

/// A custom `Result`, for convenience.
pub type Result<T, E = Error> = result::Result<T, E>;

/// A BigML-related error.
#[derive(Debug, Fail)]
#[non_exhaustive]
pub enum Error {
    /// We could not access the specified URL.
    ///
    /// **WARNING:** Do not construct this directly, but use
    /// `Error::could_not_access_url` to handle various URL sanitization and
    /// security issues.
    #[fail(display = "error accessing '{}': {}", url, error)]
    CouldNotAccessUrl {
        url: Url,
        /*#[cause]*/ error: Box<Error>,
    },

    /// We could not get an output value from a WhizzML script.
    #[fail(display = "could not get WhizzML output '{}': {}", name, error)]
    CouldNotGetOutput {
        name: String,
        /*#[cause]*/ error: Box<Error>,
    },

    /// We could not parse the specified URL.
    ///
    /// **WARNING:** This takes a domain, not the full URL that we couldn't
    /// parse, because we want to be careful to exclude credentials from error
    /// messages, and we can't remove credentials from a URL we can't parse.
    #[fail(
        display = "could not parse a URL with the domain '{}': {}",
        domain, error
    )]
    CouldNotParseUrlWithDomain {
        domain: String,
        /*#[cause]*/ error: Box<url::ParseError>,
    },

    /// We could not read a file.
    #[fail(display = "could not read file {:?}: {}", path, error)]
    CouldNotReadFile {
        path: PathBuf,
        /*#[cause]*/ error: Box<Error>,
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
        status: StatusCode,
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
            error: Box::new(error.into()),
        }
    }

    pub(crate) fn could_not_get_output<E>(name: &str, error: E) -> Error
    where
        E: Into<Error>,
    {
        Error::CouldNotGetOutput {
            name: name.to_owned(),
            error: Box::new(error.into()),
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
            error: Box::new(error),
        }
    }

    pub(crate) fn could_not_read_file<P, E>(path: P, error: E) -> Error
    where
        P: Into<PathBuf>,
        E: Into<Error>,
    {
        Error::CouldNotReadFile {
            path: path.into(),
            error: Box::new(error.into()),
        }
    }

    /// Is this error likely to be temporary?
    pub fn might_be_temporary(&self) -> bool {
        match self {
            Error::CouldNotAccessUrl { error, .. } => error.might_be_temporary(),
            Error::CouldNotGetOutput { error, .. } => error.might_be_temporary(),
            Error::CouldNotReadFile { error, .. } => error.might_be_temporary(),
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

    /// Return the original `bigml::Error` that caused this error, without any
    /// wrapper errors.
    pub fn original_bigml_error(&self) -> &Error {
        match self {
            Error::CouldNotAccessUrl { error, .. } => error.original_bigml_error(),
            Error::CouldNotGetOutput { error, .. } => error.original_bigml_error(),
            Error::CouldNotReadFile { error, .. } => error.original_bigml_error(),

            Error::CouldNotParseUrlWithDomain { .. }
            | Error::Other { .. }
            | Error::OutputNotAvailable
            | Error::PaymentRequired { .. }
            | Error::Timeout
            | Error::UnexpectedHttpStatus { .. }
            | Error::WaitFailed { .. }
            | Error::WrongResourceType { .. } => self,
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
        // TODO: We might be able to classify more `serde` errors as temporary
        // now.
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
