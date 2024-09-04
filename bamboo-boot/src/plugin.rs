use std::any::Any;
use std::ops::Deref;
use std::sync::Arc;
use async_trait::async_trait;
use tokio_graceful::ShutdownGuard;
use bamboo_status::status::AnyResult;

#[async_trait]
pub trait Plugin: Any + Send + Sync {
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
    async fn serve(&self, guard: ShutdownGuard) -> AnyResult<()>;
}

#[derive(Clone)]
pub struct PluginRef(Arc<dyn Plugin>);

impl PluginRef {
    pub(crate) fn new<T: Plugin>(plugin: T) -> Self {
        Self(Arc::new(plugin))
    }
}

impl Deref for PluginRef {
    type Target = dyn Plugin;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}