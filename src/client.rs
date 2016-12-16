//! A client connection to BigML.

use reqwest;
use serde::Deserialize;
use serde_json;
use std::io::Read;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

use errors::*;
use multipart_form_data;
use resource::{self, Id, Resource, Source};
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
    fn url(&self, path: &str) -> Url {
        let mut url: Url = BIGML_URL.clone();
        url.set_path(path);
        url.set_query(Some(&self.auth()));
        url
    }

    /// Create a new resource.
    pub fn create<Args>(&self, args: &Args) -> Result<Args::Resource>
        where Args: resource::Args
    {
        let url = self.url(Args::Resource::create_path());
        debug!("POST {} {:#?}", Args::Resource::create_path(), &serde_json::to_string(args));
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .stringify_error()
            .chain_err(&mkerr)?;
        let res = client.post(url.clone())
            .json(args)
            .send()
            .stringify_error()
            .chain_err(&mkerr)?;
        self.handle_response(res).chain_err(&mkerr)
    }

    /// Create a new resource, and wait until it is ready.
    pub fn create_and_wait<Args>(&self, args: &Args) -> Result<Args::Resource>
        where Args: resource::Args
    {
        self.wait(self.create(args)?.id())
    }

    /// Create a BigML data source using data from the specified path.  We
    /// stream the data over the network without trying to load it all into
    /// memory.
    pub fn create_source_from_path<P>(&self, path: P) -> Result<Source>
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
        let url = self.url("/source");
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

    /// Fetch an existing resource.
    pub fn fetch<R: Resource>(&self, resource: &Id<R>) -> Result<R> {
        let url = self.url(resource.as_str());
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .stringify_error()
            .chain_err(&mkerr)?;
        let res = client.get(url.clone())
            .send()
            .stringify_error()
            .chain_err(&mkerr)?;
        self.handle_response(res).chain_err(&mkerr)
    }

    /// Poll an existing resource, returning it once it's ready.
    pub fn wait<R: Resource>(&self, resource: &Id<R>) -> Result<R> {
        loop {
            let res = self.fetch(resource)?;
            if res.status().code().is_ready() {
                return Ok(res);
            } else if res.status().code().is_err() {
                // TODO: We should probably allow a few errors before
                // giving up, and we should probably have some sort of
                // timeout.
                let err: Error = res.status().message().into();
                let url = self.url(resource.as_str());
                return Err(err)
                    .chain_err(|| ErrorKind::CouldNotAccessUrl(url.clone()));
            }

            // If we're not ready, then sleep 10 seconds.  Anything less
            // than 4 may get us rate-limited or banned according to BigML
            // support.
            sleep(Duration::from_secs(10));
        }
    }

    /// Download a resource as a CSV file.  This only makes sense for
    /// certain kinds of resources.
    pub fn download<R: Resource>(&self, resource: &Id<R>)
                                 -> Result<reqwest::Response> {
        let url = self.url(&format!("{}/download", &resource));
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .stringify_error()
            .chain_err(&mkerr)?;
        let res = client.get(url.clone())
            .send()
            .stringify_error()
            .chain_err(&mkerr)?;
        if res.status().is_success() {
            debug!("Downloading {}", &resource);
            Ok(res)
        } else {
            self.response_to_err(res).chain_err(&mkerr)
        }
    }

    /// Delete the specified resource.
    pub fn delete<R: Resource>(&self, resource: &Id<R>) -> Result<()> {
        let url = self.url(resource.as_str());
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .stringify_error()
            .chain_err(&mkerr)?;
        let res = client.request(reqwest::Method::Delete, url.clone())
            .send()
            .stringify_error()
            .chain_err(&mkerr)?;
        if res.status().is_success() {
            debug!("Deleted {}", &resource);
            Ok(())
        } else {
            self.response_to_err(res).chain_err(&mkerr)
        }
    }

    /// Handle a response from the server, deserializing it as the
    /// appropriate type.
    fn handle_response<T>(&self, mut res: reqwest::Response) -> Result<T>
        where T: Deserialize
    {
        if res.status().is_success() {
            let mut body = String::new();
            res.read_to_string(&mut body)?;
            debug!("Success body: {}", &body);
            let properties = serde_json::from_str(&body)?;
            Ok(properties)
        } else {
            self.response_to_err(res)
        }
    }

    fn response_to_err<T>(&self, mut res: reqwest::Response) -> Result<T> {
        let mut body = String::new();
        res.read_to_string(&mut body)?;
        debug!("Error body: {}", &body);
        Err(ErrorKind::UnexpectedHttpStatus(res.status().to_owned(), body).into())
    }
}
