//! A [`tokio::codec::Encoder`] that outputs a line-delimited JSON stream.

use bytes::{BufMut, BytesMut};
use failure::Error;
use serde::Serialize;
use std::marker::PhantomData;
use tokio_util::codec::Encoder;

/// A [`tokio::codec::Encoder`] that outputs a [line-delimited JSON
/// stream][json]. This can be used to output a `Stream` of values implementing
/// `Serialize` to an `AsyncWrite`.
///
/// [json]: https://en.wikipedia.org/wiki/JSON_streaming
pub struct LineDelimitedJsonCodec<T: Serialize> {
    _placeholder: PhantomData<T>,
}

impl<T: Serialize> LineDelimitedJsonCodec<T> {
    /// Create a new `LineDelimitedJsonCodec`.
    pub fn new() -> Self {
        Self {
            _placeholder: PhantomData,
        }
    }
}

impl<T: Serialize> Encoder<T> for LineDelimitedJsonCodec<T> {
    type Error = Error;

    fn encode(&mut self, item: T, buf: &mut BytesMut) -> Result<(), Error> {
        let json = serde_json::to_vec(&item)?;
        buf.reserve(json.len() + 1);
        buf.put(&json[..]);
        buf.put_u8(b'\n');
        Ok(())
    }
}
