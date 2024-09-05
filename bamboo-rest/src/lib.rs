pub mod validate;

use async_trait::async_trait;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use bamboo_status::status::{AnyResult, Result};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use axum::http::Request;
use axum::routing::post;
use hyper::body::Incoming;
use hyper_util::rt::TokioIo;
use tokio::sync::watch;
use tokio_graceful::ShutdownGuard;
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, ServiceBuilderExt, LatencyUnit};
use tower_http::metrics::InFlightRequestsLayer;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tower_service::Service;
use bamboo_boot::plugin::Plugin;
use bamboo_tower_http::{request_id::MyMakeRequestId,
                        log::{
                            on_request::DefaultOnRequest,
                            on_body_chunk::DefaultOnBodyChunk,
                            on_failure::DefaultOnFailure,
                            on_response::DefaultOnResponse,
                        },
};


#[async_trait]
pub trait AppState {
    // async fn route(&self, Router<S>) -> Result<()>;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Http {
    pub address: String,
}

pub trait Config {
    fn http(&self) -> &Http;
}

pub struct Server<C, R, S> {
    conf: Arc<C>,
    r: R,
    s: S,
}

impl<C, R, S> Server<C, R, S>
    where S: Clone
{
    fn new(conf: Arc<C>, r: R, s: S) -> Self {
        Self {
            conf,
            r,
            s,
        }
    }

    async fn update_in_flight_requests_metric(&self, count: usize) {
        // ...
        // log::info!(" {} requests per 10s", count)
    }

    fn app(&self) -> Router {
        // state

        let (in_flight_requests_layer, counter) = InFlightRequestsLayer::pair();

        // Spawn a task that will receive the number of in-flight requests every 10 seconds.
        tokio::spawn(
            counter.run_emitter(Duration::from_secs(10), |count| async move {
                self.update_in_flight_requests_metric(count).await;
            }),
        );

        // Build our database for holding the key/value pairs
        let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();

        // Build our middleware stack
        let middleware = ServiceBuilder::new()
            // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
            .sensitive_request_headers(sensitive_headers.clone())
            .set_x_request_id(MyMakeRequestId::default())
            // Set a timeout
            // .layer(TimeoutLayer::new(Duration::from_secs(100)))
            // .layer(ValidateRequestHeaderLayer::custom(AcceptBody::new()))
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::default())
                    .on_body_chunk(DefaultOnBodyChunk::default())
                    .on_failure(DefaultOnFailure::default())
                    .on_response(
                        DefaultOnResponse::new()
                            .include_headers(true)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(in_flight_requests_layer)
            // Box the response body so it implements `Default` which is required by axum
            // .map_response_body(axum::body::boxed)
            // Compress responses
            .compression()
            .propagate_x_request_id()
            // Set a `Content-Type` if there isn't one already.
            .sensitive_response_headers(sensitive_headers);

        // Build route service
        Router::new()
            .layer(middleware)
            .with_state(self.s.clone())
    }
}

impl<C, R, S> Plugin for Server<C, R, S>
    where C: Config + Send + Sync + 'static,
          R: Send + Sync + 'static,
          S: Send + Sync + 'static
{
    async fn serve(&self, guard: ShutdownGuard) -> AnyResult<()> {


        // Create a regular axum app.
        let app = self.app();

        // Run our service
        let addr = self.conf.http().address.parse::<SocketAddr>()?;
        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        log::info!("Http Listening on {}", addr);

        // Create a watch channel to track tasks that are handling connections and wait for them to
        // complete.
        let (close_tx, close_rx) = watch::channel(());
        let signal1 = guard.clone();
        // Continuously accept new connections.
        loop {
            let (socket, remote_addr) = tokio::select! {
            // Either accept a new connection...
            result = listener.accept() => {
                result.unwrap()
            }
            // ...or wait to receive a shutdown signal and stop the accept loop.
            _ = signal1.cancelled() => {
                log::info!("signal received, not accepting new connections");
                break;
            }
        };

            log::debug!("connection {remote_addr} accepted");

            // We don't need to call `poll_ready` because `Router` is always ready.
            let tower_service = app.clone();

            // Clone the watch receiver and move it into the task.
            let close_rx = close_rx.clone();

            // Spawn a task to handle the connection. That way we can serve multiple connections
            // concurrently.
            let signal2 = guard.clone();
            tokio::spawn(async move {
                // Hyper has its own `AsyncRead` and `AsyncWrite` traits and doesn't use tokio.
                // `TokioIo` converts between them.
                let socket = TokioIo::new(socket);

                // Hyper also has its own `Service` trait and doesn't use tower. We can use
                // `hyper::service::service_fn` to create a hyper `Service` that calls our app through
                // `tower::Service::call`.
                let hyper_service = hyper::service::service_fn(move |request: Request<Incoming>| {
                    // We have to clone `tower_service` because hyper's `Service` uses `&self` whereas
                    // tower's `Service` requires `&mut self`.
                    //
                    // We don't need to call `poll_ready` since `Router` is always ready.
                    tower_service.clone().call(request)
                });

                // `hyper_util::server::conn::auto::Builder` supports both http1 and http2 but doesn't
                // support graceful so we have to use hyper directly and unfortunately pick between
                // http1 and http2.
                let conn = hyper::server::conn::http1::Builder::new()
                    .serve_connection(socket, hyper_service)
                    // `with_upgrades` is required for websockets.
                    .with_upgrades();

                // `graceful_shutdown` requires a pinned connection.
                let mut conn = std::pin::pin!(conn);

                loop {
                    tokio::select! {
                    // Poll the connection. This completes when the client has closed the
                    // connection, graceful shutdown has completed, or we encounter a TCP error.
                    result = conn.as_mut() => {
                        if let Err(err) = result {
                            log::error!("failed to serve connection: {err:#}");
                        }
                        break;
                    }
                    // Start graceful shutdown when we receive a shutdown signal.
                    //
                    // We use a loop to continue polling the connection to allow requests to finish
                    // after starting graceful shutdown. Our `Router` has `TimeoutLayer` so
                    // requests will finish after at most 10 seconds.
                    _ = signal2.cancelled() => {
                        log::info!("signal received, http connection starting graceful shutdown");
                        conn.as_mut().graceful_shutdown();
                    }
                }
                }

                log::debug!("connection {remote_addr} closed");

                // Drop the watch receiver to signal to `main` that this task is done.
                drop(close_rx);
            });
        }

        // We only care about the watch receivers that were moved into the tasks so close the residual
        // receiver.
        drop(close_rx);

        // Close the listener to stop accepting new connections.
        drop(listener);

        // Wait for all tasks to complete.
        log::info!("waiting for {} tasks to finish", close_tx.receiver_count());
        close_tx.closed().await;

        log::info!("Http stopping");
        Ok(())
    }
}


// `StatusCode` gives an empty response with that status code
async fn status() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn json() -> Json<Vec<String>> {
    Json(vec!["foo".to_owned(), "bar".to_owned()])
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

    #[tokio::test]
    async fn valid_send_tran() {}
}
