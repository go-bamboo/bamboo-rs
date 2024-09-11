use std::any::Any;
use std::sync::Arc;

use dashmap::DashMap;
use tokio_graceful::Shutdown;

use bamboo_status::status::{AnyResult, Result};

use crate::plugin::{Plugin, PluginRef};

pub type Registry<T> = DashMap<String, T>;

pub struct App<C> {
    conf : Arc<C>,
    components: Registry<PluginRef>,
}

impl<C> App<C>
{
    pub fn new(conf: Arc<C>) -> Self {
        Self {
            conf,
            components: Registry::new(),
        }
    }

    pub fn with(&self, p: PluginRef) ->Result<()> {
        self.components.insert(p.name().to_string(), p);
        Ok(())
    }

    pub async fn run(&self) -> AnyResult<()> {
        let shutdown = Shutdown::default();

        let it = self.components.iter();
        for kv in it {
            let block_conf = Arc::clone(&self.conf);
            let t = kv.value().clone();
            let _ = shutdown.spawn_task_fn(|guard: tokio_graceful::ShutdownGuard| async move {
                let _ = t.serve(guard).await;
            });
        }

        shutdown.shutdown().await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Config {}

    impl Config {
        fn new()->Self {
            Self{}
        }
    }

    #[test]
    fn it_works() {
        let app = App::new(Arc::new(Config::new()));
        assert_eq!(app.name(), "App");
    }
}
