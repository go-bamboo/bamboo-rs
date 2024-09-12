use tower_layer::Layer;
use crate::simple::service::MyMiddleware;

#[derive(Debug, Clone, Default)]
struct MyMiddlewareLayer {}

impl<S> Layer<S> for MyMiddlewareLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyMiddleware { inner: service }
    }
}
