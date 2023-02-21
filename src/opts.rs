/// Types that implement Filter can be used in filter queries.
pub trait Filter {
    fn query_item(&self) -> FilterItem;
}

pub struct FilterItem {
    key: &'static str,
    value: String,
}

impl FilterItem {
    pub fn new(key: &'static str, value: impl Into<String>) -> Self {
        Self {
            key,
            value: value.into(),
        }
    }

    pub fn key(&self) -> &'static str {
        self.key
    }
}

impl std::fmt::Display for FilterItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl From<(&'static str, String)> for FilterItem {
    fn from(it: (&'static str, String)) -> Self {
        Self::new(it.0, it.1)
    }
}

#[macro_export]
/// Implements methods to set a parameter of a specified type serialized as JSON.
macro_rules! impl_field {
    ($(#[doc = $docs:expr])* $name:ident: $ty:ty => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: $ty)-> Self
            {
                self.params.insert($param_name, serde_json::json!($name));
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a specified parameter that contains a seuquence of items serialized as JSON.
macro_rules! impl_vec_field {
    ($(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name  >]<S>(mut self, $name: impl IntoIterator<Item = S>)-> Self
            where
                S: serde::Serialize
            {
                self.params.insert($param_name, serde_json::json!($name.into_iter().collect::<Vec<_>>()));
                self
            }
        }
    };
    ($(#[doc = $docs:expr])* $name:ident: $ty:ty => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name  >](mut self, $name: impl IntoIterator<Item = $ty>)-> Self
            {
                self.params.insert($param_name, serde_json::json!($name.into_iter().collect::<Vec<_>>()));
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a string parameter serialized as JSON.
macro_rules! impl_str_field {
    ($(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: impl serde::Serialize)-> Self
            {
                self.params.insert($param_name, serde_json::json!($name));
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded string enum parameter.
macro_rules! impl_str_enum_field {
    ($(#[doc = $docs:expr])* $name:ident: $ty:tt => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: $ty)-> Self
            {
                self.params.insert($param_name, serde_json::json!($name.to_string()));
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded string parameter.
macro_rules! impl_url_str_field {
    ($(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: impl Into<String>)-> Self
            {
                self.params.insert($param_name, $name.into());
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded parameter of a specified type.
macro_rules! impl_url_field {
    ($(#[doc = $docs:expr])* $name:ident : $ty:tt => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: $ty)-> Self {
                self.params.insert($param_name, $name.to_string());
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded parameter of a sequence of items.
macro_rules! impl_url_vec_field {
    ($(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >]<S>(mut self, $name: impl IntoIterator<Item = S>)-> Self
            where
                S: Into<String>
            {
                self.vec_params.insert($param_name, $name.into_iter().map(|s| s.into()).collect());
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded parameter of a boolean.
macro_rules! impl_url_bool_field {
    ($(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: bool)-> Self {
                self.params.insert($param_name, $name.to_string());
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded enum parameter.
macro_rules! impl_url_enum_field {
    ($(#[doc = $docs:expr])* $name:ident: $ty:tt => $param_name:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name >](mut self, $name: $ty)-> Self
            {
                self.params.insert($param_name, $name.to_string());
                self
            }
        }
    };
}

#[macro_export]
/// Implements methods to set a urlencoded squence of key:value items.
macro_rules! impl_map_field {
    (url $(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        impl_map_field! { $(#[doc = $docs])* $name => $param_name => serde_json::to_string(&$name.into_iter().collect::<std::collections::HashMap<_, _>>()).unwrap_or_default() }
    };
    (json $(#[doc = $docs:expr])* $name:ident => $param_name:literal) => {
        impl_map_field! { $(#[doc = $docs])* $name => $param_name => serde_json::json!($name.into_iter().collect::<std::collections::HashMap<_, _>>()) }
    };
    ($(#[doc = $docs:expr])* $name:ident => $param_name:literal => $ret:expr) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            pub fn [< $name  >]<K, V>(mut self, $name: impl IntoIterator<Item = (K, V)>)-> Self
            where
                K: serde::Serialize + Eq + std::hash::Hash,
                V: serde::Serialize
            {
                self.params.insert($param_name, $ret);
                self
            }
        }
    };
}

#[macro_export]
/// Implements a filter method that uses a [`Filter`](crate::opts::Filter) trait parameter
macro_rules! impl_filter_func {
    ($(#[doc = $doc:expr])* $filter_ty:ident) => {
        $(
            #[doc = $doc]
        )*
        pub fn filter(mut self, filters: impl IntoIterator<Item = $filter_ty>) -> Self
        {
            let mut param = std::collections::BTreeMap::new();
            for filter_item in filters.into_iter().map(|f| f.query_item()) {
                let key = filter_item.key();
                let entry_vec = param.entry(key).or_insert(Vec::new());
                entry_vec.push(filter_item.to_string());
            }
            // structure is a a json encoded object mapping string keys to a list
            // of string values
            self.params
                .insert("filters", serde_json::to_string(&param).unwrap_or_default());
            self
        }
    };
}

#[macro_export]
macro_rules! impl_url_serialize {
    ($name: ident) => {
        paste::item! {
            impl [< $name  Opts >] {
                /// Serialize options as a URL query String. Returns None if no options are defined.
                pub fn serialize(&self) -> Option<String> {
                    let params = $crate::url::encoded_pairs(&self.params);
                    let vec_params = $crate::url::encoded_vec_pairs(&self.vec_params);

                    let mut serialized = format!("{params}");
                    if !vec_params.is_empty() {
                        if !serialized.is_empty() {
                            serialized.push('&');
                        }
                        serialized.push_str(&vec_params);
                    }

                    if serialized.is_empty() {
                        None
                    } else {
                        Some(serialized)
                    }
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_json_serialize {
    ($name: ident) => {
        paste::item! {
            impl [< $name Opts >] {
                /// Serialize options as a JSON String. Returns an error if the options will fail
                /// to serialize.
                pub fn serialize(&self) -> crate::Result<String> {
                    serde_json::to_string(&self.params).map_err(crate::Error::from)
                }

                /// Serialize options as a JSON bytes. Returns an error if the options will fail
                /// to serialize.
                pub fn serialize_vec(&self) -> crate::Result<Vec<u8>> {
                    serde_json::to_vec(&self.params).map_err(crate::Error::from)
                }
            }
        }
    };
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
/// Initialize a `Opts` struct with a `OptsBuilder` struct to construct it.
macro_rules! define_opts_builder {
    (base_json $(#[doc = $docs:expr])* $name:ident $ty:expr) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            #[derive(serde::Serialize, Debug, Default, Clone)]
            pub struct [< $name Opts >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, $ty>,
            }

            #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
            #[derive(Default, Debug, Clone)]
            pub struct [< $name OptsBuilder >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, $ty>,
            }
        }
    };
    (base_url $(#[doc = $docs:expr])* $name:ident $ty:expr) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            #[derive(serde::Serialize, Debug, Default, Clone)]
            pub struct [< $name Opts >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, $ty>,
                pub(crate) vec_params: std::collections::BTreeMap<&'static str, Vec<$ty>>,
            }

            #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
            #[derive(Default, Debug, Clone)]
            pub struct [< $name OptsBuilder >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, $ty>,
                pub(crate) vec_params: std::collections::BTreeMap<&'static str, Vec<$ty>>,
            }
        }
    }
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
/// Initialize a `Opts` struct with a `OptsBuilder` struct to construct it.
macro_rules! impl_opts_builder {
    (__builder $name:ident) => {
        paste::item! {
            impl [< $name Opts >] {
                #[doc = concat!("Returns a new instance of a builder for ", stringify!($name), "Opts.")]
                pub fn builder() -> [< $name OptsBuilder >] {
                    [< $name OptsBuilder >]::default()
                }
            }
        }
    };
    (base_json $(#[doc = $docs:expr])* $name:ident $ty:expr) => {
        $crate::define_opts_builder!(base_json $(#[doc = $docs])* $name $ty);
        impl_opts_builder!(__builder $name);
        paste::item! {
            impl [< $name OptsBuilder >] {
                #[doc = concat!("Finish building ", stringify!($name), "Opts.")]
                pub fn build(self) -> [< $name Opts >] {
                    [< $name Opts >] {
                        params: self.params,
                    }
                }
            }
       }
    };
    (base_url $(#[doc = $docs:expr])* $name:ident $ty:expr) => {
        $crate::define_opts_builder!(base_url $(#[doc = $docs])* $name $ty);
        impl_opts_builder!(__builder $name);
        paste::item! {
            impl [< $name OptsBuilder >] {
                #[doc = concat!("Finish building ", stringify!($name), "Opts.")]
                pub fn build(self) -> [< $name Opts >] {
                    [< $name Opts >] {
                        params: self.params,
                        vec_params: self.vec_params
                    }
                }
            }
       }
    };
    (json => $(#[doc = $docs:expr])* $name:ident) => {
        paste::item! {
            impl_opts_builder!(base_json $(#[doc = $docs])* $name serde_json::Value);
            $crate::impl_json_serialize!($name);
        }
    };
    (url => $(#[doc = $docs:expr])* $name:ident) => {
        paste::item! {
            impl_opts_builder!(base_url $(#[doc = $docs])* $name String);
            $crate::impl_url_serialize!($name);
        }
    };
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
/// Initialize a `Opts` struct with a required parameter and `OptsBuilder` struct to construct it.
macro_rules! impl_opts_required_builder {
    (__builder $name:ident $ty:expr; $(#[doc = $param_docs:expr])* $param:ident: $param_ty:expr) => {
        paste::item! {
            impl [< $name Opts >] {
                #[doc = concat!("Returns a new instance of a builder for ", stringify!($name), "Opts.")]
                $(
                    #[doc= $param_docs]
                )*
                pub fn builder($param: impl Into<$param_ty>) -> [< $name OptsBuilder >] {
                    [< $name OptsBuilder >]::new($param)
                }

                pub fn get_param(&self, key: &str) -> Option<&$ty> {
                    self.params.get(key)
                }
            }
        }
    };
    (base_json $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident: $param_ty:expr => $param_key:literal) => {
        impl_opts_required_builder!(__builder $name serde_json::Value; $(#[doc = $param_docs])* $param: $param_ty);
        paste::item! {
            $(
                #[doc= $docs]
            )*
            #[derive(serde::Serialize, Debug, Default, Clone)]
            pub struct [< $name Opts >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, serde_json::Value>,
                [< $param >]: $param_ty,
            }
            impl [< $name Opts >] {
                pub fn [< $param >](&self) -> &$param_ty {
                    &self.$param
                }
            }

            #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
            #[derive(Default, Debug, Clone)]
            pub struct [< $name OptsBuilder >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, serde_json::Value>,
                [< $param >]: $param_ty,
            }
            impl [< $name OptsBuilder >] {
                #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
                $(
                    #[doc= $param_docs]
                )*
                pub fn new($param: impl Into<$param_ty>) -> Self {
                    let param = $param.into();
                    Self {
                        params: [($param_key, serde_json::json!(param.clone()))].into(),
                        [< $param >]: param,
                    }
                }

                #[doc = concat!("Finish building ", stringify!($name), "Opts.")]
                pub fn build(self) -> [< $name Opts >] {
                    [< $name Opts >] {
                        params: self.params,
                        [< $param >]: self.$param
                    }
                }
            }
       }
    };
    (base_url $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident: $param_ty:expr => $param_key:literal) => {
        impl_opts_required_builder!(__builder $name String; $(#[doc = $param_docs])* $param: $param_ty);
        paste::item! {
            $(
                #[doc= $docs]
            )*
            #[derive(serde::Serialize, Debug, Default, Clone)]
            pub struct [< $name Opts >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, String>,
                pub(crate) vec_params: std::collections::BTreeMap<&'static str, Vec<String>>,
                [< $param >]: $param_ty,
            }
            impl [< $name Opts >] {
                pub fn [< $param >](&self) -> &$param_ty {
                    &self.$param
                }
            }

            #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
            #[derive(Debug, Clone)]
            pub struct [< $name OptsBuilder >] {
                pub(crate) params: std::collections::BTreeMap<&'static str, String>,
                pub(crate) vec_params: std::collections::BTreeMap<&'static str, Vec<String>>,
                [< $param >]: $param_ty,
            }

            impl [< $name OptsBuilder >] {
                #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
                $(
                    #[doc= $param_docs]
                )*
                pub fn new($param: impl Into<$param_ty>) -> Self {
                    let param = $param.into();
                    Self {
                        params: [($param_key, param.clone())].into(),
                        vec_params: Default::default(),
                        [< $param >]: param,
                    }
                }

                #[doc = concat!("Finish building ", stringify!($name), "Opts.")]
                pub fn build(self) -> [< $name Opts >] {
                    [< $name Opts >] {
                        params: self.params,
                        vec_params: self.vec_params,
                        [< $param >]: self.$param,
                    }
                }
            }
       }
    };
    (json => $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident: $param_ty:expr => $param_key:literal) => {
        impl_opts_required_builder!(base_json $(#[doc = $docs])* $name, $(#[doc = $param_docs])* $param: $param_ty => $param_key);
        $crate::impl_json_serialize!($name);
    };
    (json => $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident => $param_key:literal) => {
        impl_opts_required_builder!(base_json $(#[doc = $docs])* $name, $(#[doc = $param_docs])* $param: serde_json::Value => $param_key);
        $crate::impl_json_serialize!($name);
    };
    (url => $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident: $param_ty:expr => $param_key:literal) => {
        impl_opts_required_builder!(base_url $(#[doc = $docs])* $name, $(#[doc = $param_docs])* $param => $param_key);
        $crate::impl_url_serialize!($name);
    };
    (url => $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident => $param_key:literal) => {
        impl_opts_required_builder!(base_url $(#[doc = $docs])* $name, $(#[doc = $param_docs])* $param: String => $param_key);
        $crate::impl_url_serialize!($name);
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn url_filter_query() {
        pub enum ListFilter {
            Id(crate::id::Id),
            LabelKey(String),
            LabelKeyVal(String, String),
        }

        impl Filter for ListFilter {
            fn query_item(&self) -> FilterItem {
                use ListFilter::*;
                match &self {
                    Id(id) => FilterItem::new("id", id.to_string()),
                    LabelKey(key) => FilterItem::new("label", key.clone()),
                    LabelKeyVal(key, val) => FilterItem::new("label", format!("{key}={val}")),
                }
            }
        }

        impl_opts_builder! (url =>
            UrlTest
        );

        impl UrlTestOptsBuilder {
            impl_filter_func!(ListFilter);
        }

        let opts = UrlTestOpts::builder()
            .filter([
                ListFilter::Id("testid".into()),
                ListFilter::LabelKey("test1".into()),
                ListFilter::LabelKeyVal("test2".into(), "key".into()),
            ])
            .build();

        let want = Some("filters=%7B%22id%22%3A%5B%22testid%22%5D%2C%22label%22%3A%5B%22test1%22%2C%22test2%3Dkey%22%5D%7D".into());
        let got = opts.serialize();
        assert_eq!(got, want);
    }

    #[test]
    fn url_vec_query() {
        impl_opts_builder! (url =>
            UrlTest
        );

        impl UrlTestOptsBuilder {
            impl_url_vec_field!(
                test => "tests"
            );
        }

        let opts = UrlTestOpts::builder().test(["abc", "def", "ghi"]).build();

        let want = Some("tests=abc&tests=def&tests=ghi".into());
        let got = opts.serialize();
        assert_eq!(got, want);
    }
}
