pub mod remote;

use std::error::Error;
use std::fmt::Debug;

use crate::{nacos::NacosStoredFormat, format::Format};

/// Describes where the file is sourced
pub trait NacosSource<T>: Debug + Clone
    where
        T: Format + NacosStoredFormat,
{
    fn resolve(
        &self,
        format_hint: Option<T>,
    ) -> Result<NacosSourceResult, Box<dyn Error + Send + Sync>>;
}

pub struct NacosSourceResult {
    pub(crate) uri: Option<String>,
    pub(crate) content: String,
    pub(crate) format: Box<dyn Format>,
}

impl NacosSourceResult {
    pub fn uri(&self) -> &Option<String> {
        &self.uri
    }

    pub fn content(&self) -> &str {
        self.content.as_str()
    }

    pub fn format(&self) -> &dyn Format {
        self.format.as_ref()
    }
}
