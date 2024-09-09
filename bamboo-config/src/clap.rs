pub use clap::Parser;
use crate::{
    error::Result,
    config::Config,
    file::File,
    env::Environment,
    nacos::Nacos,
};


#[derive(Debug, Parser)]
pub struct Flag {
    /// The port to listen on
    #[clap(short = 'c', long, default_value = "./configs/dev.yaml")]
    pub conf: String,
}

pub fn load<'de, T>(path: &str) -> Result<T>
    where
        T: serde::Deserialize<'de>,
{
    let settings = Config::builder()
        // Add in `./Settings.toml`
        .add_source(File::with_name(&path))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    let bootstrap = settings.try_deserialize::<T>()?;
    Ok(bootstrap)
}

pub fn load_env<'de, T>(path: &str) -> Result<T>
    where
        T: serde::Deserialize<'de>,
{
    let settings = Config::builder()
        // Add in settings from the environment (with a prefix of APP)
        // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
        .add_source(Environment::with_prefix("APP"))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    let bootstrap = settings.try_deserialize::<T>()?;
    Ok(bootstrap)
}

pub fn load_nacos<'de, T>(path: &str) -> Result<T>
    where
        T: serde::Deserialize<'de>,
{
    let settings = Config::builder()
        .add_source(Nacos::with_name("APP"))
        .build()
        .unwrap();

    // Print out our settings (as a HashMap)
    let bootstrap = settings.try_deserialize::<T>()?;
    Ok(bootstrap)
}



pub fn watch<'de, T>(bootstrap: &T) -> Result<()> {
    Ok(())
}

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
