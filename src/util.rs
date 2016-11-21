//! Miscellaneous utilities.

use std::error;
use std::result;

use errors::*;

/// TODO: Force various errors to be representable as our custom `Result`
/// by first turning them into strings.  See
/// https://github.com/seanmonstar/reqwest/issues/17
pub trait StringifyError<T, E: error::Error> {
    /// Convert to our `Result` type.
    fn stringify_error(self) -> Result<T>;
}

impl<T, E: error::Error> StringifyError<T, E> for result::Result<T, E> {
    fn stringify_error(self) -> Result<T> {
        self.map_err(|e| {
            let kind: Error = format!("{}", e).into();
            kind
        })
    }
}
