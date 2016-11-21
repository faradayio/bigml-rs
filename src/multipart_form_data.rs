//! Support for sending multipart form data with a file attachment.

use mime;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use uuid::Uuid;

use errors::*;

/// A `multipart/form-data` body containing exactly one file.  We can
/// generalize this latter if we need to, but maybe upstream will be fixed
/// by then.
pub struct Body {
    boundary: String,
    reader: Box<Read>,
}

impl Body {
    /// Create a new multipart body.
    pub fn new<S, P>(name: S, path: P) -> Result<Body>
        where S: Into<String>, P: Into<PathBuf>
    {
        // Convert our parameters.
        let name = name.into();
        let path = path.into();
        let filename = path.to_string_lossy();

        // Open up our file.
        let file = fs::File::open(&path)
            .chain_err(|| ErrorKind::CouldNotReadFile(path.clone()))?;

        // Create a streaming, multi-part encoder.  Don't even think of
        // reading all the data into memory; there may be 10s of gigabytes
        // for some applications.
        //
        // TODO: Escape filename.
        let boundary = format!("--------------------------{}", Uuid::new_v4());
        let header = format!("--{}\r
Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r
Content-Type: application/octet-stream\r
\r
", &boundary, &name, filename);
        let footer = format!("\r
--{}--\r
", &boundary);
        let body = io::Cursor::new(header)
            .chain(file)
            .chain(io::Cursor::new(footer));
        Ok(Body {
            boundary: boundary,
            reader: Box::new(body)
        })
    }

    /// The MIME type for this body, including the `boundary` value.
    pub fn mime_type(&self) -> mime::Mime {
        mime::Mime(mime::TopLevel::Multipart,
                   mime::SubLevel::FormData,
                   vec![(mime::Attr::Boundary,
                         mime::Value::Ext(self.boundary.to_owned()))])
    }
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}
