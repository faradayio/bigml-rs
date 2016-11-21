//! A client connection to BigML.

use reqwest;
use serde::Deserialize;
use serde_json;
use std::io::Read;
use std::path::Path;
use url::Url;

use errors::*;
use multipart_form_data;
use serde_types::SourceProperties;
use util::StringifyError;

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
    pub fn source_create_from_path<P>(&self, path: P) -> Result<SourceProperties>
        where P: AsRef<Path>
    {
        let path = path.as_ref();
        let mut body = multipart_form_data::Body::new("file", path)
            .chain_err(|| ErrorKind::CouldNotReadFile(path.to_owned()))?;

        // TODO: File upstream. Work around the facts that:
        //
        // 1. `reqwest` can't take a `Read` impl and a size as imput, and so
        //    it falls back to `chunked` mode if given a reader, and
        // 2. BigML can't handle chunked transfer encoding,
        //
        // ...by reading everything into memory.
        let mut body_data = vec![];
        body.read_to_end(&mut body_data)
            .chain_err(|| ErrorKind::CouldNotReadFile(path.to_owned()))?;

        // Post our request.
        let url = self.url("/source")?;
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .stringify_error()
            .chain_err(&mkerr)?;
        let res = client.post(url.clone())
            .header(reqwest::header::ContentType(body.mime_type()))
            .body(body_data)
            .send()
            .stringify_error()
            .chain_err(&mkerr)?;
        self.handle_response(res).chain_err(&mkerr)
    }

    /// Handle a response from the server, deserializing it as the
    /// appropriate type.
    fn handle_response<T>(&self, mut res: reqwest::Response) -> Result<T>
        where T: Deserialize
    {
        if res.status().is_success() {
            let mut body = String::new();
            res.read_to_string(&mut body)?;
            let properties = serde_json::from_str(&body)?;
            Ok(properties)
        } else {
            let mut body = String::new();
            res.read_to_string(&mut body)?;
            let err: Error =
                ErrorKind::UnexpectedHttpStatus(res.status().to_owned()).into();
            Err(err)
        }

    }
}
