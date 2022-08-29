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
                let &(ref k, ref v) = v.borrow();
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
