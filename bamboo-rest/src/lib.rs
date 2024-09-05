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
use tokio_graceful::ShutdownGuard;
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, ServiceBuilderExt};
use tower_http::metrics::InFlightRequestsLayer;
use tower_http::trace::TraceLayer;
use bamboo_boot::plugin::Plugin;

#[async_trait]
pub trait AppState {
    async fn open_eth_order(&self) -> Result<()>;
    async fn tick(&self) -> Result<()>;
    async fn tick_dta(&mut self) -> Result<()>;
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Http {
    pub address: String,
}

pub trait Config {
    fn http(&self) -> &Http;
}

pub struct Server<C, R> {
    conf: Arc<C>,
    r: R,
}

impl <C, R> Server<C, R> {
    fn new(conf: Arc<C>, r: R) ->Self {
        Self {
            conf,
            r
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
            .route("/status", get(status))
            .route("/chain/createAddressByMnemonic", post(create_address_by_mnemonic))
            .route("/chain/createAddressByMnemonic1", post(create_address_by_mnemonic1))
            .route("/chain/getBalance", post(get_balance))
            .route("/chain/free/:master/:gas_add", post(miner_fee1))
            .route(
                "/chain/createAssociatedAccount",
                post(create_associated_account),
            )
            .route("/chain/transferTo", post(transfer_to))
            // .route("/chain/getLastBlockHeight", post(get_last_block_height))
            // .route(
            //     "/chain/getBlockHashByHeight/:height",
            //     post(get_block_hash_by_height),
            // )
            // .route(
            //     "/chain/getBlockHashByHeight1/:height",
            //     post(get_block_hash_by_height1),
            // )
            .layer(middleware)
            .with_state(state)
    }
}

impl<C, R> Plugin for Server<C, R>
    where C: Config
{
    async fn serve(&self, guard: ShutdownGuard) -> AnyResult<()> {


        // Create a regular axum app.
        let app = app(state);

        // Run our service
        let addr = conf.server.http.address.parse::<SocketAddr>()?;
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
    async fn valid_send_tran() {

    }
}
