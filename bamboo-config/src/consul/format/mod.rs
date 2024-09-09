// If no features are used, there is an "unused mut" warning in `ALL_EXTENSIONS`
// BUG: ? For some reason this doesn't do anything if I try and function scope this
#![allow(unused_mut)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;

use crate::{
    map::Map,
    value::Value,
    consul::ConsulStoredFormat,
    format::Format,
};

#[cfg(feature = "toml")]
mod toml;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "yaml")]
mod yaml;

#[cfg(feature = "ini")]
mod ini;

#[cfg(feature = "ron")]
mod ron;

#[cfg(feature = "json5")]
mod json5;

/// File formats provided by the library.
///
/// Although it is possible to define custom formats using [`Format`] trait it is recommended to use FileFormat if possible.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
pub enum ConsulFormat {
    /// TOML (parsed with toml)
    #[cfg(feature = "toml")]
    Toml,

    /// JSON (parsed with serde_json)
    #[cfg(feature = "json")]
    Json,

    /// YAML (parsed with yaml_rust)
    #[cfg(feature = "yaml")]
    Yaml,

    /// INI (parsed with rust_ini)
    #[cfg(feature = "ini")]
    Ini,

    /// RON (parsed with ron)
    #[cfg(feature = "ron")]
    Ron,

    /// JSON5 (parsed with json5)
    #[cfg(feature = "json5")]
    Json5,
}

lazy_static! {
    #[doc(hidden)]
    // #[allow(unused_mut)] ?
    pub static ref ALL_EXTENSIONS: HashMap<ConsulFormat, Vec<&'static str>> = {
        let mut formats: HashMap<ConsulFormat, Vec<_>> = HashMap::new();

        #[cfg(feature = "toml")]
        formats.insert(ConsulFormat::Toml, vec!["toml"]);

        #[cfg(feature = "json")]
        formats.insert(ConsulFormat::Json, vec!["json"]);

        #[cfg(feature = "yaml")]
        formats.insert(ConsulFormat::Yaml, vec!["yaml", "yml"]);

        #[cfg(feature = "ini")]
        formats.insert(ConsulFormat::Ini, vec!["ini"]);

        #[cfg(feature = "ron")]
        formats.insert(ConsulFormat::Ron, vec!["ron"]);

        #[cfg(feature = "json5")]
        formats.insert(ConsulFormat::Json5, vec!["json5"]);

        formats
    };
}

impl ConsulFormat {
    pub(crate) fn extensions(&self) -> &'static [&'static str] {
        // It should not be possible for this to fail
        // A FileFormat would need to be declared without being added to the
        // ALL_EXTENSIONS map.
        ALL_EXTENSIONS.get(self).unwrap()
    }

    pub(crate) fn parse(
        &self,
        uri: Option<&String>,
        text: &str,
    ) -> Result<Map<String, Value>, Box<dyn Error + Send + Sync>> {
        match self {
            #[cfg(feature = "toml")]
            ConsulFormat::Toml => toml::parse(uri, text),

            #[cfg(feature = "json")]
            ConsulFormat::Json => json::parse(uri, text),

            #[cfg(feature = "yaml")]
            ConsulFormat::Yaml => yaml::parse(uri, text),

            #[cfg(feature = "ini")]
            ConsulFormat::Ini => ini::parse(uri, text),

            #[cfg(feature = "ron")]
            ConsulFormat::Ron => ron::parse(uri, text),

            #[cfg(feature = "json5")]
            ConsulFormat::Json5 => json5::parse(uri, text),

            #[cfg(all(
                not(feature = "toml"),
                not(feature = "json"),
                not(feature = "yaml"),
                not(feature = "ini"),
                not(feature = "ron"),
                not(feature = "json5"),
            ))]
            _ => unreachable!("No features are enabled, this library won't work without features"),
        }
    }
}

impl Format for ConsulFormat {
    fn parse(
        &self,
        uri: Option<&String>,
        text: &str,
    ) -> Result<Map<String, Value>, Box<dyn Error + Send + Sync>> {
        self.parse(uri, text)
    }
}

impl ConsulStoredFormat for ConsulFormat {
    fn file_extensions(&self) -> &'static [&'static str] {
        self.extensions()
    }
}
