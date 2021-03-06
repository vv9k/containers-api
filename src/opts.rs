/// Types that implement Filter can be used in filter queries.
pub trait Filter {
    // TODO: Add a stronger return type. Not all filters are `key=val`, soma are only `key`
    fn query_key_val(&self) -> (&'static str, String);
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
                let joined = $name.into_iter().map(|it| it.into()).collect::<Vec<_>>().join(",");
                self.params.insert($param_name, format!("{}",joined));
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
            let mut param = std::collections::HashMap::new();
            for (key, val) in filters.into_iter().map(|f| f.query_key_val()) {
                param.insert(key, vec![val]);
            }
            // structure is a a json encoded object mapping string keys to a list
            // of string values
            self.params
                .insert("filters", serde_json::to_string(&param).unwrap_or_default());
            self
        }
    };
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
/// Initialize a `Opts` struct with a `OptsBuilder` struct to construct it.
macro_rules! impl_opts_builder {
    ($(#[doc = $docs:expr])* $name:ident $ty:expr) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            #[derive(serde::Serialize, Debug, Default, Clone)]
            pub struct [< $name Opts >] {
                params: std::collections::HashMap<&'static str, $ty>,
            }
            impl [< $name Opts >] {
                #[doc = concat!("Returns a new instance of a builder for ", stringify!($name), "Opts.")]
                pub fn builder() -> [< $name OptsBuilder >] {
                    [< $name OptsBuilder >]::default()
                }
            }

            #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
            #[derive(Default, Debug, Clone)]
            pub struct [< $name OptsBuilder >] {
                params: std::collections::HashMap<&'static str, $ty>,
            }

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
    (json => $(#[doc = $docs:expr])* $name:ident) => {
        paste::item! {
            impl_opts_builder!($(#[doc = $docs])* $name serde_json::Value);

            impl [< $name Opts >] {
                /// Serialize options as a JSON String. Returns an error if the options will fail
                /// to serialize.
                pub fn serialize(&self) -> crate::Result<String> {
                    serde_json::to_string(&self.params).map_err(crate::Error::from)
                }
            }
        }
    };
    (url => $(#[doc = $docs:expr])* $name:ident) => {
        paste::item! {
            impl_opts_builder!($(#[doc = $docs])* $name String);

            impl [< $name  Opts >] {
                /// Serialize options as a URL query String. Returns None if no options are defined.
                pub fn serialize(&self) -> Option<String> {
                    if self.params.is_empty() {
                        None
                    } else {
                        Some(
                            containers_api::url::encoded_pairs(&self.params)
                        )
                    }
                }
            }
        }
    };
}

#[allow(clippy::crate_in_macro_def)]
#[macro_export]
/// Initialize a `Opts` struct with a required parameter and `OptsBuilder` struct to construct it.
macro_rules! impl_opts_required_builder {
    ($(#[doc = $docs:expr])* $name:ident $ty:expr, $(#[doc = $param_docs:expr])* $param:ident => $param_key:literal) => {
        paste::item! {
            $(
                #[doc= $docs]
            )*
            #[derive(serde::Serialize, Debug, Default, Clone)]
            pub struct [< $name Opts >] {
                params: std::collections::HashMap<&'static str, $ty>,
            }
            impl [< $name Opts >] {
                #[doc = concat!("Returns a new instance of a builder for ", stringify!($name), "Opts.")]
                $(
                    #[doc= $param_docs]
                )*
                pub fn builder($param: impl Into<$ty>) -> [< $name OptsBuilder >] {
                    [< $name OptsBuilder >]::new($param)
                }

                pub fn get_param(&self, key: &str) -> Option<&$ty> {
                    self.params.get(key)
                }
            }

            #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
            #[derive(Debug, Clone)]
            pub struct [< $name OptsBuilder >] {
                params: std::collections::HashMap<&'static str, $ty>,
            }

            impl [< $name OptsBuilder >] {
                #[doc = concat!("A builder struct for ", stringify!($name), "Opts.")]
                $(
                    #[doc= $param_docs]
                )*
                pub fn new($param: impl Into<$ty>) -> Self {
                    Self {
                        params: [($param_key, $param.into())].into()
                    }
                }

                #[doc = concat!("Finish building ", stringify!($name), "Opts.")]
                pub fn build(self) -> [< $name Opts >] {
                    [< $name Opts >] {
                        params: self.params,
                    }
                }
            }
       }
    };
    (json => $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident => $param_key:literal) => {
        impl_opts_required_builder!($(#[doc = $docs])* $name serde_json::Value, $(#[doc = $param_docs])* $param => $param_key);

        paste::item! {
            impl [< $name Opts >] {
                /// Serialize options as a JSON String. Returns an error if the options will fail
                /// to serialize.
                pub fn serialize(&self) -> crate::Result<String> {
                    serde_json::to_string(&self.params).map_err(crate::Error::from)
                }
            }
        }
    };
    (url => $(#[doc = $docs:expr])* $name:ident, $(#[doc = $param_docs:expr])* $param:ident => $param_key:literal) => {
        impl_opts_required_builder!($(#[doc = $docs])* $name String, $(#[doc = $param_docs])* $param => $param_key);

        paste::item! {
            impl [< $name  Opts >] {
                /// Serialize options as a URL query String. Returns None if no options are defined.
                pub fn serialize(&self) -> Option<String> {
                    if self.params.is_empty() {
                        None
                    } else {
                        Some(
                            containers_api::url::encoded_pairs(&self.params)
                        )
                    }
                }
            }
        }
    };
}
