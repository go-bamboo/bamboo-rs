use std::{net::SocketAddr, sync::Arc, time::Duration};
use std::convert::Infallible;

use async_trait::async_trait;
use axum::{
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    Router,
    http::Request,
    serve::IncomingStream,
    body::Body,
};
pub use axum::extract::{State, Path, FromRequest};
use serde::{Deserialize, Serialize};
use tokio::{
    net::TcpListener,
    time::sleep,
};
use tokio_graceful::ShutdownGuard;
use tower_http::{
    ServiceBuilderExt,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tower_service::Service;

use bamboo_boot::plugin::Plugin;
use bamboo_status::status::AnyResult;

pub use axum::Json;
pub use axum::routing::{get, post};
pub use axum::debug_handler;

pub mod validate;

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
pub struct Http {
    pub address: String,
}

pub trait Config {
    fn http(&self) -> &Http;
}

pub struct Server<C, S> {
    conf: Arc<C>,
    r: Router<S>,
}

impl<C, S> Server<C, S>
{
    pub fn new(conf: Arc<C>, r: Router<S>) -> Self {
        Self {
            conf,
            r,
        }
    }

    async fn update_in_flight_requests_metric(&self, count: usize) {
        // ...
        // log::info!(" {} requests per 10s", count)
    }

    // fn app(&self) -> Router {
    //     // state
    //
    //     let (in_flight_requests_layer, counter) = InFlightRequestsLayer::pair();
    //
    //     // Spawn a task that will receive the number of in-flight requests every 10 seconds.
    //     // tokio::spawn(
    //     //     counter.run_emitter(Duration::from_secs(10), |count| async move {
    //     //         self.update_in_flight_requests_metric(count).await;
    //     //     }),
    //     // );
    //
    //     // Build our database for holding the key/value pairs
    //     let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();
    //
    //     // Build our middleware stack
    //     let middleware = ServiceBuilder::new()
    //         // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
    //         .sensitive_request_headers(sensitive_headers.clone())
    //         .set_x_request_id(MyMakeRequestId::default())
    //         // Set a timeout
    //         // .layer(TimeoutLayer::new(Duration::from_secs(100)))
    //         // .layer(ValidateRequestHeaderLayer::custom(AcceptBody::new()))
    //         .layer(
    //             TraceLayer::new_for_http()
    //                 .make_span_with(DefaultMakeSpan::new().include_headers(true))
    //                 .on_request(DefaultOnRequest::default())
    //                 .on_body_chunk(DefaultOnBodyChunk::default())
    //                 .on_failure(DefaultOnFailure::default())
    //                 .on_response(
    //                     DefaultOnResponse::new()
    //                         .include_headers(true)
    //                         .latency_unit(LatencyUnit::Micros),
    //                 ),
    //         )
    //         .layer(in_flight_requests_layer)
    //         // Box the response body so it implements `Default` which is required by axum
    //         // .map_response_body(axum::body::boxed)
    //         // Compress responses
    //         .compression()
    //         .propagate_x_request_id()
    //         // Set a `Content-Type` if there isn't one already.
    //         .sensitive_response_headers(sensitive_headers);
    //
    //     // Build route service
    //     Router::new()
    //         .layer(middleware)
    //         .route_service("/", self.r.clone())
    // }
}

#[async_trait]
impl<C, S> Plugin for Server<C, S>
    where C: Config + Send + Sync + 'static,
          S: Service<Request<Body>, Response=Response, Error=Infallible>
          + Clone
          + Send
          + Sync + 'static,
          Router<S>: for<'a> Service<IncomingStream<'a>>,
{
    async fn serve(&self, guard: ShutdownGuard) -> AnyResult<()> {
        let r = Router::new()
            .route("/status", get(status))
            .route("/json", get(json));
        // Create a regular axum app.
        let app = Router::new()
            .route("/slow", get(|| sleep(Duration::from_secs(5))))
            .route("/forever", get(std::future::pending::<()>))
            .merge(r)
            .merge(self.r.clone())
            .layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(Duration::from_secs(10)),
            ));

        // Create a `TcpListener` using tokio.
        let addr = self.conf.http().address.parse::<SocketAddr>()?;
        let listener = TcpListener::bind(&addr).await.unwrap();
        log::info!("Http Listening on {}", addr);

        // Run the server with graceful shutdown
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                guard.cancelled().await
            })
            .await?;
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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(4, 4);
    }

    #[tokio::test]
    async fn valid_serve() {
        assert_eq!(4, 4);
    }
}
