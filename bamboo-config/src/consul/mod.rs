mod format;
pub mod source;

use std::fmt::Debug;

use crate::{ConfigError, map::Map, format::Format, source::Source, value::Value};
use url::Url;

use self::format::ConsulFormat;
use self::source::ConsulSource;

/// A configuration source backed up by a nacos.
///
/// It supports optional automatic file format discovery.
#[derive(Clone, Debug)]
#[must_use]
pub struct Consul<T, F> {
    source: T,

    /// Format of file (which dictates what driver to use).
    format: Option<F>,

    /// A required File will error if it cannot be found
    required: bool,
}

/// An extension of [`Format`](crate::Format) trait.
///
/// Associates format with file extensions, therefore linking storage-agnostic notion of format to a file system.
pub trait ConsulStoredFormat: Format {
    /// Returns a vector of file extensions, for instance `[yml, yaml]`.
    fn file_extensions(&self) -> &'static [&'static str];
}

impl<F> Consul<source::remote::Remote, F>
    where
        F: ConsulStoredFormat + 'static,
{
    pub fn new(name: &str, format: F) -> Self {
        Self {
            format: Some(format),
            required: true,
            source: source::remote::Remote::new(Url::parse(name).unwrap()),
        }
    }
}

impl Consul<source::remote::Remote, ConsulFormat> {
    /// Given the basename of a file, will attempt to locate a file by setting its
    /// extension to a registered format.
    pub fn with_name(name: &str) -> Self {
        Self {
            format: None,
            required: true,
            source: source::remote::Remote::new(Url::parse(name).unwrap()),
        }
    }
}


impl<T, F> Consul<T, F>
    where
        F: ConsulStoredFormat + 'static,
        T: ConsulSource<F>,
{
    pub fn format(mut self, format: F) -> Self {
        self.format = Some(format);
        self
    }

    /// Set required to false to make a file optional when building the config.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }
}

impl<T, F> Source for Consul<T, F>
    where
        F: ConsulStoredFormat + Debug + Clone + Send + Sync + 'static,
        T: Sync + Send + ConsulSource<F> + 'static,
{
    fn clone_into_box(&self) -> Box<dyn Source + Send + Sync> {
        Box::new((*self).clone())
    }

    fn collect(&self) -> Result<Map<String, Value>, ConfigError> {
        // Coerce the file contents to a string
        let (uri, contents, format) = match self
            .source
            .resolve(self.format.clone())
            .map_err(ConfigError::Foreign)
        {
            Ok(result) => (result.uri, result.content, result.format),

            Err(error) => {
                if !self.required {
                    return Ok(Map::new());
                }

                return Err(error);
            }
        };

        // Parse the string using the given format
        format
            .parse(uri.as_ref(), &contents)
            .map_err(|cause| ConfigError::FileParse { uri, cause })
    }
}
