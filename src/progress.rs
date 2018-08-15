//! Code used for reporting execution progress.

use errors::*;

/// A callback which we be callled every time we have a new `T` value.
pub type ProgressCallback<'a, T> = FnMut(&T) -> Result<()> + 'a;

/// Options specifying how to report progress.
pub struct ProgressOptions<'a, T: 'static> {
    /// Our callback value. Only accessible from inside this crate.
    pub(crate) callback: Option<&'a mut ProgressCallback<'a, T>>,
}

impl<'a, T: 'static> ProgressOptions<'a, T> {
    /// Specify a callback to be called whenever we see a new `T` value.
    pub fn callback(mut self, callback: &'a mut ProgressCallback<'a, T>) -> Self {
        self.callback = Some(callback);
        self
    }
}

impl<'a, T: 'static> Default for ProgressOptions<'a, T> {
    fn default() -> Self {
        ProgressOptions { callback: None, }
    }
}
