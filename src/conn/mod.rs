//! Connection related items
pub mod transport;
pub mod tty;

pub use transport::*;
pub use tty::*;

pub use http;
pub use hyper;

use hyper::client::HttpConnector;
use hyper::StatusCode;
use thiserror::Error as ThisError;

#[cfg(feature = "tls")]
use {
    hyper_openssl::HttpsConnector,
    openssl::error::ErrorStack,
    openssl::ssl::{SslConnector, SslFiletype, SslMethod},
    std::path::Path,
};

/// Common result type used throughout this crate
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, ThisError)]
/// All error variants that can happen during communication.
pub enum Error {
    #[error("The HTTP connection was not upgraded by the podman host")]
    ConnectionNotUpgraded,
    #[error(transparent)]
    #[allow(clippy::upper_case_acronyms)]
    IO(#[from] futures_util::io::Error),
    #[error("error {code} - {message}")]
    Fault { code: StatusCode, message: String },
    #[error("Failed to parse uri - {0}")]
    InvalidUri(http::uri::InvalidUri),
    #[error(transparent)]
    Hyper(#[from] hyper::Error),
    #[error(transparent)]
    Http(#[from] hyper::http::Error),
    #[error(transparent)]
    Encoding(#[from] std::string::FromUtf8Error),
    #[cfg(feature = "tls")]
    #[error(transparent)]
    ErrorStack(#[from] ErrorStack),
    #[error(transparent)]
    Any(Box<dyn std::error::Error + 'static + Send + Sync>),
}

pub const AUTH_HEADER: &str = "X-Registry-Auth";

pub fn get_http_connector() -> HttpConnector {
    let mut http = HttpConnector::new();
    http.enforce_http(false);

    http
}

#[cfg(feature = "tls")]
pub fn get_https_connector(
    cert_path: &Path,
    verify: bool,
) -> Result<HttpsConnector<HttpConnector>> {
    let mut ssl = SslConnector::builder(SslMethod::tls())?;
    ssl.set_cipher_list("DEFAULT")?;
    ssl.set_certificate_file(&cert_path.join("cert.pem"), SslFiletype::PEM)?;
    ssl.set_private_key_file(&cert_path.join("key.pem"), SslFiletype::PEM)?;
    verify.then(|| ssl.set_ca_file(&cert_path.join("ca.pem")));

    HttpsConnector::with_connector(get_http_connector(), ssl).map_err(Error::from)
}

#[cfg(unix)]
pub fn get_unix_connector() -> hyperlocal::UnixConnector {
    hyperlocal::UnixConnector
}
