use api::quantkline::v1::quant_kline_v1_server::{QuantKlineV1, QuantKlineV1Server};
use ecode::{Result};
use hyper::header::{self};
use serde::{Deserialize, Serialize};
use std::{iter::once, net::SocketAddr, time::Duration};
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

use tonic::{Status, async_trait, Request, Response};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Grpc {
    pub address: String,
}

// We make this a separate function so we're able to call it from tests.
pub async fn grpc_serve<T>(
    conf: &Grpc,
    guard: ShutdownGuard,
    service: QuantKlineV1Server<T>,
) -> Result<()>
    where
        T: QuantKlineV1,
{
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
    health_reporter.set_serving::<QuantKlineV1Server<T>>().await;

    // Build and run the server
    let addr = conf.address.parse::<SocketAddr>().unwrap();
    log::info!("Grpc Listening on {}", addr);
    let listener = TcpListener::bind(addr).await?;
    let listener_stream = TcpListenerStream::new(listener);
    let _result = tonic::transport::Server::builder()
        .layer(layer)
        .add_service(health_service)
        .add_service(service)
        .serve_with_incoming_shutdown(listener_stream, async move {
            guard.cancelled().await;
        })
        .await;
    log::info!("Grpc stopping");
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
