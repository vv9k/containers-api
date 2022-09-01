use hyper::Body;

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
