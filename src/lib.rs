//! A Rust client for BigML's REST API.

// Needed for error-chain.
#![recursion_limit = "1024"]

#![warn(missing_docs)]

extern crate chrono;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
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

mod client;
mod errors;
mod multipart_form_data;
pub mod resource;
