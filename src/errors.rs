//! Declare our error types using error-chain.

// Unforunately, error_chain does not generate docs for all the types it
// defines.
#![allow(missing_docs)]

use reqwest;
use serde_json;
use std::io;
use std::path::PathBuf;
use url::Url;

error_chain! {
    foreign_links {
        io::Error, Io;
        serde_json::Error, Json;
    }

    errors {
        /// We could not access the specified URL.
        CouldNotAccessUrl(url: Url) {
            description("could not access URL")
            display("could not access '{}'", &url)
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
