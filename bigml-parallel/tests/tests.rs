//! Integration tests for the entire `bigml-parallel` executable.

use bigml::{
    resource::{script, source, Execution, Resource, StatusCode},
    Client,
};
use cli_test_dir::*;
use common_failures::prelude::*;
use futures::{FutureExt, TryFutureExt};
use serde_json;
use std::{env, future::Future, io::Write};
use tokio::runtime::Runtime;

/// Create a BigML client using environment varaibles to authenticate.
fn new_client() -> Result<Client> {
    let username =
        env::var("BIGML_USERNAME").context("must specify BIGML_USERNAME")?;
    let api_key = env::var("BIGML_API_KEY").context("must specify BIGML_API_KEY")?;
    Ok(Client::new(username, api_key)?)
}

/// Run the future `f` asynchronously.
fn run_async<F, T>(fut: F) -> Result<T>
where
    F: Future<Output = Result<T>> + Send + 'static,
    T: Send + 'static,
{
    let mut runtime = Runtime::new().expect("Unable to create a runtime");
    runtime.block_on(fut.boxed().compat())
}

#[test]
fn help_flag() {
    let testdir = TestDir::new("bigml-parallel", "help_flag");
    let output = testdir.cmd().arg("--help").expect_success();
    assert!(output.stdout_str().contains("bigml-parallel"));
}

#[test]
fn version_flag() {
    let testdir = TestDir::new("bigml-parallel", "version_flag");
    let output = testdir.cmd().arg("--version").expect_success();
    assert!(output.stdout_str().contains(env!("CARGO_PKG_VERSION")));
}

/// Our test WhizzML script to execute.
static WHIZZML_SCRIPT: &str = r#"
;; Input: source
;; Input: n
;; Output: dataset
;; Output: n_times_2

(define dataset
  (create-and-wait-dataset {"source" source "name" "bigml-parallel test"}))
(define n_times_2 (* n 2))
"#;

#[test]
#[ignore]
fn parallel_executions() {
    let _ = env_logger::try_init();

    let testdir = TestDir::new("bigml-parallel", "parallel_executions");

    // Set up our test infrastructure on BigML.
    let (sources, script) = run_async(async {
        let client = new_client()?;

        // Build some source objects to test.
        let raw_sources = &["id,color\n1,green\n", "id,color\n2,blue\n"];
        let mut sources = vec![];
        for (i, &raw_source) in raw_sources.into_iter().enumerate() {
            let mut args = source::Args::data(raw_source);
            args.disable_datetime = Some(true);
            args.name = Some(format!("bigml-parallel test {}", i));
            let source = client.create_and_wait(&args).await?;
            sources.push(source.id().to_owned());
        }

        // Upload our WhizzML script object.
        let mut args = script::Args::new(WHIZZML_SCRIPT);
        args.inputs
            .push(script::Input::new("source", script::Type::ResourceId));
        args.inputs
            .push(script::Input::new("n", script::Type::Integer));
        let script = client.create_and_wait(&args).await?;

        Ok((sources, script.id().to_owned()))
    })
    .unwrap();

    // Construct standard input with
    let mut input = vec![];
    for source in &sources {
        writeln!(&mut input, "{}", source).unwrap();
    }

    // Run `bigml-parallel`.
    let output = testdir
        .cmd()
        .args(&["-n", "bigml-parallel test"])
        .args(&["-s", &script.to_string()])
        .args(&["-R", "source"])
        .args(&["-i", "n=2"])
        .args(&["-o", "dataset"])
        .args(&["-o", "n_times_2"])
        .args(&["--tag", "bigml-parallel:test"])
        .output_with_stdin(&input)
        .tee_output()
        .expect("error running bigml-parallel");

    // Parse our output as JSON execution resources, and make sure they look
    // reasonable.
    assert_eq!(output.stdout_str().lines().count(), sources.len());
    for line in output.stdout_str().lines() {
        let execution: Execution =
            serde_json::from_str(line).expect("error parsing output JSON");
        assert_eq!(execution.status.code, StatusCode::Finished);
        let outputs = &execution.execution.outputs;
        assert!(outputs.iter().any(|output| output.name == "dataset"));
        assert!(outputs.iter().any(|output| {
            output.name == "n_times_2" && output.value == Some(4.into())
        }));
    }
}
