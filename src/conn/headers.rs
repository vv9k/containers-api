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
