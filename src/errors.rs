//! Declare our error types using error-chain.

// Unforunately, error_chain does not generate docs for all the types it
// defines.
#![allow(missing_docs)]

use reqwest;
use std::path::PathBuf;
use url::Url;

error_chain! {
    errors {
        /// We could not access the specified URL.
        CouldNotAccessUrl(url: Url) {
            description("could not access URL")
            display("could not access '{}'", &url)
        }

        /// We failed to read the specified file.
        CouldNotReadFile(path: PathBuf) {
            description("could not read file")
            display("could not read file '{}'", &path.display())
        }

        /// We received an unexpected HTTP status code.
        UnexpectedHttpStatus(status: reqwest::StatusCode) {
            description("Unexpected HTTP status")
            display("{}", &status)
        }
    }
}
