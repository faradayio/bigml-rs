//! A client connection to BigML.

use reqwest;
use serde::de::DeserializeOwned;
use serde_json;
use std::collections::HashMap;
use std::io::Read;
use std::iter::FromIterator;
use std::path::Path;
use std::thread::sleep;
use std::time::Duration;
use url::Url;

use errors::*;
use multipart_form_data;
use resource::{self, Id, Resource, Source, source};

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
            .chain_err(&mkerr)?;
        let res = client.post(url.clone())
            .chain_err(&mkerr)?
            .json(args)
            .chain_err(&mkerr)?
            .send()
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
        let body = multipart_form_data::Body::new("file", path)
            .chain_err(|| ErrorKind::CouldNotReadFile(path.to_owned()))?;

        // Post our request.
        let url = self.url("/source");
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .chain_err(&mkerr)?;
        let res = client.post(url.clone())
            .chain_err(&mkerr)?
            .header(reqwest::header::ContentType(body.mime_type()))
            .body(body)
            .send()
            .chain_err(&mkerr)?;
        self.handle_response(res).chain_err(&mkerr)
    }

    /// Create a BigML data source using data from the specified path.  We
    /// stream the data over the network without trying to load it all into
    /// memory.
    pub fn create_source_from_path_and_wait<P>(&self, path: P) -> Result<Source>
        where P: AsRef<Path>
    {
        let source = self.create_source_from_path(path)?;
        self.wait(source.id())
    }

    /// When a `source` is initially created, a few of the field types may
    /// have been guessed incorrectly, which means that we need to update
    /// them (which is rare in the BigML API).  To update the field types,
    /// update them in the `source` object and then pass it to this
    /// function.
    ///
    /// TODO: This is a terrible API and it will go away sometime soon.  We
    /// need a much more general solution for this, or perhaps a nice,
    /// special-purpose API with good ergonomics.
    #[doc(hidden)]
    pub fn update_source_fields(&self, source: &Source) -> Result<()> {
        if let Some(ref fields) = source.fields {
            #[derive(Debug, Serialize)]
            struct FieldDiff {
                optype: source::Optype,
            }

            #[derive(Debug, Serialize)]
            struct SourceUpdate {
                fields: HashMap<String, FieldDiff>,
            }

            let body = SourceUpdate {
                fields: HashMap::from_iter(fields.iter().map(|(id, field)| {
                    (id.clone(), FieldDiff { optype: field.optype })
                }))
            };

            let url = self.url(source.id().as_str());
            let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
            debug!("PUT {}: {:?}", &url, &body);
            let client = reqwest::Client::new()
                .chain_err(&mkerr)?;
            let res = client.request(reqwest::Method::Put, url.clone())
                .chain_err(&mkerr)?
                .json(&body)
                .chain_err(&mkerr)?
                .send()
                .chain_err(&mkerr)?;
            // Parse our result as JSON, because it often seems to be missing
            // fields like `name`.
            let _json: serde_json::Value =
                self.handle_response(res).chain_err(&mkerr)?;
            Ok(())
        } else {
            Err(format!("No fields to update in {}", source.id()).into())
        }
    }

    /// Fetch an existing resource.
    pub fn fetch<R: Resource>(&self, resource: &Id<R>) -> Result<R> {
        let url = self.url(resource.as_str());
        let mkerr = || ErrorKind::CouldNotAccessUrl(url.clone());
        let client = reqwest::Client::new()
            .chain_err(&mkerr)?;
        let res = client.get(url.clone())
            .chain_err(&mkerr)?
            .send()
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
            .chain_err(&mkerr)?;
        let res = client.get(url.clone())
            .chain_err(&mkerr)?
            .send()
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
            .chain_err(&mkerr)?;
        let res = client.request(reqwest::Method::Delete, url.clone())
            .chain_err(&mkerr)?
            .send()
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
        where T: DeserializeOwned
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
