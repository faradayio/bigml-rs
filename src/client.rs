//! A client connection to BigML.

use mime;
use multipart::client::lazy::Multipart;
use reqwest;
use std::fs;
use std::io::{self, Read};
use std::path::Path;
use url::Url;

use errors::*;

lazy_static! {
    /// The URL of the BigML API.
    static ref BIGML_URL: Url = Url::parse("https://bigml.io/")
        .expect("Cannot parse BigML URL in source code");
}

/// A client connection to BigML.
pub struct Client {
    username: String,
    api_key: String,
}

impl Client {
    /// Create a new `Client`.
    pub fn new<S1, S2>(username: S1, api_key: S2) -> Result<Client>
        where S1: Into<String>, S2: Into<String>
    {
        Ok(Client {
            username: username.into(),
            api_key: api_key.into(),
        })
    }

    /// Format our BigML auth credentials.
    fn auth(&self) -> String {
        format!("username={};api_key={}", self.username, self.api_key)
    }

    /// Generate an authenticate URL with the specified path.
    fn url(&self, path: &str) -> Result<Url> {
        let mut url: Url = BIGML_URL.clone();
        url.set_path(path);
        url.set_query(Some(&self.auth()));
        Ok(url)
    }

    /// Create a BigML data source using data from the specified path.  We
    /// stream the data over the network without trying to load it all into
    /// memory.
    pub fn source_create_from_path<P>(&self, path: P)
                                      -> Result<String>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let file = fs::File::open(&path)
            .chain_err(|| ErrorKind::CouldNotReadFile(path.to_owned()))?;
        self.source_create_from_reader(&path.to_string_lossy(), file)
    }

    /// Create a BigML data source using data from a reader.  We stream the
    /// data over the network without trying to load it all into memory.
    pub fn source_create_from_reader<R>(&self, filename: &str, reader: R)
                                        -> Result<String>
        where R: Read + 'static
    {
        // Create a streaming, multi-part encoder.  Don't even think of
        // reading all the data into memory; there may be 10s of gigabytes
        // for some applications.
        let mut multipart: Multipart<'static, 'static> = Multipart::new();
        multipart.add_stream("file", reader, Some(filename.to_owned()), None);
        //multipart.add_file("file", Path::new("iris.csv"));
        let mut encoded_reader = multipart.prepare()
            .chain_err(|| {
                ErrorKind::CouldNotReadFile(Path::new(filename).to_owned())
            })?;

        //println!("BEGIN OUTPUT");
        //io::copy(&mut encoded_reader, &mut io::stdout()).unwrap();
        //println!("END OUTPUT");

        // Generate an appropriate Content-Type header.
        let boundary = encoded_reader.boundary().into();
        let mime = mime::Mime(mime::TopLevel::Multipart,
                              mime::SubLevel::FormData,
                              vec![(mime::Attr::Boundary,
                                    mime::Value::Ext(boundary))]);

        // Post our request.
        let url = self.url("/source")?;
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            // TODO: https://github.com/seanmonstar/reqwest/issues/17
            .map_err(|e| { let kind: Error = format!("{}", e).into(); kind })
            .chain_err(&mkerr)?;
        let mut res = client.post(url.clone())
            .header(reqwest::header::ContentType(mime))
            .body(reqwest::Body::new(encoded_reader))
            .send()
            .map_err(|e| { let kind: Error = format!("{}", e).into(); kind })
            .chain_err(&mkerr)?;

        if res.status().is_success() {
            let mut body = String::new();
            res.read_to_string(&mut body).chain_err(&mkerr)?;
            Ok(body)
        } else {
            let err: Error = ErrorKind::UnexpectedHttpStatus(res.status().to_owned())
                .into();
            Err(err).chain_err(&mkerr)
        }
    }
}
