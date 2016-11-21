//! A Rust client for BigML's REST API.

#![recursion_limit = "1024"]

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mime;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate url;
extern crate uuid;

pub use client::Client;
pub use errors::*;
pub use serde_types::*;

mod client;
mod errors;
mod multipart_form_data;
mod util;

/// Types preprocessed by serde.
mod serde_types {
    include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
