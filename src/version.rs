use std::str::FromStr;
use thiserror::Error as ThisError;

#[derive(Debug, ThisError)]
pub enum Error {
    #[error("Invalid version - {0}")]
    MalformedVersion(String),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// Structure representing API version used to determine compatibility between a client and a server.
pub struct ApiVersion {
    major: usize,
    minor: Option<usize>,
    patch: Option<usize>,
}

impl ApiVersion {
    pub const fn new(major: usize, minor: Option<usize>, patch: Option<usize>) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub fn major(&self) -> usize {
        self.major
    }

    pub fn minor(&self) -> Option<usize> {
        self.minor
    }

    pub fn patch(&self) -> Option<usize> {
        self.patch
    }

    pub fn make_endpoint(&self, ep: impl AsRef<str>) -> String {
        let ep = ep.as_ref();
        format!(
            "/v{}{}{}",
            self,
            if !ep.starts_with('/') { "/" } else { "" },
            ep
        )
    }
}

impl std::fmt::Display for ApiVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.major)?;
        if let Some(minor) = self.minor {
            write!(f, ".{minor}")?;
        }
        if let Some(patch) = self.patch {
            write!(f, ".{patch}")?;
        }
        Ok(())
    }
}

impl From<usize> for ApiVersion {
    fn from(v: usize) -> Self {
        ApiVersion {
            major: v,
            minor: None,
            patch: None,
        }
    }
}

impl From<(usize, usize)> for ApiVersion {
    fn from(v: (usize, usize)) -> Self {
        ApiVersion {
            major: v.0,
            minor: Some(v.1),
            patch: None,
        }
    }
}

impl From<(usize, usize, usize)> for ApiVersion {
    fn from(v: (usize, usize, usize)) -> Self {
        ApiVersion {
            major: v.0,
            minor: Some(v.1),
            patch: Some(v.2),
        }
    }
}

impl FromStr for ApiVersion {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut elems = s.split('.');

        let major = if let Some(it) = elems.next() {
            match it.parse::<usize>() {
                Ok(it) => it,
                Err(e) => return Err(Error::MalformedVersion(e.to_string())),
            }
        } else {
            return Err(Error::MalformedVersion("expected major version".into()));
        };

        let minor = elems.next().and_then(|elem| elem.parse::<usize>().ok());
        let patch = elems.next().and_then(|elem| elem.parse::<usize>().ok());

        if elems.next().is_some() {
            return Err(Error::MalformedVersion(
                "unexpected extra tokens".to_string(),
            ));
        }

        Ok(Self {
            major,
            minor,
            patch,
        })
    }
}
