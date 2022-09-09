use crate::conn::{
    self, build_request, get_response_string, stream_json_response, stream_response, Compat,
    Headers, Payload, Transport,
};
use futures_util::{
    io::{AsyncRead, AsyncWrite},
    stream::Stream,
    TryFutureExt, TryStreamExt,
};
use hyper::{body::Bytes, header, Body, Method, Request, Response, StatusCode};
use log::trace;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone)]
pub struct RequestClient<E> {
    transport: Transport,
    validate_fn: Box<ValidateResponseFn<E>>,
    _error_type: std::marker::PhantomData<E>,
}

pub type ValidateResponseFn<E> =
    fn(Response<Body>) -> Pin<Box<dyn Future<Output = Result<Response<Body>, E>> + Send + Sync>>;

impl<E: From<conn::Error> + From<serde_json::Error>> RequestClient<E> {
    /// Creates a new RequestClient with a specified transport and a function to validate
    /// each response.
    pub fn new(transport: Transport, validate_fn: Box<ValidateResponseFn<E>>) -> Self {
        Self {
            transport,
            validate_fn,
            _error_type: std::marker::PhantomData,
        }
    }

    fn make_request<B>(
        &self,
        method: http::Method,
        endpoint: &str,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> conn::Result<Request<Body>>
    where
        B: Into<Body>,
    {
        let uri = self.transport.make_uri(endpoint)?;
        build_request(method, uri, body, headers)
    }

    async fn send_request(&self, request: Request<Body>) -> Result<Response<Body>, E> {
        let response = self.transport.request(request).await.map_err(E::from)?;
        (self.validate_fn)(response).await
    }

    //####################################################################################################
    // GET
    //####################################################################################################

    /// Make a GET request to the `endpoint` and return the response.
    pub async fn get(&self, endpoint: impl AsRef<str>) -> Result<Response<Body>, E> {
        let req = self.make_request(
            Method::GET,
            endpoint.as_ref(),
            Payload::empty(),
            Headers::none(),
        );
        self.send_request(req?).await
    }

    /// Make a GET request to the `endpoint` and return the response as a string.
    pub async fn get_string(&self, endpoint: impl AsRef<str>) -> Result<String, E> {
        let response = self.get(endpoint).await?;
        get_response_string(response).await.map_err(E::from)
    }

    /// Make a GET request to the `endpoint` and return the response as a JSON deserialized object.
    pub async fn get_json<T: DeserializeOwned>(&self, endpoint: impl AsRef<str>) -> Result<T, E> {
        let raw_string = self.get_string(endpoint).await?;
        trace!("{raw_string}");
        serde_json::from_str::<T>(&raw_string).map_err(E::from)
    }

    async fn get_stream_impl(
        &self,
        endpoint: impl AsRef<str>,
    ) -> Result<impl Stream<Item = Result<Bytes, E>> + '_, E> {
        let response = self.get(endpoint).await?;
        Ok(stream_response(response).map_err(E::from))
    }

    /// Make a GET request to the `endpoint` and return a stream of byte chunks.
    pub fn get_stream<'client>(
        &'client self,
        endpoint: impl AsRef<str> + 'client,
    ) -> impl Stream<Item = Result<Bytes, E>> + 'client {
        self.get_stream_impl(endpoint).try_flatten_stream()
    }

    /// Make a GET request to the `endpoint` and return a stream of JSON chunk results.
    pub fn get_json_stream<'client, T>(
        &'client self,
        endpoint: impl AsRef<str> + 'client,
    ) -> impl Stream<Item = Result<T, E>> + 'client
    where
        T: DeserializeOwned,
    {
        self.get_stream(endpoint)
            .and_then(|chunk| async move {
                let stream = futures_util::stream::iter(
                    serde_json::Deserializer::from_slice(&chunk)
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
                .map_err(E::from);

                Ok(stream)
            })
            .try_flatten()
    }

    //####################################################################################################
    // POST
    //####################################################################################################

    /// Make a POST request to the `endpoint` and return the response.
    pub async fn post<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> Result<Response<Body>, E>
    where
        B: Into<Body>,
    {
        let req = self.make_request(Method::POST, endpoint.as_ref(), body, headers);
        self.send_request(req?).await
    }

    /// Make a POST request to the `endpoint` and return the response as a string.
    pub async fn post_string<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> Result<String, E>
    where
        B: Into<Body>,
    {
        let response = self.post(endpoint, body, headers).await?;
        get_response_string(response).await.map_err(E::from)
    }

    /// Make a POST request to the `endpoint` and return the response as a JSON
    /// deserialized value.
    pub async fn post_json<B, T>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> Result<T, E>
    where
        T: DeserializeOwned,
        B: Into<Body>,
    {
        let raw_string = self.post_string(endpoint, body, headers).await?;
        trace!("{raw_string}");
        serde_json::from_str::<T>(&raw_string).map_err(E::from)
    }

    async fn post_stream_impl<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> Result<impl Stream<Item = Result<Bytes, E>> + '_, E>
    where
        B: Into<Body>,
    {
        let response = self.post(endpoint, body, headers).await?;
        Ok(stream_response(response).map_err(E::from))
    }

    /// Make a straeming POST request to the `endpoint` and return a
    /// stream of byte chunks.
    ///
    /// Use [`post_into_stream`](RequestClient::post_into_stream) if the endpoint
    /// returns JSON values.
    pub fn post_stream<'client, B>(
        &'client self,
        endpoint: impl AsRef<str> + 'client,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> impl Stream<Item = Result<Bytes, E>> + 'client
    where
        B: Into<Body> + 'client,
    {
        self.post_stream_impl(endpoint, body, headers)
            .try_flatten_stream()
    }

    async fn post_json_stream_impl<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> Result<impl Stream<Item = Result<Bytes, E>> + '_, E>
    where
        B: Into<Body>,
    {
        let response = self.post(endpoint, body, headers).await?;
        Ok(stream_json_response(response).map_err(E::from))
    }

    /// Send a streaming post request.
    fn post_json_stream<'client, B>(
        &'client self,
        endpoint: impl AsRef<str> + 'client,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> impl Stream<Item = Result<Bytes, E>> + 'client
    where
        B: Into<Body> + 'client,
    {
        self.post_json_stream_impl(endpoint, body, headers)
            .try_flatten_stream()
    }

    /// Make a streaming POST request to the `endpoint` and return a stream of
    /// JSON deserialized chunks.
    pub fn post_into_stream<'client, B, T>(
        &'client self,
        endpoint: impl AsRef<str> + 'client,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> impl Stream<Item = Result<T, E>> + 'client
    where
        B: Into<Body> + 'client,
        T: DeserializeOwned,
    {
        self.post_json_stream(endpoint, body, headers)
            .and_then(|chunk| async move {
                trace!("got chunk {:?}", chunk);
                let stream = futures_util::stream::iter(
                    serde_json::Deserializer::from_slice(&chunk)
                        .into_iter()
                        .collect::<Vec<_>>(),
                )
                .map_err(E::from);

                Ok(stream)
            })
            .try_flatten()
    }

    pub async fn post_upgrade_stream<'client, B>(
        &'client self,
        endpoint: impl AsRef<str> + 'client,
        body: Payload<B>,
    ) -> Result<impl AsyncRead + AsyncWrite + 'client, E>
    where
        B: Into<Body> + 'client,
    {
        self.stream_upgrade(Method::POST, endpoint, body)
            .await
            .map_err(E::from)
    }

    //####################################################################################################
    // PUT
    //####################################################################################################

    /// Make a PUT request to the `endpoint` and return the response.
    pub async fn put<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
    ) -> Result<Response<Body>, E>
    where
        B: Into<Body>,
    {
        let req = self.make_request(Method::PUT, endpoint.as_ref(), body, Headers::none());
        self.send_request(req?).await
    }

    /// Make a PUT request to the `endpoint` and return the response as a string.
    pub async fn put_string<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
    ) -> Result<String, E>
    where
        B: Into<Body>,
    {
        let response = self.put(endpoint, body).await?;
        get_response_string(response).await.map_err(E::from)
    }

    //####################################################################################################
    // DELETE
    //####################################################################################################

    /// Make a DELETE request to the `endpoint` and return the response.
    pub async fn delete(&self, endpoint: impl AsRef<str>) -> Result<Response<Body>, E> {
        let req = self.make_request(
            Method::DELETE,
            endpoint.as_ref(),
            Payload::empty(),
            Headers::none(),
        );
        self.send_request(req?).await
    }

    /// Make a DELETE request to the `endpoint` and return the response as a string.
    pub async fn delete_string(&self, endpoint: impl AsRef<str>) -> Result<String, E> {
        let response = self.delete(endpoint).await?;
        get_response_string(response).await.map_err(E::from)
    }

    /// Make a DELETE request to the `endpoint` and return the response as a JSON
    /// deserialized object.
    pub async fn delete_json<T: DeserializeOwned>(
        &self,
        endpoint: impl AsRef<str>,
    ) -> Result<T, E> {
        let raw_string = self.delete_string(endpoint).await?;
        trace!("{raw_string}");
        serde_json::from_str::<T>(&raw_string).map_err(E::from)
    }

    //####################################################################################################
    // HEAD
    //####################################################################################################

    /// Make a HEAD request to the `endpoint` and return the response.
    pub async fn head(&self, endpoint: impl AsRef<str>) -> Result<Response<Body>, E> {
        let req = self.make_request(
            Method::HEAD,
            endpoint.as_ref(),
            Payload::empty(),
            Headers::none(),
        );
        self.send_request(req?).await
    }

    //####################################################################################################
    // STREAM
    //####################################################################################################

    async fn stream_upgrade<B>(
        &self,
        method: Method,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
    ) -> Result<impl AsyncRead + AsyncWrite, E>
    where
        B: Into<Body>,
    {
        self.stream_upgrade_tokio(method, endpoint.as_ref(), body)
            .await
            .map(Compat::new)
    }

    /// Makes an HTTP request, upgrading the connection to a TCP
    /// stream on success.
    async fn stream_upgrade_tokio<B>(
        &self,
        method: Method,
        endpoint: &str,
        body: Payload<B>,
    ) -> Result<hyper::upgrade::Upgraded, E>
    where
        B: Into<Body>,
    {
        let mut headers = Headers::default();
        headers.add(header::CONNECTION.as_str(), "Upgrade");
        headers.add(header::UPGRADE.as_str(), "tcp");

        let req = self.make_request(method, endpoint, body, Some(headers));

        let response = self.send_request(req?).await?;
        match response.status() {
            StatusCode::SWITCHING_PROTOCOLS => Ok(hyper::upgrade::on(response)
                .await
                .map_err(conn::Error::from)?),
            _ => Err(E::from(conn::Error::ConnectionNotUpgraded)),
        }
    }
}
