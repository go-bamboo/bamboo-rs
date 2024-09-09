pub mod remote;

use std::error::Error;
use std::fmt::Debug;

use crate::{consul::ConsulStoredFormat, format::Format};

/// Describes where the consul is sourced
pub trait ConsulSource<T>: Debug + Clone
    where
        T: Format + ConsulStoredFormat,
{
    fn resolve(
        &self,
        format_hint: Option<T>,
    ) -> Result<ConsulSourceResult, Box<dyn Error + Send + Sync>>;
}

pub struct ConsulSourceResult {
    pub(crate) uri: Option<String>,
    pub(crate) content: String,
    pub(crate) format: Box<dyn Format>,
}

impl ConsulSourceResult {
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
