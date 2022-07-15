/// Types that implement Filter can be used in filter queries.
pub trait Filter {
    // TODO: Add a stronger return type. Not all filters are `key=val`, soma are only `key`
    fn query_key_val(&self) -> (&'static str, String);
}
