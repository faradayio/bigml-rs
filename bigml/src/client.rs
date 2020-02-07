//! A client connection to BigML.

use bytes::Bytes;
use failure::Fail;
use futures::{prelude::*, FutureExt};
use reqwest::{self, multipart, StatusCode};
use serde::de::DeserializeOwned;
use serde_json;
use std::error;
use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::fs;
use tokio_util::codec;
use url::Url;

use crate::errors::*;
use crate::progress::ProgressOptions;
use crate::resource::{self, Id, Resource, Source, Updatable};
use crate::wait::{wait, WaitOptions, WaitStatus};

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
    where
        S1: Into<String>,
        S2: Into<String>,
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

    /// Generate an authenticated URL with the specified path.
    fn url(&self, path: &str) -> Url {
        let mut url: Url = BIGML_URL.clone();
        url.set_path(path);
        url.set_query(Some(&self.auth()));
        url
    }

    /// Create a new resource.
    pub async fn create<'a, Args>(&'a self, args: &'a Args) -> Result<Args::Resource>
    where
        Args: resource::Args,
    {
        let url = self.url(Args::Resource::create_path());
        debug!(
            "POST {} {:#?}",
            Args::Resource::create_path(),
            &serde_json::to_string(args)
        );
        let client = reqwest::Client::new();
        let res = client
            .post(url.clone())
            .json(args)
            .send()
            .await
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        self.handle_response_and_deserialize(&url, res).await
    }

    /// Create a new resource, and wait until it is ready.
    pub async fn create_and_wait<'a, Args>(
        &'a self,
        args: &'a Args,
    ) -> Result<Args::Resource>
    where
        Args: resource::Args,
    {
        let resource = self.create(args).await?;
        self.wait(resource.id()).await
    }

    /// Create a BigML data source using data from the specified stream.  We
    /// stream the data over the network without trying to load it all into
    /// memory at once.
    #[deprecated = "This won't work until BigML fixes Transfer-Encoding: chunked"]
    pub async fn create_source_from_stream<S>(
        &self,
        filename: &str,
        stream: S,
    ) -> Result<Source>
    where
        S: TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn error::Error + Send + Sync>>,
        Bytes: From<S::Ok>,
    {
        debug!("uploading {} from stream", filename);

        let data = multipart::Part::stream(reqwest::Body::wrap_stream(stream))
            .mime_str("application/octet-stream")?;
        let form = multipart::Form::new().part("file", data);

        // Post our request.
        let url = self.url("/source");
        let client = reqwest::Client::new();
        let res = client
            .post(url.clone())
            .multipart(form)
            .send()
            .await
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        self.handle_response_and_deserialize(&url, res).await
    }

    /// Create a BigML data source using data from the specified path.  We
    /// stream the data over the network without trying to load it all into
    /// memory at once.
    #[allow(clippy::needless_lifetimes, deprecated)]
    #[deprecated = "This won't work until BigML fixes Transfer-Encoding: chunked"]
    pub async fn create_source_from_path(&self, path: PathBuf) -> Result<Source> {
        // Convert our path to a stream of `Bytes`.
        let file = fs::File::open(&path)
            .await
            .map_err(|err| Error::could_not_read_file(&path, err))?;
        let err_path = path.clone();
        let stream = codec::FramedRead::new(file, codec::BytesCodec::new())
            .map_ok(|bytes| bytes.freeze())
            .map_err(move |err| Error::could_not_read_file(&err_path, err).compat());

        // Create our source.
        let filename = path.to_string_lossy();
        self.create_source_from_stream(&filename, stream).await
    }

    /// Create a BigML data source using data from the specified path.  We
    /// stream the data over the network without trying to load it all into
    /// memory.
    #[allow(clippy::needless_lifetimes, deprecated)]
    #[deprecated = "This won't work until BigML fixes Transfer-Encoding: chunked"]
    pub async fn create_source_from_path_and_wait(
        &self,
        path: PathBuf,
    ) -> Result<Source> {
        let source = self.create_source_from_path(path).await?;
        // Only wait 2 hours for a source to be created
        let options = WaitOptions::default().timeout(Duration::from_secs(2 * 60 * 60));
        let mut progress_options = ProgressOptions::default();
        self.wait_opt(source.id(), &options, &mut progress_options)
            .await
    }

    /// Update the specified `resource` using `update`. We do not return the
    /// updated resource because of peculiarities with BigML's API, but you
    /// can always use `Client::fetch` if you need the updated version.
    pub async fn update<'a, R: Resource + Updatable>(
        &'a self,
        resource: &'a Id<R>,
        update: &'a <R as Updatable>::Update,
    ) -> Result<()> {
        let url = self.url(resource.as_str());
        debug!("PUT {}: {:?}", url, update);
        let client = reqwest::Client::new();
        let res = client
            .request(reqwest::Method::PUT, url.clone())
            .json(update)
            .send()
            .await
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        // Parse our result as JSON, because it often seems to be missing
        // fields like `name` for `Source`. It's not always a complete,
        // valid resource.
        let _json: serde_json::Value =
            self.handle_response_and_deserialize(&url, res).await?;

        Ok(())
    }

    /// Fetch an existing resource.
    pub async fn fetch<'a, R: Resource>(&'a self, resource: &'a Id<R>) -> Result<R> {
        let url = self.url(resource.as_str());
        let client = reqwest::Client::new();
        let res = client
            .get(url.clone())
            .send()
            .await
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        self.handle_response_and_deserialize(&url, res).await
    }

    /// Poll an existing resource, returning it once it's ready.
    pub async fn wait<'a, R: Resource>(&'a self, resource: &'a Id<R>) -> Result<R> {
        let options = WaitOptions::default();
        let mut progress_options = ProgressOptions::default();
        self.wait_opt(resource, &options, &mut progress_options)
            .await
    }

    /// Poll an existing resource, returning it once it's ready, and honoring
    /// wait and progress options.
    pub async fn wait_opt<'a, 'b, R: Resource>(
        &self,
        resource: &'a Id<R>,
        wait_options: &'a WaitOptions,
        progress_options: &'a mut ProgressOptions<'b, R>,
    ) -> Result<R> {
        let url = self.url(resource.as_str());
        debug!("Waiting for {}", url_without_api_key(&url));

        // We actually want to pass an `aync || { ... }` to `wait`, below, but
        // async closures are going to stablize later than the rest of
        // `async_await`. So we need to use `|| { async { ... } }`, which is a
        // regular closure that returns a future. Except that this doesn't
        // _quite_ work, because the `async { ... }` would contain a mutable
        // reference `progress_options`, which can't be allowed to escape the
        // outer `|| { ... }` block. So we cheat, and wrap our mutable state in
        // a lock. When `async || { ... }` stablizes, we can just delete this
        // line.
        let progress_options = Arc::new(RwLock::new(progress_options));

        wait(&wait_options, || {
            let progress_options = progress_options.clone();
            async move {
                // TODO: Consider replacing `try_with_temporary_failure!`
                // and `try_with_permanent_failure!` with `try_wait!` and
                // appropriate error wrapping.
                let res = try_with_temporary_failure!(self.fetch(resource).await);
                if let Some(ref mut callback) =
                    progress_options.write().unwrap().callback
                {
                    try_with_permanent_failure!(callback(&res));
                }
                if res.status().code().is_ready() {
                    WaitStatus::Finished(res)
                } else if res.status().code().is_err() {
                    let message = res.status().message();
                    let err = Error::WaitFailed {
                        id: resource.to_string(),
                        message: message.to_owned(),
                    };
                    // I think we always want to fail for good here? We may need to
                    // tweak this.
                    WaitStatus::FailedPermanently(err)
                } else {
                    WaitStatus::Waiting
                }
            }
            .boxed()
        })
        .await
        .map_err(|e| Error::could_not_access_url(&url, e))
    }

    /// Download a resource as a CSV file.  This only makes sense for
    /// certain kinds of resources.
    pub async fn download<'a, R: Resource>(
        &'a self,
        resource: &'a Id<R>,
    ) -> Result<reqwest::Response> {
        // This timeout needs to be set fairly high, because when we first try
        // to download a dataset, even one which has been `wait`ed on, we get
        // back a JSON message informing us that the dataset isn't ready for
        // download yet. We've definitely seen this process take longer than 3
        // minutes, so let's try this.
        let options = WaitOptions::default().timeout(Duration::from_secs(10 * 60));
        self.download_opt(resource, &options).await
    }

    /// Download a resource as a CSV file.  This only makes sense for
    /// certain kinds of resources.
    pub async fn download_opt<'a, R: Resource>(
        &'a self,
        resource: &'a Id<R>,
        options: &'a WaitOptions,
    ) -> Result<reqwest::Response> {
        let url = self.url(&format!("{}/download", &resource));
        debug!("Downloading {}", url_without_api_key(&url));
        let client = reqwest::Client::new();
        wait(
            &options,
            || -> Pin<Box<dyn Future<Output = WaitStatus<_, Error>> + Send>> {
                async {
                    // TODO: Consider replacing `try_with_temporary_failure!`
                    // and `try_with_permanent_failure!` with `try_wait!` and
                    // appropriate error wrapping.
                    let res = try_with_temporary_failure!(
                        client.get(url.clone()).send().await
                    );
                    if res.status().is_success() {
                        // Sometimes "/download" returns JSON instead of CSV, which
                        // is generally a sign that we need to wait.
                        let headers = res.headers().to_owned();
                        if let Some(ct) = headers.get("Content-Type") {
                            if ct.as_bytes().starts_with(b"application/json") {
                                let body =
                                    try_with_temporary_failure!(res.text().await);
                                debug!("Got JSON when downloading CSV: {}", body);
                                return WaitStatus::Waiting;
                            }
                        }
                        WaitStatus::Finished(res)
                    } else {
                        try_with_temporary_failure!(
                            self.response_to_err(&url, res).await
                        );
                        // The above always returns `Err` and bails out, so we can't get
                        // here.
                        unreachable!()
                    }
                }
                .boxed()
            },
        )
        .await
        .map_err(|e| Error::could_not_access_url(&url, e))
    }

    /// Delete the specified resource.
    pub async fn delete<'a, R: Resource>(&'a self, resource: &'a Id<R>) -> Result<()> {
        let url = self.url(resource.as_str());
        let client = reqwest::Client::new();
        let res = client
            .request(reqwest::Method::DELETE, url.clone())
            .send()
            .await
            .map_err(|e| Error::could_not_access_url(&url, e))?;
        if res.status().is_success() {
            debug!("Deleted {}", &resource);
            Ok(())
        } else {
            self.response_to_err(&url, res).await
        }
    }

    /// Handle a response from the server, deserializing it as the
    /// appropriate type.
    async fn handle_response_and_deserialize<'a, T>(
        &'a self,
        url: &'a Url,
        res: reqwest::Response,
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        if res.status().is_success() {
            let body = res
                .text()
                .await
                .map_err(|e| Error::could_not_access_url(&url, e))?;
            debug!("Success body: {}", &body);
            let properties = serde_json::from_str(&body)
                .map_err(|e| Error::could_not_access_url(&url, e))?;
            Ok(properties)
        } else {
            self.response_to_err(url, res).await
        }
    }

    async fn response_to_err<'a, T>(
        &'a self,
        url: &'a Url,
        res: reqwest::Response,
    ) -> Result<T> {
        let url = url.to_owned();
        let status: StatusCode = res.status().to_owned();
        let body = res.text().await?;
        debug!("Error status: {} body: {}", status, body);
        match status {
            StatusCode::PAYMENT_REQUIRED => Err(Error::PaymentRequired { url, body }),
            _ => Err(Error::UnexpectedHttpStatus { url, status, body }),
        }
    }
}

#[test]
fn client_url_is_sanitizable() {
    let client = Client::new("example", "secret").unwrap();
    let err: Error =
        Error::could_not_access_url(&client.url("/test"), format_err!("Details"));
    let err_str = format!("{}", err);
    println!("err_str = {:?}", err_str);
    assert!(!err_str.contains("secret"));
}
