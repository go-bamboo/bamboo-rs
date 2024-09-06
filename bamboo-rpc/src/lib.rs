use bamboo_status::status::{Result, AnyResult};
use hyper::header::{self};
use serde::{Deserialize, Serialize};
use std::{iter::once, net::SocketAddr, time::Duration};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_graceful::ShutdownGuard;
use tokio_stream::wrappers::TcpListenerStream;
use tower::ServiceBuilder;
use tower_http::{
    classify::{GrpcCode, GrpcErrorsAsFailures, SharedClassifier},
    compression::CompressionLayer,
    sensitive_headers::SetSensitiveHeadersLayer,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tonic::{
    Status, async_trait, Request, Response,
    body::BoxBody,
    server::NamedService,
    transport::Body,
};
use tower_service::Service;
use bamboo_boot::plugin::Plugin;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Grpc {
    pub address: String,
}

pub trait Config {
    fn grpc(&self) -> &Grpc;
}

pub struct Server<C, S> {
    conf: Arc<C>,
    s: S,
}

impl<C, S> Server<C, S> where
    S: Service<Request<Body>, Response=Response<BoxBody>, Error=Infallible>
    + NamedService
    + Clone
    + Send
    + 'static,
    S::Future: Send + 'static,
{
    fn new(conf: Arc<C>, s: S) -> Self {
        Self {
            conf,
            s,
        }
    }
}

impl<C, S> Plugin for Server<C, S> where
    C: Config + Send + Sync + 'static,
    S: Service<Request<Body>, Response=Response<BoxBody>, Error=Infallible>
    + NamedService
    + Clone
    + Send
    + Sync
    + 'static,
    S::Future: Send + 'static,
{
    async fn serve(&self, guard: ShutdownGuard) -> AnyResult<()> {
        // let s = ServerImpl::new()?;
        // let service = QuantServer::new(s);
        // Response classifier that doesn't consider `Ok`, `Invalid Argument`, or `Not Found` as
        // failures
        let classifier = GrpcErrorsAsFailures::new()
            .with_success(GrpcCode::InvalidArgument)
            .with_success(GrpcCode::NotFound);

        // Build our middleware stack
        let layer = ServiceBuilder::new()
            // Set a timeout
            .timeout(Duration::from_secs(10))
            // Compress responses
            .layer(CompressionLayer::new())
            // Mark the `Authorization` header as sensitive so it doesn't show in logs
            .layer(SetSensitiveHeadersLayer::new(once(header::AUTHORIZATION)))
            // Log all requests and responses
            .layer(
                TraceLayer::new(SharedClassifier::new(classifier))
                    .make_span_with(DefaultMakeSpan::new().include_headers(true)),
            )
            .into_inner();

        let (mut health_reporter, health_service) = tonic_health::server::health_reporter();
        health_reporter.set_serving::<S>().await;

        // Build and run the server
        let addr = self.conf.grpc().address.parse::<SocketAddr>().unwrap();
        log::info!("Grpc Listening on {}", addr);
        let listener = TcpListener::bind(addr).await?;
        let listener_stream = TcpListenerStream::new(listener);
        let _result = tonic::transport::Server::builder()
            .layer(layer)
            .add_service(health_service)
            .add_service(self.s.clone())
            .serve_with_incoming_shutdown(listener_stream, async move {
                guard.cancelled().await;
            })
            .await;
        log::info!("Grpc stopping");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }

    #[tokio::test]
    async fn valid_send_tran() {
        assert_eq!(4, 4);
    }
}
