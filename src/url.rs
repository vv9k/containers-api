//! Utility functions to handle URL manipulation.

pub use url;

use std::{borrow::Borrow, string::ToString};
use url::form_urlencoded;

/// Creates an endpoint with a query
pub fn construct_ep<E, Q>(ep: E, query: Option<Q>) -> String
where
    E: Into<String>,
    Q: AsRef<str>,
{
    let mut ep = ep.into();
    if let Some(query) = query {
        append_query(&mut ep, query);
    }
    ep
}

/// Appends a query to an endpoint
pub fn append_query<Q>(ep: &mut String, query: Q)
where
    Q: AsRef<str>,
{
    ep.push('?');
    ep.push_str(query.as_ref());
}

/// Encodes `key` and `val` as urlencoded values.
pub fn encoded_pair<K, V>(key: K, val: V) -> String
where
    K: AsRef<str>,
    V: ToString,
{
    form_urlencoded::Serializer::new(String::new())
        .append_pair(key.as_ref(), &val.to_string())
        .finish()
}

/// Encodes multiple values for the same key
pub fn encoded_vec_pairs<K, I>(pairs: impl IntoIterator<Item = (K, I)>) -> String
where
    K: AsRef<str>,
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut serializer = form_urlencoded::Serializer::new(String::new());
    pairs.into_iter().for_each(|(key, vals)| {
        let key = key.as_ref();
        vals.into_iter().for_each(|val| {
            serializer.append_pair(key, val.as_ref());
        });
    });

    serializer.finish()
}

/// Encodes an iterator of key:value pairs as urlencoded values.
pub fn encoded_pairs<I, K, V>(iter: I) -> String
where
    I: IntoIterator,
    I::Item: Borrow<(K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    iter.into_iter()
        .fold(
            form_urlencoded::Serializer::new(String::new()),
            |mut acc, v| {
                let (k, v) = v.borrow();
                let k = k.as_ref();
                let v = v.as_ref();
                if v.is_empty() {
                    acc.append_key_only(k);
                } else {
                    acc.append_pair(k, v);
                }
                acc
            },
        )
        .finish()
}

#[cfg(test)]
mod tests {
    use super::{append_query, construct_ep, encoded_pair, encoded_pairs, encoded_vec_pairs};

    #[test]
    fn appends_query() {
        let mut ep = "http://somewebsite.xxx".to_owned();
        let query = "lang=en";
        let want = "http://somewebsite.xxx?lang=en";
        append_query(&mut ep, query);
        assert_eq!(ep, want);
    }

    #[test]
    fn constructs_endpoint() {
        let ep = "http://somewebsite.xxx";
        let query = "lang=en,id=55555";
        let want = "http://somewebsite.xxx?lang=en,id=55555";
        assert_eq!(construct_ep(ep, None::<&str>), ep);
        assert_eq!(construct_ep(ep, Some(query)), want);
    }

    #[test]
    fn encodes_pair() {
        let key = "lang";
        let val = "en&";
        let want = "lang=en%26";
        assert_eq!(encoded_pair(key, val), want);
    }

    #[test]
    fn encodes_pairs() {
        let pairs = [("lang", "en&"), ("id", "1337"), ("country", "xxx")];
        let want = "lang=en%26&id=1337&country=xxx";
        assert_eq!(encoded_pairs(pairs), want);
    }

    #[test]
    fn encodes_vec_pairs() {
        let pairs = [
            ("lang", vec!["en", "pl&"]),
            ("id", vec!["1337"]),
            ("country", vec!["xxx", "yyy", "zzz"]),
        ];
        let want = "lang=en&lang=pl%26&id=1337&country=xxx&country=yyy&country=zzz";
        assert_eq!(encoded_vec_pairs(pairs), want);
    }
}
