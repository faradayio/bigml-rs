//! Declare our error types using error-chain.

// Unforunately, error_chain does not generate docs for all the types it
// defines.
#![allow(missing_docs, unused_doc_comment)]

use reqwest;
use serde_json;
use std::collections::BTreeMap;
use std::io;
use std::path::PathBuf;
use url::Url;

error_chain! {
    foreign_links {
        Io(io::Error);
        Json(serde_json::Error);
    }

    errors {
        /// We could not access the specified URL.
        ///
        /// **WARNING:** Do not construct this directly, but use
        /// `ErrorKind::could_not_access_url` to handle various URL sanitization
        /// and security issues.
        CouldNotAccessUrl(url: Url) {
            description("could not access URL")
            display("could not access '{}'", url)
        }

        /// We could not get an output value from a WhizzML script.
        CouldNotGetOutput(name: String) {
            description("could not get WhizzML output")
            display("could not get WhizzML output '{}'", &name)
        }

        /// We failed to read the specified file.
        CouldNotReadFile(path: PathBuf) {
            description("could not read file")
            display("could not read file '{}'", &path.display())
        }

        /// We could not access an output value of a WhizzML script.
        OutputNotAvailable {
            description("WhizzML output is not (yet?) available")
            display("WhizzML output is not (yet?) available")
        }

        /// We received an unexpected HTTP status code.
        UnexpectedHttpStatus(status: reqwest::StatusCode, body: String) {
            description("Unexpected HTTP status")
            display("{} ({})", &status, &body)
        }

        /// We found a type mismatch deserializing a BigML resource ID.
        WrongResourceType(expected: &'static str, found: String) {
            description("Wrong BigML resource type found")
            display("Expected BigML resource ID starting with '{}', found '{}'",
                    expected, &found)
        }
    }
}

impl ErrorKind {
    /// Construct an `ErrorKind::CouldNotAccessUrl` value, taking care to
    /// sanitize the URL query.
    pub fn could_not_access_url(url: Url) -> ErrorKind {
        ErrorKind::CouldNotAccessUrl(url_without_api_key(&url))
    }
}

/// Given a URL with a possible `api_key` parameter, replace the `api_key` with
/// `*****` to minimize the risk of leaking credentials into logs somewhere.
fn url_without_api_key(url: &Url) -> Url {
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
    let url = Url::parse("https://www.example.com/?a=b&api_key=12345")
        .expect("could not parse URL");
    let cleaned = url_without_api_key(&url);
    assert_eq!(cleaned.as_str(), "https://www.example.com/?a=b&api_key=*****");
}
