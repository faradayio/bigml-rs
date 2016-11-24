//! A Rust client for BigML's REST API.

// Needed for serde_derive until Macros 1.1 stablizes (in Rust 1.15?).
#![feature(proc_macro)]

// Needed for error-chain.
#![recursion_limit = "1024"]

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mime;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate url;
extern crate uuid;

pub use client::Client;
pub use errors::*;
pub use serde_types::*;

mod client;
mod errors;
mod multipart_form_data;
mod serde_types;
mod util;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
