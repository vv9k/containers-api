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
    K: AsRef<str> + 'static,
    V: ToString,
{
    form_urlencoded::Serializer::new(String::new())
        .append_pair(key.as_ref(), &val.to_string())
        .finish()
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
