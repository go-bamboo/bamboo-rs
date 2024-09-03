use std::any::Any;
use std::sync::Arc;
use tokio_graceful::Shutdown;
use bamboo_status::status::{Result, AnyResult};
use crate::add;
use crate::plugin::Plugin;

pub struct App {
    plugins: Vec<dyn Plugin>,
}

impl<C, P> App
    where C: Any,
          P: Plugin,
{
    fn new() -> Self {
        Self {
            plugins: Vec::new(),
        }
    }

    fn name(&self) -> &str {
        "App"
    }

    fn with(&mut self, p: P) ->&mut Self {
        self
    }

    pub async fn run(&self, conf: Arc<C>) -> AnyResult<()> {
        let shutdown = Shutdown::default();
        let block_conf = Arc::clone(&conf);
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

    #[test]
    fn it_works() {
        let app = App::new();
        assert_eq!(app.name(), "OK");
    }
}
