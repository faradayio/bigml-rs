//! A CLI tool for executing BigML jobs in parallel.

use anyhow::{Error, Result};
use bigml::{
    self,
    resource::{execution, Execution, Id, Resource, Script},
    try_wait, try_with_permanent_failure,
    wait::{wait, BackoffType, WaitOptions, WaitStatus},
    Client,
};
use clap::Parser;
use futures::{self, stream, FutureExt, StreamExt, TryStreamExt};
use regex::Regex;
use std::{process, sync::Arc, time::Duration};
use tokio::io;
use tokio_util::codec::{FramedRead, FramedWrite, LinesCodec};
use tracing::{debug, error, instrument};
use tracing_subscriber::{
    fmt::{format::FmtSpan, Subscriber},
    prelude::*,
    EnvFilter,
};

mod execution_input;
mod line_delimited_json_codec;

use execution_input::ExecutionInput;
use line_delimited_json_codec::LineDelimitedJsonCodec;

/// Our standard stream type, containing values of type `T`.
type BoxStream<T> = futures::stream::BoxStream<'static, Result<T>>;

/// Our standard future type, yield a value of type `T`.
type BoxFuture<T> = futures::future::BoxFuture<'static, Result<T>>;

/// Our command-line arguments.
#[derive(Debug, Parser)]
#[command(
    name = "bigml-parallel",
    about = "Execute WhizzML script in parallel over one or more BigML resources",
    author,
    version
)]
struct Opt {
    /// The WhizzML script ID to run.
    #[arg(long = "script", short = 's')]
    script: Id<Script>,

    /// The name to use for our execution objects.
    #[arg(long = "name", short = 'n')]
    name: Option<String>,

    /// The resource IDs to process. (Alternatively, pipe resource IDs on standard
    /// input, one per line.)
    #[arg(long = "resource", short = 'r')]
    resources: Vec<String>,

    /// The input name used to pass the dataset.
    #[arg(long = "resource-input-name", short = 'R', default_value = "resource")]
    resource_input_name: String,

    /// Extra inputs to our WhizzML script, specified as "name=value". These
    /// will be parsed as JSON if possible, or treated as strings otherwise.
    #[arg(long = "input", short = 'i')]
    inputs: Vec<ExecutionInput>,

    /// Expected outputs to our WhizzML script, specified as "name".
    #[arg(long = "output", short = 'o')]
    outputs: Vec<String>,

    /// How many BigML tasks should we use at a time?
    #[arg(long = "max-tasks", short = 'J', default_value = "2")]
    max_tasks: usize,

    /// Apply a tag to the BigML resources we create.
    #[arg(long = "tag")]
    tags: Vec<String>,

    /// A regular expression specifying which WhizzML script errors should be retried.
    #[arg(long = "retry-on")]
    retry_on: Option<Regex>,

    /// How many times should we retry a failed execution matching --retry-on?
    #[arg(long = "retry-count", default_value = "0")]
    retry_count: u16,
}

/// A `main` function that prints out pretty errors. All the real work is done
/// in `run`
#[tokio::main]
async fn main() {
    // Configure tracing.
    let filter = EnvFilter::from_default_env();
    Subscriber::builder()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_env_filter(filter)
        .finish()
        .init();

    // Run our main program and watch for errors.
    if let Err(err) = run().await {
        eprintln!("Error: {}", err);
        for cause in err.chain().skip(1) {
            eprintln!("  caused by: {}", cause);
        }
        eprintln!("{}", err.backtrace());
        process::exit(1);
    }
}

/// Our real `main` function, called by `main`.
#[instrument(level = "trace", name = "bigml_parallel")]
async fn run() -> Result<()> {
    let opt = Opt::parse();
    debug!("command-line options: {:?}", opt);

    // We want to represent our input resource IDs as an asynchronous stream,
    // which will make it very easy to have controlled parallel execution.
    let resources: BoxStream<String> = if !opt.resources.is_empty() {
        // Turn our `--resource` arguments into a stream.
        let resources = opt.resources.clone();
        stream::iter(resources.into_iter().map(Ok)).boxed()
    } else {
        // Parse standard input as a stream of dataset IDs.
        let lines = FramedRead::new(io::stdin(), LinesCodec::new());
        lines.map_err(|e| -> Error { e.into() }).boxed()
    };

    // Wrap our command line arguments in a thread-safe reference counter, so
    // that all our parallel tasks can access them.
    let opt = Arc::new(opt);

    // Transform our stream of IDs into a stream of _futures_, each of which will
    // return an `Execution` object from BigML.
    let opt2 = opt.clone();
    let execution_futures: BoxStream<BoxFuture<Execution>> = resources
        .map_ok(move |resource| {
            resource_id_to_execution(opt2.clone(), resource).boxed()
        })
        .boxed();

    // Now turn the stream of futures into a stream of executions, using
    // `buffer_unordered` to execute up to `opt.max_tasks` in parallel. This is
    // basically the "payoff" for all the async code up above, and it is
    // wonderful.
    //
    // TODO: In tokio 0.1, this had weird buffering behavior, and
    // appeared to wait until it buffered `opt.max_tasks` items. I have
    // not verified this in tokio 0.2.
    let executions: BoxStream<Execution> = execution_futures
        .try_buffer_unordered(opt.max_tasks)
        .boxed();

    // Copy our stream of `Execution`s to standard output as line-delimited
    // JSON.
    //
    // TODO: `forward` may also have weird buffering behavior.
    let stdout = FramedWrite::new(io::stdout(), LineDelimitedJsonCodec::new());
    executions.forward(stdout).await?;
    Ok(())
}

/// Use our command-line options and a resource ID to create and run a BigML
/// execution.
#[instrument(level = "debug", fields(script = %opt.script), skip(opt))]
async fn resource_id_to_execution(
    opt: Arc<Opt>,
    resource: String,
) -> Result<Execution> {
    // Specify what script to run.
    let mut args = execution::Args::default();
    args.script = Some(opt.script.clone());

    // Optionally set the script name.
    if let Some(name) = opt.name.as_ref() {
        args.name = Some(name.to_owned());
    }

    // Specify the input dataset.
    args.add_input(&opt.resource_input_name, &resource)?;

    // Add any other inputs.
    for input in &opt.inputs {
        args.add_input(&input.name, &input.value)?;
    }

    // Add outputs.
    for output in &opt.outputs {
        args.add_output(output);
    }

    // Add tags.
    args.tags = opt.tags.clone();

    // Execute our script, with three types of retries.
    //
    // 1. Retry the entire execution if it fails with an error that looks
    //    transient. This is often caused by BigML overload, as far as we can
    //    tell.
    //     a. Retry the creation if that fails with a transient error. This is often
    //        caused by running out of slots.
    //     b. Internally retry the `wait` if it fails with a transient network error.
    let exec_wait_opt = WaitOptions::default()
        .retry_interval(Duration::from_secs(2 * 60))
        .backoff_type(BackoffType::Exponential)
        .allowed_errors(opt.retry_count);
    let execution = wait(&exec_wait_opt, || {
        create_and_wait_execution(&args, opt.retry_on.as_ref())
    })
    .await?;
    Ok(execution)
}

/// Create a BigML execution and wait for it to finish.
///
/// Returns a `WaitStatus`, allowing our caller to retry us as necessary.
#[instrument(level = "trace")]
async fn create_and_wait_execution(
    args: &execution::Args,
    retry_on: Option<&Regex>,
) -> WaitStatus<Execution, bigml::Error> {
    // If we can't create a client, just give up immediately.
    let client = try_with_permanent_failure!(Client::new_from_env());

    // Attempt to create a new execution. This has custom retry logic with
    // unusually long timeouts because temporary failures here are generally
    // caused by hitting API limits, and if we wait 30 minutes, somebody else's
    // batch job may finish. But if those retries fail, we want to fail
    // permanently.
    let create_wait_opt = WaitOptions::default()
        .retry_interval(Duration::from_secs(60))
        .backoff_type(BackoffType::Exponential)
        .allowed_errors(6);
    let execution = try_with_permanent_failure!(
        wait(&create_wait_opt, || {
            async {
                // We use `try_wait`, because it knows which errors are
                // permanent and which are temporary.
                WaitStatus::Finished(try_wait!(client.create(args).await))
            }
        })
        .await
    );

    // `client.wait` has its own internal retry logic, but it only triggers for
    // things like failed HTTP calls to BigML. We also want to retry any script
    // errors that match `retry_on`.
    match client.wait(execution.id()).await {
        Ok(execution) => WaitStatus::Finished(execution),
        Err(err) => match (err.original_bigml_error(), retry_on) {
            // We failed with a `WaitError`, we have a `retry_on` pattern, and that
            // pattern matches our error message from BigML.
            (bigml::Error::WaitFailed { message, .. }, Some(retry_on))
                if retry_on.is_match(message) =>
            {
                error!("{} failed with temporary error: {}", execution.id(), err);
                WaitStatus::FailedTemporarily(err)
            }

            // We have a different kind of error.
            _ => WaitStatus::FailedPermanently(err),
        },
    }
}
