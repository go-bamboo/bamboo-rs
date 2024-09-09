use std::error::Error;
use std::path::PathBuf;

use crate::nacos::{
    Format, NacosSource, NacosStoredFormat, source::NacosSourceResult,
};

/// Describes a file sourced from a file
#[derive(Clone, Debug)]
pub struct Remote {
    /// Path of configuration file
    u: url::Url,
}

impl Remote {
    pub fn new(u: url::Url) -> Self {
        Self { u }
    }
}

impl<F> NacosSource<F> for Remote
where
    F: Format + NacosStoredFormat + 'static,
{
    fn resolve(
        &self,
        format_hint: Option<F>,
    ) -> Result<NacosSourceResult, Box<dyn Error + Send + Sync>> {
        // Find file
        Err("".into())
    }
}

fn add_dummy_extension(mut filename: PathBuf) -> PathBuf {
    match filename.extension() {
        Some(extension) => {
            let mut ext = extension.to_os_string();
            ext.push(".");
            ext.push("dummy");
            filename.set_extension(ext);
        }
        None => {
            filename.set_extension("dummy");
        }
    }
    filename
}
