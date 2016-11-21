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
extern crate url;
extern crate uuid;

use serde::{Deserialize, Deserializer};
use std::result;

pub use client::Client;
pub use errors::*;

mod client;
mod errors;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
