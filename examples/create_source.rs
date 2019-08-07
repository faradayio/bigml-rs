use bigml;
use env_logger;

use bigml::resource::Resource;
use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process;

fn main() {
    env_logger::init();

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
    let initial_response = client
        .create_source_from_path(&path)
        .expect("can't create source");
    let response = client
        .wait(initial_response.id())
        .expect("error waiting for resource");

    println!("{:#?}", &response);
}
