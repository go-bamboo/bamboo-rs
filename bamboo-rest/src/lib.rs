use async_trait::async_trait;
use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use ecode::{Result};
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tokio_graceful::ShutdownGuard;
use tower::ServiceBuilder;
use tower_http::{timeout::TimeoutLayer, ServiceBuilderExt};

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

pub async fn http_serve<T>(conf: &Http, guard: ShutdownGuard, state: T) -> Result<()>
    where
        T: AppState + Clone + Send + Sync + 'static,
{
    // Run our service
    let addr = conf.address.parse::<SocketAddr>()?;
    log::info!("Http Listening on {}", addr);

    // state
    // let state = AppState::new(Arc::clone(&conf)).await.unwrap();
    axum::Server::bind(&addr)
        .serve(app(state).into_make_service())
        .with_graceful_shutdown(async move {
            guard.cancelled().await;
        })
        .await?;
    log::info!("Http stopping");
    Ok(())
}

fn app<T>(state: T) -> Router
    where
        T: AppState + Clone + Send + Sync + 'static,
{
    // Build our database for holding the key/value pairs
    let sensitive_headers: Arc<[_]> = vec![header::AUTHORIZATION, header::COOKIE].into();

    // Build our middleware stack
    let middleware = ServiceBuilder::new()
        // Mark the `Authorization` and `Cookie` headers as sensitive so it doesn't show in logs
        .sensitive_request_headers(sensitive_headers.clone())
        // Add high level tracing/logging to all requests
        // .layer(
        //     TraceLayer::new_for_http()
        //         .on_body_chunk(|chunk: &Bytes, latency: Duration, _: &tracing::Span| {
        //             tracing::trace!(size_bytes = chunk.len(), latency = ?latency, "sending body chunk")
        //         })
        //         .make_span_with(DefaultMakeSpan::new().include_headers(true))
        //         .on_response(DefaultOnResponse::new().include_headers(true).latency_unit(LatencyUnit::Micros)),
        // )
        .sensitive_response_headers(sensitive_headers)
        // Set a timeout
        .layer(TimeoutLayer::new(Duration::from_secs(10)))
        // Box the response body so it implements `Default` which is required by axum
        .map_response_body(axum::body::boxed)
        // Compress responses
        .compression()
        // Set a `Content-Type` if there isn't one already.
        .insert_response_header_if_not_present(
            header::CONTENT_TYPE,
            HeaderValue::from_static("application/octet-stream"),
        );

    // Build route service
    Router::new()
        .route("/status", get(status))
        .route("/json", get(json))
        .route("/hello", get(hello::<T>))
        .route("/tick", get(tick::<T>))
        .route("/api/v1/:key", get(get_key::<T>).post(set_key::<T>))
        .layer(middleware)
        .with_state(state)
}

// `StatusCode` gives an empty response with that status code
async fn status() -> StatusCode {
    StatusCode::NOT_FOUND
}

async fn json() -> Json<Vec<String>> {
    Json(vec!["foo".to_owned(), "bar".to_owned()])
}

async fn hello<T>(_state: State<T>) -> impl IntoResponse
    where
        T: AppState + Clone + Send + Sync + 'static,
{
    // info!("Hello World");
    // Ok(Bytes::from("World"))
    let info = Address {
        // message: "World".to_string(),
        street: "HuaYang".to_string(),
        city: "ChengDu".to_string(),
    };
    Json(info)
}

async fn tick<T>(State(mut state): State<T>) -> impl IntoResponse
    where
        T: AppState + Clone + Send + Sync + 'static,
{
    if let Err(e) = state.tick().await {
        log::error!("err {}", e)
    }
    log::info!("Hello World");
    let info = HelloResponse {
        message: "World".to_string(),
    };
    Json(info)
}

async fn get_key<T>(_path: Path<String>, _state: State<T>) -> impl IntoResponse
    where
        T: AppState + Clone + Send + Sync + 'static,
{
    // let state = state.db.read().await;
    // if let Some(value) = state.get(&*path).cloned() {
    //     tracing::info!("Hello World");
    // } else {
    //     tracing::error!("error");
    // }
    let info = HelloResponse {
        message: "World".to_string(),
    };
    Json(info)
}

async fn set_key<T>(Path(_path): Path<String>, _state: State<T>, _value: Bytes)
    where
        T: AppState + Clone + Send + Sync + 'static,
{
    // let mut state = state.db.write().await;
    // state.insert(path, value);
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
