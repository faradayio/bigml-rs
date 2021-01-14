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
//! use bigml::{Client, resource::{execution, Id, Script}};
//! use futures::{executor::block_on, FutureExt, TryFutureExt};
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
//! // Create a source (actually, you should do this via S3 for now).
//! let source =
//!     block_on(client.create_source_from_path_and_wait(path.to_owned()))?;
//! println!("{:?}", source);
//!
//! // Execute the script.
//! let mut args = execution::Args::default();
//! args.set_script(script_id);
//! args.add_input("source-id", &source.resource)?;
//! args.add_output("my-output");
//! let execution = block_on(client.create_and_wait(&args))?;
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

#[macro_use]
extern crate failure;
#[macro_use]
extern crate log;

pub use client::{Client, DEFAULT_BIGML_DOMAIN};
pub use errors::*;
pub use progress::{ProgressCallback, ProgressOptions};
pub use wait::WaitOptions;

#[macro_use]
pub mod wait;
mod client;
mod errors;
mod progress;
pub mod resource;
