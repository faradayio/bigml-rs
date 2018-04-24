//! Utilities for waiting, timeouts and error retries.

use std::cmp::max;
use std::fmt::Display;
use std::result;
use std::thread::sleep;
use std::time::{Duration, SystemTime};

use errors::*;

/// Minimum sleep time recommended by BigML support to avoid ban.
const MIN_SLEEP_SECS: u64 = 4;

/// Options controlling how long we wait and what makes us give up.
/// This uses a "builder" pattern, so you can write:
///
/// ```
/// use std::time::Duration;
/// use bigml::wait::WaitOptions;
///
/// let options = WaitOptions::default()
///     .timeout(Duration::from_secs(120))
///     .allowed_errors(5);
/// ```
pub struct WaitOptions {
    /// Time between each retry.
    timeout: Option<Duration>,

    /// How long to wait between retries.
    retry_interval: Duration,

    /// How many errors are we allowed before giving up?
    allowed_errors: u16,
}

impl WaitOptions {
    /// Set an optional timeout after which to abandon this `wait`.
    pub fn timeout<D: Into<Option<Duration>>>(mut self, timeout: D) -> Self {
        self.timeout = timeout.into();
        self
    }

    /// How long should we wait between retries? Defaults to 10 seconds. Note
    /// that BigML has suggested not polling more often than every 4 seconds,
    /// (to avoid losing API access) so if you set a lower value, this will be
    pub fn retry_interval(mut self, interval: Duration) -> Self {
        self.retry_interval = interval;
        self
    }

    /// How many errors should be ignored before giving up? This can be useful
    /// for long-running `Execution` jobs, where we don't want a transient
    /// network error to result in failure.
    pub fn allowed_errors(mut self, count: u16) -> Self {
        self.allowed_errors = count;
        self
    }
}

impl Default for WaitOptions {
    fn default() -> Self {
        Self {
            timeout: None,
            retry_interval: Duration::from_secs(10),
            allowed_errors: 0,
        }
    }
}

/// Return this value from a `wait` callback.
pub enum WaitStatus<T> {
    /// The task has finished.
    Finished(T),

    /// The task hasn't finished yet, so wait a while and try again.
    Waiting,
}

impl<T> From<T> for WaitStatus<T> {
    fn from(value: T) -> Self {
        WaitStatus::Finished(value)
    }
}

/// Call `f` repeatedly, wait for it to return `WaitStatus::Finished`, an error,
/// or a timeout. Honors `WaitOptions`.
///
/// ```
/// # extern crate bigml;
/// # extern crate failure;
/// # fn main() {
/// use bigml::wait::{wait, WaitOptions, WaitStatus};
/// use failure::Error;
///
/// let value = wait::<_, failure::Error, _>(&WaitOptions::default(), || {
///     Ok(WaitStatus::Finished("my value"))
/// }).expect("an error occured while waiting");
///
/// assert_eq!(value, "my value");
/// # }
/// ```
///
/// If you return `Ok(WaitStatus::Waiting)` instead, this function will wait
/// some number of seconds, and then try again.
pub fn wait<T, E, F>(
    options: &WaitOptions,
    mut f: F,
) -> result::Result<T, E>
where
    F: FnMut() -> result::Result<WaitStatus<T>, E>,
    E: Display,
    Error: Into<E>,
{
    let deadline = options.timeout.map(|to| SystemTime::now() + to);
    let mut errors_seen = 0;
    loop {
        // Check to see if we've exceeded our deadline (if we have one).
        if let Some(deadline) = deadline {
            if SystemTime::now() > deadline {
                return Err(Error::Timeout.into());
            }
        }

        // Call the function we're waiting on.
        match f() {
            Ok(WaitStatus::Finished(value)) => { return Ok(value); }
            Ok(WaitStatus::Waiting) => {}
            Err(ref e) if errors_seen < options.allowed_errors => {
                errors_seen += 1;
                error!(
                    "Got error, will retry ({}/{}): {}",
                    errors_seen,
                    options.allowed_errors,
                    e,
                );
            }
            Err(e) => { return Err(e); }
        }

        // Sleep until our next call.
        sleep(max(Duration::from_secs(MIN_SLEEP_SECS), options.retry_interval));
    }
}
