//! An unofficial Rust client for BigML's REST API.
//!
//! BigML is an commercial machine-learning service. This unofficial library
//! allows you to talk to BigML from Rust.
//!
//! We focus on passing data to BigML and running WhizzML scripts, though it's
//! pretty easy to add support for new resource types and resource fields. See
//! our [GitHub repository][] for more information.
//!
//! ```no_run(
//! # extern crate bigml;
//! #
//! use bigml::{Client, resource::{execution, Id, Script}};
//! use std::{path::Path, str::FromStr};
//!
//! # fn main() -> bigml::Result<()> {
//! #
//! let username = "username";
//! let api_key = "api_key";
//! let path = Path::new("sample.csv");
//! let script_id: Id<Script> = Id::from_str("script/123abc")?;
//!
//! // Create a BigML client.
//! let client = bigml::Client::new(username, api_key)?;
//!
//! // Create a source.
//! let source = client.create_source_from_path_and_wait(path)?;
//! println!("{:?}", source);
//!
//! // Execute the script.
//! let mut args = execution::Args::default();
//! args.set_script(script_id);
//! args.add_input("source-id", &source.resource)?;
//! args.add_output("my-output");
//! let execution = client.create_and_wait(&args)?;
//! println!("{:?}", execution);
//! #
//! #   Ok(())
//! # }
//! ```
//!
//! For more information, see the [BigML API][] and our [example code][].
//!
//! [GitHub repository]: https://github.com/faradayio/bigml-rs
//! [BigML API]: https://bigml.com/api
//! [example code]: https://github.com/faradayio/bigml-rs/tree/master/examples

#![warn(missing_docs)]

#[macro_use]
extern crate bigml_derive;
extern crate chrono;
#[macro_use]
extern crate failure;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate mime;
extern crate reqwest;
extern crate serde;
#[cfg_attr(test, macro_use)]
extern crate serde_json;
extern crate url;
extern crate uuid;

pub use client::Client;
pub use errors::*;
pub use progress::{ProgressCallback, ProgressOptions};
pub use wait::WaitOptions;

#[macro_use]
pub mod wait;
mod client;
mod errors;
mod multipart_form_data;
mod progress;
pub mod resource;
