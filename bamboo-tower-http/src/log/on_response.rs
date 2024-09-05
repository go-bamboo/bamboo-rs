use std::time::Duration;
use http::Response;

use log::Level;
use tower_http::{trace::OnResponse, LatencyUnit};
use tracing::Span;

/// The default [`OnResponse`] implementation used by [`Trace`].
///
/// [`Trace`]: super::Trace
#[derive(Clone, Debug)]
pub struct DefaultOnResponse {
    level: Level,
    latency_unit: LatencyUnit,
    include_headers: bool,
}

impl Default for DefaultOnResponse {
    fn default() -> Self {
        Self {
            level: log::Level::Info,
            latency_unit: LatencyUnit::Millis,
            include_headers: false,
        }
    }
}

impl DefaultOnResponse {
    /// Create a new `DefaultOnResponse`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Please note that while this will set the level for the tracing events
    /// themselves, it might cause them to lack expected information, like
    /// request method or path. You can address this using
    /// [`DefaultMakeSpan::level`].
    ///
    /// Defaults to [`Level::DEBUG`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
    /// [`DefaultMakeSpan::level`]: crate::trace::DefaultMakeSpan::level
    pub fn level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the [`LatencyUnit`] latencies will be reported in.
    ///
    /// Defaults to [`LatencyUnit::Millis`].
    pub fn latency_unit(mut self, latency_unit: LatencyUnit) -> Self {
        self.latency_unit = latency_unit;
        self
    }

    /// Include response headers on the [`Event`].
    ///
    /// By default headers are not included.
    ///
    /// [`Event`]: tracing::Event
    pub fn include_headers(mut self, include_headers: bool) -> Self {
        self.include_headers = include_headers;
        self
    }
}

impl<B> OnResponse<B> for DefaultOnResponse {
    fn on_response(self, response: &Response<B>, latency: Duration, _: &Span) {
        let response_headers = self
            .include_headers
            .then(|| tracing::field::debug(response.headers()));

        log::info!(
            " latency: {:?}, {:?}, finished processing request",
            latency,
            response_headers
        );
    }
}

// fn status<B>(res: &Response<B>) -> Option<i32> {
//     use tower_http::classify::grpc_errors_as_failures::ParsedGrpcStatus;

//     // gRPC-over-HTTP2 uses the "application/grpc[+format]" content type, and gRPC-Web uses
//     // "application/grpc-web[+format]" or "application/grpc-web-text[+format]", where "format" is
//     // the message format, e.g. +proto, +json.
//     //
//     // So, valid grpc content types include (but are not limited to):
//     //  - application/grpc
//     //  - application/grpc+proto
//     //  - application/grpc-web+proto
//     //  - application/grpc-web-text+proto
//     //
//     // For simplicity, we simply check that the content type starts with "application/grpc".
//     let is_grpc = res
//         .headers()
//         .get(http::header::CONTENT_TYPE)
//         .map_or(false, |value| {
//             value.as_bytes().starts_with("application/grpc".as_bytes())
//         });

//     if is_grpc {
//         match classify::grpc_errors_as_failures::classify_grpc_metadata(
//             res.headers(),
//             classify::GrpcCode::Ok.into_bitmask(),
//         ) {
//             ParsedGrpcStatus::Success
//             | ParsedGrpcStatus::HeaderNotString
//             | ParsedGrpcStatus::HeaderNotInt => Some(0),
//             ParsedGrpcStatus::NonSuccess(status) => Some(status.get()),
//             // if `grpc-status` is missing then its a streaming response and there is no status
//             // _yet_, so its neither success nor error
//             ParsedGrpcStatus::GrpcStatusHeaderMissing => None,
//         }
//     } else {
//         Some(res.status().as_u16().into())
//     }
// }
