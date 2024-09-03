use std::sync::Arc;
use tokio_graceful::ShutdownGuard;
use bamboo_status::status::AnyResult;

pub trait Plugin {
    async fn http_serve(guard: ShutdownGuard) -> AnyResult<()>;
}