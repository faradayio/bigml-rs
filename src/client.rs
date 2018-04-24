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
        format!("username={}&api_key={}", self.username, self.api_key)
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
        let client = reqwest::Client::new();
        let res = client.post(url.clone())
            .json(args)
            .send()
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        self.handle_response(res)
            .map_err(|e| Error::could_not_access_url(&url, e))
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
            .map_err(|e| Error::could_not_read_file(&path, e))?;

        // Post our request.
        let url = self.url("/source");
        let client = reqwest::Client::new();
        let res = client.post(url.clone())
            .header(reqwest::header::ContentType(body.mime_type()))
            .body(body)
            .send()
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        self.handle_response(res)
            .map_err(|e| Error::could_not_access_url(&url, e))
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
            debug!("PUT {}: {:?}", &url, &body);
            let client = reqwest::Client::new();
            let res = client.request(reqwest::Method::Put, url.clone())
                .json(&body)
                .send()
                .map_err(|e| Error::could_not_access_url(&url, e))?;
            // Parse our result as JSON, because it often seems to be missing
            // fields like `name`.
            let _json: serde_json::Value = self.handle_response(res)
                .map_err(|e| Error::could_not_access_url(&url, e))?;
            Ok(())
        } else {
            Err(format_err!("No fields to update in {}", source.id()).into())
        }
    }

    /// Fetch an existing resource.
    pub fn fetch<R: Resource>(&self, resource: &Id<R>) -> Result<R> {
        let url = self.url(resource.as_str());
        let client = reqwest::Client::new();
        let res = client.get(url.clone())
            .send()
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        self.handle_response(res)
            .map_err(|e| Error::could_not_access_url(&url, e))
    }

    /// Poll an existing resource, returning it once it's ready.
    pub fn wait<R: Resource>(&self, resource: &Id<R>) -> Result<R> {
        let mut error_count: u32 = 0;
        loop {
            let res = self.fetch(resource)?;
            if res.status().code().is_ready() {
                return Ok(res);
            } else if res.status().code().is_err() {
                let err = res.status().message();
                let url = self.url(resource.as_str());
                if error_count < 5 {
                    debug!(
                        "Error fetching {}, retrying: {}",
                        url_without_api_key(&url),
                        err,
                    );
                    error_count += 1;
                } else {
                    return Err(format_err!("{}", err))
                        .map_err(|e| Error::could_not_access_url(&url, e));
                }
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
        let client = reqwest::Client::new();
        let res = client.get(url.clone())
            .send()
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        if res.status().is_success() {
            debug!("Downloading {}", &resource);
            Ok(res)
        } else {
            self.response_to_err(res)
                .map_err(|e| Error::could_not_access_url(&url, e))
        }
    }

    /// Delete the specified resource.
    pub fn delete<R: Resource>(&self, resource: &Id<R>) -> Result<()> {
        let url = self.url(resource.as_str());
        let client = reqwest::Client::new();
        let res = client.request(reqwest::Method::Delete, url.clone())
            .send()
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        if res.status().is_success() {
            debug!("Deleted {}", &resource);
            Ok(())
        } else {
            self.response_to_err(res)
                .map_err(|e| Error::could_not_access_url(&url, e))
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
        Err(Error::UnexpectedHttpStatus {
            status: res.status().to_owned(),
            body,
        })
    }
}

#[test]
fn client_url_is_sanitizable() {
    let client = Client::new("example", "secret").unwrap();
    let err: Error = Error::could_not_access_url(
        &client.url("/test"),
        format_err!("Details"),
    );
    let err_str = format!("{}", err);
    println!("err_str = {:?}", err_str);
    assert!(!err_str.contains("secret"));
}
