use futures::executor::block_on;
#[macro_use]
extern crate log;

use anyhow::{Context, Error};
use bigml::resource;
use futures::FutureExt;
use std::{
    env,
    io::{self, Write},
    process, result,
    str::FromStr,
};

/// A custom `Result`, for convenience.
pub type Result<T, E = Error> = result::Result<T, E>;

/// A local helper function which does the real work, and which can return
/// an error (unlike `main`).
fn helper(
    script_id: &str,
    inputs: &[(String, String)],
    outputs: &[String],
) -> Result<()> {
    // Get our BigML credentials.
    let bigml_username = env::var("BIGML_USERNAME")
        .context("pass BIGML_USERNAME as an environment variable")?;
    let bigml_api_key = env::var("BIGML_API_KEY")
        .context("pass BIGML_API_KEY as an environment variable")?;

    // Create a BigML client.
    let client = bigml::Client::new(bigml_username, bigml_api_key)?;

    // Prepare our execution arguments.
    let mut args = resource::execution::Args::default();
    args.set_script(resource::Id::<resource::Script>::from_str(script_id)?);
    for &(ref name, ref value) in inputs {
        args.add_input(name.to_owned(), value)?;
    }
    for name in outputs {
        args.add_output(name.to_owned());
    }

    // Execute the script, wait for it to complete, and print the result.
    let execution = block_on(client.create_and_wait(&args).boxed())?;
    println!("{:#?}", execution);

    Ok(())
}

fn usage() -> ! {
    writeln!(
        &mut io::stderr(),
        "\
         Usage: create_execution <SCRIPT_ID> <INPUT>=<VALUE>... --output=<NAME>..."
    )
    .expect("can't write to stderr, giving up");
    process::exit(1);
}

fn main() {
    env_logger::init();

    // Parse our command line options.
    let mut script_id = None;
    let mut inputs = vec![];
    let mut outputs = vec![];
    let mut first = true;
    for arg in env::args().skip(1) {
        if first {
            // Handle <SCRIPT_ID> arg.
            first = false;
            debug!("script_id: {}", &arg);
            script_id = Some(arg);
        } else {
            let chunks = arg.splitn(2, '=').collect::<Vec<_>>();
            debug!("parsing: {} = {}", chunks[0], chunks[1]);
            if chunks.len() == 2 {
                if chunks[0] == "--output" {
                    // Handle --output=<NAME> arg.
                    outputs.push(chunks[1].to_owned());
                } else {
                    // Handle <INPUT>=<NAME> arg.
                    inputs.push((chunks[0].to_owned(), chunks[1].to_owned()));
                }
            } else {
                usage();
            }
        }
    }
    let script_id = match script_id {
        Some(script_id) => script_id,
        None => usage(),
    };

    // Dispatch to our helper function and report any errors it returns.
    if let Err(err) = helper(&script_id, &inputs, &outputs) {
        eprint!("ERROR");
        for e in err.chain() {
            eprint!(": {}", e);
        }
        eprintln!();
        eprintln!("{:?}", err.backtrace());
        process::exit(1);
    }
}
