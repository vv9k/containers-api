//! Transports for communicating with the Podman or Docker daemon

use crate::conn::{Error, Result};

use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::{self, Stream},
    StreamExt, TryFutureExt,
};
use hyper::{
    body::Bytes,
    client::{Client, HttpConnector},
    header, Body, Method, Request, Response, StatusCode,
};
#[cfg(feature = "tls")]
use hyper_openssl::HttpsConnector;
#[cfg(unix)]
use hyperlocal::UnixConnector;
#[cfg(unix)]
use hyperlocal::Uri as DomainUri;
use pin_project::pin_project;

use serde::{Deserialize, Serialize};
use url::Url;

use std::{
    io,
    iter::IntoIterator,
    path::PathBuf,
    pin::Pin,
    task::{Context, Poll},
};

#[derive(Debug, Default, Clone)]
/// Helper structure used as a container for HTTP headers passed to a request
pub struct Headers(Vec<(&'static str, String)>);

impl Headers {
    /// Shortcut for when one does not want headers in a request
    pub fn none() -> Option<Headers> {
        None
    }

    /// Adds a single key=value header pair
    pub fn add<V>(&mut self, key: &'static str, val: V)
    where
        V: Into<String>,
    {
        self.0.push((key, val.into()))
    }

    /// Constructs an instance of Headers with initial pair, usually used when there is only
    /// a need for one header.
    pub fn single<V>(key: &'static str, val: V) -> Self
    where
        V: Into<String>,
    {
        let mut h = Self::default();
        h.add(key, val);
        h
    }
}

impl IntoIterator for Headers {
    type Item = (&'static str, String);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

/// Types of payload that can be sent
pub enum Payload<B: Into<Body>> {
    None,
    Text(B),
    Json(B),
    XTar(B),
    Tar(B),
}

impl Payload<Body> {
    /// Creates an empty payload
    pub fn empty() -> Self {
        Payload::None
    }
}

impl<B: Into<Body>> Payload<B> {
    /// Extracts the inner body if there is one and returns it
    pub fn into_inner(self) -> Option<B> {
        match self {
            Self::None => None,
            Self::Text(b) => Some(b),
            Self::Json(b) => Some(b),
            Self::XTar(b) => Some(b),
            Self::Tar(b) => Some(b),
        }
    }

    /// Returns the mime type of this payload
    pub fn mime_type(&self) -> Option<mime::Mime> {
        match &self {
            Self::None => None,
            Self::Text(_) => None,
            Self::Json(_) => Some(mime::APPLICATION_JSON),
            Self::XTar(_) => Some("application/x-tar".parse().expect("parsed mime")),
            Self::Tar(_) => Some("application/tar".parse().expect("parsed mime")),
        }
    }

    /// Checks if there is no payload
    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }
}

pub async fn body_to_string(body: Body) -> Result<String> {
    let bytes = hyper::body::to_bytes(body).await?;
    String::from_utf8(bytes.to_vec()).map_err(Error::from)
}

/// Transports are types which define supported means of communication.
#[derive(Clone, Debug)]
pub enum Transport {
    /// A network tcp interface
    Tcp {
        client: Client<HttpConnector>,
        host: Url,
    },
    /// TCP/TLS
    #[cfg(feature = "tls")]
    #[cfg_attr(docsrs, doc(cfg(feature = "tls")))]
    EncryptedTcp {
        client: Client<HttpsConnector<HttpConnector>>,
        host: Url,
    },
    /// A Unix domain socket
    #[cfg(unix)]
    Unix {
        client: Client<UnixConnector>,
        path: PathBuf,
    },
}

impl Transport {
    pub fn remote_addr(&self) -> &str {
        match &self {
            Self::Tcp { ref host, .. } => host.as_ref(),
            #[cfg(feature = "tls")]
            Self::EncryptedTcp { ref host, .. } => host.as_ref(),
            #[cfg(unix)]
            Self::Unix { ref path, .. } => path.to_str().unwrap_or_default(),
        }
    }

    pub fn make_uri(&self, ep: &str) -> Result<hyper::Uri> {
        match self {
            Transport::Tcp { host, .. } => {
                format!("{}{}", host, ep).parse().map_err(Error::InvalidUri)
            }
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { host, .. } => {
                format!("{}{}", host, ep).parse().map_err(Error::InvalidUri)
            }
            #[cfg(unix)]
            Transport::Unix { path, .. } => Ok(DomainUri::new(&path, ep).into()),
        }
    }

    pub async fn request(&self, req: Result<Request<Body>>) -> Result<Response<Body>> {
        self.send_request(req?).await
    }

    pub async fn request_string(&self, req: Result<Request<Body>>) -> Result<String> {
        let body = self.get_body_request(req).await?;
        body_to_string(body).await
    }

    pub fn stream_chunks(
        &self,
        req: Result<Request<Body>>,
    ) -> impl Stream<Item = Result<Bytes>> + '_ {
        self.get_chunk_stream(req).try_flatten_stream()
    }

    pub fn stream_json_chunks(
        &self,
        req: Result<Request<Body>>,
    ) -> impl Stream<Item = Result<Bytes>> + '_ {
        self.get_json_chunk_stream(req).try_flatten_stream()
    }

    pub async fn stream_upgrade<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
    ) -> Result<impl AsyncRead + AsyncWrite>
    where
        B: Into<Body>,
    {
        self.stream_upgrade_tokio(method, endpoint.as_ref(), body)
            .await
            .map(Compat::new)
            .map_err(Error::from)
    }

    pub async fn get_body_request(&self, req: Result<Request<Body>>) -> Result<Body> {
        let response = self.request(req).await?;
        self.get_body_response(response).await
    }

    pub async fn get_body_response(&self, response: Response<Body>) -> Result<Body> {
        log::trace!(
            "got response {} {:?}",
            response.status(),
            response.headers()
        );
        let status = response.status();
        let body = response.into_body();

        match status {
            // Success case: pass on the response
            StatusCode::OK
            | StatusCode::CREATED
            | StatusCode::SWITCHING_PROTOCOLS
            | StatusCode::NO_CONTENT => Ok(body),
            _ => {
                let bytes = hyper::body::to_bytes(body).await?;
                let message_body = String::from_utf8(bytes.to_vec())?;

                log::trace!("{message_body:#?}");
                Err(Error::Fault {
                    code: status,
                    message: Self::get_error_message(&message_body).unwrap_or_else(|| {
                        status
                            .canonical_reason()
                            .unwrap_or("unknown error code")
                            .to_owned()
                    }),
                })
            }
        }
    }

    pub async fn get_response_string(&self, response: Response<Body>) -> Result<String> {
        let body = self.get_body_response(response).await?;
        body_to_string(body).await
    }

    async fn get_chunk_stream(
        &self,
        req: Result<Request<Body>>,
    ) -> Result<impl Stream<Item = Result<Bytes>>> {
        self.get_body_request(req).await.map(stream_body)
    }

    async fn get_json_chunk_stream(
        &self,
        req: Result<Request<Body>>,
    ) -> Result<impl Stream<Item = Result<Bytes>>> {
        self.get_body_request(req).await.map(stream_json_body)
    }

    /// Send the given request and return a Future of the response.
    async fn send_request(&self, req: Request<Body>) -> Result<Response<Body>> {
        log::trace!("sending request {} {}", req.method(), req.uri());
        match self {
            Transport::Tcp { ref client, .. } => client.request(req),
            #[cfg(feature = "tls")]
            Transport::EncryptedTcp { ref client, .. } => client.request(req),
            #[cfg(unix)]
            Transport::Unix { ref client, .. } => client.request(req),
        }
        .await
        .map_err(Error::from)
    }

    /// Makes an HTTP request, upgrading the connection to a TCP
    /// stream on success.
    async fn stream_upgrade_tokio<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Payload<B>,
    ) -> Result<hyper::upgrade::Upgraded>
    where
        B: Into<Body>,
    {
        let mut headers = Headers::default();
        headers.add(header::CONNECTION.as_str(), "Upgrade");
        headers.add(header::UPGRADE.as_str(), "tcp");

        let uri = self.make_uri(endpoint)?;
        let req = build_request(method, uri, body, Some(headers))?;

        let response = self.send_request(req).await?;
        match response.status() {
            StatusCode::SWITCHING_PROTOCOLS => Ok(hyper::upgrade::on(response).await?),
            _ => Err(Error::ConnectionNotUpgraded),
        }
    }

    /// Extract the error message content from an HTTP response
    fn get_error_message(body: &str) -> Option<String> {
        serde_json::from_str::<ErrorResponse>(body)
            .map(|e| e.message)
            .ok()
    }
}

/// Builds an HTTP request.
pub(crate) fn build_request<B>(
    method: Method,
    uri: hyper::Uri,
    body: Payload<B>,
    headers: Option<Headers>,
) -> Result<Request<Body>>
where
    B: Into<Body>,
{
    let builder = hyper::http::request::Builder::new();
    let req = builder.method(method).uri(&uri);
    let mut req = req.header(header::HOST, "");

    if let Some(h) = headers {
        for (k, v) in h.into_iter() {
            req = req.header(k, v);
        }
    }

    // early return
    if body.is_none() {
        return Ok(req.body(Body::empty())?);
    }

    let mime = body.mime_type();
    if let Some(c) = mime {
        req = req.header(header::CONTENT_TYPE, &c.to_string());
    }

    // it's ok to unwrap, we check that the body is not none
    req.body(body.into_inner().unwrap().into())
        .map_err(Error::from)
}

#[pin_project]
struct Compat<S> {
    #[pin]
    tokio_multiplexer: S,
}

impl<S> Compat<S> {
    fn new(tokio_multiplexer: S) -> Self {
        Self { tokio_multiplexer }
    }
}

impl<S> AsyncRead for Compat<S>
where
    S: tokio::io::AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let mut readbuf = tokio::io::ReadBuf::new(buf);
        match self.project().tokio_multiplexer.poll_read(cx, &mut readbuf) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Ok(())) => Poll::Ready(Ok(readbuf.filled().len())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e)),
        }
    }
}

impl<S> AsyncWrite for Compat<S>
where
    S: tokio::io::AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        self.project().tokio_multiplexer.poll_write(cx, buf)
    }
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_flush(cx)
    }
    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.project().tokio_multiplexer.poll_shutdown(cx)
    }
}

#[derive(Serialize, Deserialize)]
struct ErrorResponse {
    message: String,
}

fn stream_body(body: Body) -> impl Stream<Item = Result<Bytes>> {
    async fn unfold(mut body: Body) -> Option<(Result<Bytes>, Body)> {
        body.next()
            .await
            .map(|chunk| (chunk.map_err(Error::from), body))
    }

    stream::unfold(body, unfold)
}

static JSON_WHITESPACE: &[u8] = b"\r\n";

fn stream_json_body(body: Body) -> impl Stream<Item = Result<Bytes>> {
    async fn unfold(mut body: Body) -> Option<(Result<Bytes>, Body)> {
        let mut chunk = Vec::new();
        while let Some(chnk) = body.next().await {
            match chnk {
                Ok(chnk) => {
                    chunk.extend(chnk.to_vec());
                    if chnk.ends_with(JSON_WHITESPACE) {
                        break;
                    }
                }
                Err(e) => {
                    return Some((Err(Error::from(e)), body));
                }
            }
        }

        if chunk.is_empty() {
            return None;
        }

        Some((Ok(Bytes::from(chunk)), body))
    }

    stream::unfold(body, unfold)
}
