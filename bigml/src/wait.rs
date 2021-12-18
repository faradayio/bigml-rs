//! Utilities for waiting, timeouts and error retries.

use std::{
    cmp::max,
    fmt::Display,
    future::Future,
    time::{Duration, SystemTime},
};
use tokio::time::sleep;

use crate::errors::*;

/// Minimum sleep time recommended by BigML support to avoid ban.
const MIN_SLEEP_SECS: u64 = 4;

/// How should we back off if we fail?
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum BackoffType {
    /// Use the same interval for each retry.
    Linear,
    /// Double the interval after each failure.
    Exponential,
}

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

    /// What kind of back-off should we use?
    backoff_type: BackoffType,

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

    /// Should we use linear (default) or exponential backoff?
    pub fn backoff_type(mut self, backoff_type: BackoffType) -> Self {
        self.backoff_type = backoff_type;
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
            backoff_type: BackoffType::Linear,
            allowed_errors: 2,
        }
    }
}

/// Return this value from a `wait` callback.
pub enum WaitStatus<T, E> {
    /// The task has finished.
    Finished(T),

    /// The task hasn't finished yet, so wait a while and try again.
    Waiting,

    /// The task has failed, but the failure is believed to be temporary.
    FailedTemporarily(E),

    /// The task has failed, and we don't believe that it will ever succeed.
    FailedPermanently(E),
}

impl<T> From<Error> for WaitStatus<T, Error> {
    /// Convert an [`Error`] to either [`WaitStatus::FailedTemporarily`] or
    /// [`WaitStatus::FailedPermanently`] depending on [`Error::might_be_temporary`].
    fn from(error: Error) -> Self {
        if error.might_be_temporary() {
            WaitStatus::FailedTemporarily(error)
        } else {
            WaitStatus::FailedPermanently(error)
        }
    }
}

/// Try `e`, and if it fails, allow our wait function to be retried for
/// temporary errors only.
#[macro_export]
macro_rules! try_wait {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return $crate::wait::WaitStatus::<_, $crate::Error>::from(e),
        }
    };
}

/// Try `e`, and if it fails, allow our `wait` function to be retried.
#[macro_export]
macro_rules! try_with_temporary_failure {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return $crate::wait::WaitStatus::FailedTemporarily(e.into()),
        }
    };
}

/// Try `e`, and if it fails, do not allow our `wait` function to be retried.
#[macro_export]
macro_rules! try_with_permanent_failure {
    ($e:expr) => {
        match $e {
            Ok(v) => v,
            Err(e) => return $crate::wait::WaitStatus::FailedPermanently(e.into()),
        }
    };
}

/// Call `f` repeatedly, wait for it to return `WaitStatus::Finished`, an error,
/// or a timeout. Honors `WaitOptions`.
///
/// ```
/// # use futures::{FutureExt, TryFutureExt};
/// # #[tokio::main]
/// # async fn main() -> Result<(), failure::Error> {
/// use bigml::wait::{wait, WaitOptions, WaitStatus};
/// use failure::Error;
///
/// let value = wait::<_, failure::Error, _, _>(&WaitOptions::default(), || {
///     async { WaitStatus::Finished("my value") }
/// }).await?;
///
/// assert_eq!(value, "my value");
/// #   Ok(())
/// # }
/// ```
///
/// If you return `Ok(WaitStatus::Waiting)` instead, this function will wait
/// some number of seconds, and then try again.
#[allow(clippy::needless_lifetimes)]
pub async fn wait<T, E, F, R>(options: &WaitOptions, mut f: F) -> Result<T, E>
where
    F: FnMut() -> R,
    R: Future<Output = WaitStatus<T, E>>,
    E: Display,
    Error: Into<E>,
{
    let deadline = options.timeout.map(|to| SystemTime::now() + to);
    let mut retry_interval = options.retry_interval;
    trace!(
        "waiting with deadline {:?}, initial interval {:?}",
        deadline,
        retry_interval
    );
    let mut errors_seen = 0;
    loop {
        // Call the function we're waiting on.
        match f().await {
            WaitStatus::Finished(value) => {
                trace!("wait finished successfully");
                return Ok(value);
            }
            WaitStatus::Waiting => trace!("waiting some more"),
            WaitStatus::FailedTemporarily(ref e)
                if errors_seen < options.allowed_errors =>
            {
                errors_seen += 1;
                error!(
                    "got error, will retry ({}/{}): {}",
                    errors_seen, options.allowed_errors, e,
                );
            }
            WaitStatus::FailedTemporarily(err) => {
                trace!("too many temporary failures, giving up on wait: {}", err);
                return Err(err);
            }
            WaitStatus::FailedPermanently(err) => {
                trace!("permanent failure, giving up on wait: {}", err);
                return Err(err);
            }
        }

        // Check to see if we'll exceed our deadline (if we have one).
        if let Some(deadline) = deadline {
            let next_attempt = SystemTime::now() + retry_interval;
            if next_attempt > deadline {
                trace!(
                    "next attempt {:?} would fall after deadline {:?}, ending wait",
                    next_attempt,
                    deadline
                );
                return Err(Error::Timeout {}.into());
            }
        }

        // Sleep until our next call.
        let duration = max(Duration::from_secs(MIN_SLEEP_SECS), retry_interval);
        sleep(duration).await;

        // Update retry interval.
        match options.backoff_type {
            BackoffType::Linear => {}
            BackoffType::Exponential => {
                retry_interval *= 2;
                trace!("next retry doubled to {:?}", retry_interval);
            }
        }
    }
}
