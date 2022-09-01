use crate::conn::{
    self, build_request, get_response_string, stream_json_response, stream_response, Headers,
    Payload, Transport,
};
use futures;
use futures_util::{
    future,
    io::{AsyncRead, AsyncWrite},
    stream,
    stream::Stream,
    FutureExt, TryStreamExt,
};
use hyper::{body::Bytes, Body, Method, Request, Response};
use log::trace;
use serde::de::DeserializeOwned;

#[derive(Debug, Clone)]
pub struct RequestClient<E> {
    transport: Transport,
    validate_fn: Box<ValidateResponseFn<E>>,
    _error_type: std::marker::PhantomData<E>,
}

pub type ValidateResponseFn<E> = fn(Response<Body>) -> Result<Response<Body>, E>;

impl<E: From<conn::Error> + From<serde_json::Error>> RequestClient<E> {
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
        self.transport
            .request(req)
            .await
            .map_err(E::from)
            .and_then(|resp| (&self.validate_fn)(resp))
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

    /// Make a GET request to the `endpoint` and return a stream of byte chunks.
    pub fn get_stream(
        &self,
        endpoint: impl AsRef<str>,
    ) -> impl Stream<Item = Result<Bytes, E>> + '_ {
        async {
            let response = self.get(endpoint).await?;
            stream_response(response)
        }
        .flatten()
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
        self.transport
            .request(req)
            .await
            .map_err(E::from)
            .and_then(|resp| (&self.validate_fn)(resp))
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

    /// Make a straeming POST request to the `endpoint` and return a
    /// stream of byte chunks.
    ///
    /// Use [`post_into_stream`](RequestClient::post_into_stream) if the endpoint
    /// returns JSON values.
    pub fn post_stream<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> impl Stream<Item = Result<Bytes, E>> + '_
    where
        B: Into<Body>,
    {
        let req = self.make_request(Method::POST, endpoint.as_ref(), body, headers);
        self.transport.stream_chunks(req).map_err(E::from)
    }

    /// Send a streaming post request.
    fn post_json_stream<B>(
        &self,
        endpoint: impl AsRef<str>,
        body: Payload<B>,
        headers: Option<Headers>,
    ) -> impl Stream<Item = Result<Bytes, E>> + '_
    where
        B: Into<Body>,
    {
        let req = self.make_request(Method::POST, endpoint.as_ref(), body, headers);
        self.transport.stream_json_chunks(req).map_err(E::from)
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
        self.transport
            .stream_upgrade(Method::POST, endpoint, body)
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
        self.transport
            .request(req)
            .await
            .map_err(E::from)
            .and_then(|resp| (&self.validate_fn)(resp))
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
        self.transport
            .request(req)
            .await
            .map_err(E::from)
            .and_then(|resp| (&self.validate_fn)(resp))
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
        self.transport
            .request(req)
            .await
            .map_err(E::from)
            .and_then(|resp| (&self.validate_fn)(resp))
    }
}
