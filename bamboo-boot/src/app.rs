use std::any::Any;
use std::sync::Arc;
use dashmap::DashMap;
use tokio_graceful::Shutdown;
use bamboo_status::status::{Result, AnyResult};
use crate::component::ComponentRef;
use crate::plugin::Plugin;

pub type Registry<T> = DashMap<String, T>;

pub struct App<C> {
    conf : Arc<C>,
    components: Registry<ComponentRef>,
}

impl<C> App<C>
{
    fn new(conf: Arc<C>) -> Self {
        Self {
            conf,
            components: Registry::new(),
        }
    }

    fn name(&self) -> &str {
        "App"
    }

    fn with(&mut self, p: ComponentRef) ->&mut Self {
        self
    }

    pub async fn run(&self) -> AnyResult<()> {
        let shutdown = Shutdown::default();


        let it = self.components.iter();
        for val in it {
            let block_conf = Arc::clone(&self.conf);
            let _ = shutdown.spawn_task_fn(|guard: tokio_graceful::ShutdownGuard| async move {
                // sol_serve(http_conf, guard, block1).await
            });
        }


        // let http_block = Arc::new(Block::new(block_conf)?);
        // let grpc_block = Arc::clone(&http_block);

        // solws
        // let http_conf = Arc::clone(&conf);
        // let block1 = Arc::clone(&block);
        // let _ = shutdown.spawn_task_fn(|guard: tokio_graceful::ShutdownGuard| async move {
        //     sol_serve(http_conf, guard, block1).await
        // });

        // http
        // let http_conf = Arc::clone(&conf);
        // let _ = shutdown.spawn_task_fn(|guard: tokio_graceful::ShutdownGuard| async move {
        //     http_serve(http_conf, guard, http_block).await
        // });

        // grpc
        // let grpc_conf = Arc::clone(&conf);
        // let _ = shutdown.spawn_task_fn(|guard: tokio_graceful::ShutdownGuard| async move {
        //     grpc_serve(grpc_conf, guard, grpc_block).await
        // });

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
