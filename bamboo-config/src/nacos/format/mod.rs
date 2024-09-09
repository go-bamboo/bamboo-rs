// If no features are used, there is an "unused mut" warning in `ALL_EXTENSIONS`
// BUG: ? For some reason this doesn't do anything if I try and function scope this
#![allow(unused_mut)]

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;

use crate::{Map, Value, Format};
use crate::{nacos::NacosStoredFormat};

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
pub enum NacosFormat {
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
    pub static ref ALL_EXTENSIONS: HashMap<NacosFormat, Vec<&'static str>> = {
        let mut formats: HashMap<NacosFormat, Vec<_>> = HashMap::new();

        #[cfg(feature = "toml")]
        formats.insert(NacosFormat::Toml, vec!["toml"]);

        #[cfg(feature = "json")]
        formats.insert(NacosFormat::Json, vec!["json"]);

        #[cfg(feature = "yaml")]
        formats.insert(NacosFormat::Yaml, vec!["yaml", "yml"]);

        #[cfg(feature = "ini")]
        formats.insert(NacosFormat::Ini, vec!["ini"]);

        #[cfg(feature = "ron")]
        formats.insert(NacosFormat::Ron, vec!["ron"]);

        #[cfg(feature = "json5")]
        formats.insert(NacosFormat::Json5, vec!["json5"]);

        formats
    };
}

impl NacosFormat {
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
            NacosFormat::Toml => toml::parse(uri, text),

            #[cfg(feature = "json")]
            NacosFormat::Json => json::parse(uri, text),

            #[cfg(feature = "yaml")]
            NacosFormat::Yaml => yaml::parse(uri, text),

            #[cfg(feature = "ini")]
            NacosFormat::Ini => ini::parse(uri, text),

            #[cfg(feature = "ron")]
            NacosFormat::Ron => ron::parse(uri, text),

            #[cfg(feature = "json5")]
            NacosFormat::Json5 => json5::parse(uri, text),

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

impl Format for NacosFormat {
    fn parse(
        &self,
        uri: Option<&String>,
        text: &str,
    ) -> Result<Map<String, Value>, Box<dyn Error + Send + Sync>> {
        self.parse(uri, text)
    }
}

impl NacosStoredFormat for NacosFormat {
    fn file_extensions(&self) -> &'static [&'static str] {
        self.extensions()
    }
}
