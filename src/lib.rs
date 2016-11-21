//! A Rust client for BigML's REST API.

#![recursion_limit = "1024"]

#![warn(missing_docs)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
extern crate mime;
extern crate multipart;
extern crate reqwest;
extern crate url;

pub use client::Client;
pub use errors::*;

mod client;
mod errors;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
