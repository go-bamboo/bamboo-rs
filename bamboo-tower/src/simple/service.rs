use tower_service::Service;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

type BoxFuture<'a, T> = Pin<Box<dyn std::future::Future<Output=T> + Send + 'a>>;

#[derive(Debug, Clone)]
pub struct MyMiddleware<S> {
    pub inner: S,
}

impl<S, Request> Service<Request> for MyMiddleware<S>
where
    S: Service<Request> + Clone,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            // Do extra async work here...
            let response = inner.call(req).await?;

            Ok(response)
        })
    }
}