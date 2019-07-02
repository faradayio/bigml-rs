//! Support for sending multipart form data with a file attachment.

use mime;
use reqwest;
use std::fs;
use std::io::{self, Read};
use std::path::PathBuf;
use uuid::Uuid;

use crate::errors::*;

/// A `multipart/form-data` body containing exactly one file.  We can
/// generalize this latter if we need to, but maybe upstream will be fixed
/// by then.
pub struct Body {
    boundary: String,
    size: u64,
    reader: Box<dyn Read + Send>,
}

impl Body {
    /// Create a new multipart body.
    pub fn new<S, P>(name: S, path: P) -> Result<Body>
    where
        S: Into<String>,
        P: Into<PathBuf>,
    {
        // Convert our parameters.
        let name = name.into();
        let path = path.into();
        let filename = path.to_string_lossy();

        // Open up our file.
        let file =
            fs::File::open(&path).map_err(|e| Error::could_not_read_file(&path, e))?;
        let file_size = file.metadata()?.len();

        // Create a streaming, multi-part encoder.  Don't even think of
        // reading all the data into memory; there may be 10s of gigabytes
        // for some applications.
        //
        // TODO: Escape filename.
        let boundary = format!("--------------------------{}", Uuid::new_v4());
        let header = format!(
            "--{}\r
Content-Disposition: form-data; name=\"{}\"; filename=\"{}\"\r
Content-Type: application/octet-stream\r
\r
",
            &boundary, &name, filename
        );
        let footer = format!(
            "\r
--{}--\r
",
            &boundary
        );
        let size = header.len() as u64 + file_size + footer.len() as u64;
        let body = io::Cursor::new(header)
            .chain(file)
            .chain(io::Cursor::new(footer));
        Ok(Body {
            boundary,
            size,
            reader: Box::new(body),
        })
    }

    /// The MIME type for this body, including the `boundary` value.
    pub fn mime_type(&self) -> mime::Mime {
        format!("multipart/form-data; boundary={}", self.boundary)
            .parse()
            .expect("Could not parse built-in MIME type")
    }
}

impl Read for Body {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.reader.read(buf)
    }
}

impl From<Body> for reqwest::Body {
    fn from(body: Body) -> reqwest::Body {
        let size = body.size;
        reqwest::Body::sized(body, size)
    }
}
