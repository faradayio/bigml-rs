// This demo script doesn't currently work, because it relies on
// `create_source_from_path`.
#![allow(deprecated)]

use bigml::{self, resource::Resource};
use futures::{executor::block_on, FutureExt};
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use tracing_subscriber::{
    fmt::{format::FmtSpan, Subscriber},
    prelude::*,
    EnvFilter,
};

fn main() {
    // Configure tracing.
    let filter = EnvFilter::from_default_env();
    Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_env_filter(filter)
        .finish()
        .init();

    let bigml_username = env::var("BIGML_USERNAME")
        .expect("pass BIGML_USERNAME as an environment variable");
    let bigml_api_key = env::var("BIGML_API_KEY")
        .expect("pass BIGML_PASSWORD as an environment variable");

    let args: Vec<_> = env::args().collect();
    if args.len() != 2 {
        writeln!(&mut io::stderr(), "Usage: create_source <FILENAME>")
            .expect("can't write to stderr, giving up");
        process::exit(1);
    }
    let path = Path::new(&args[1]).to_owned();

    let client = bigml::Client::new(bigml_username, bigml_api_key)
        .expect("can't create bigml::Client");
    let initial_response = block_on(client.create_source_from_path(path).boxed())
        .expect("can't create source");
    let response = block_on(client.wait(initial_response.id()).boxed())
        .expect("error waiting for resource");

    println!("{:#?}", &response);
}
