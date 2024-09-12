use std::time::Duration;

use bytes::Bytes;
use tower_http::trace::OnBodyChunk;
use tracing::Span;

/// The default [`OnBodyChunk`] implementation used by [`Trace`].
///
/// Simply does nothing.
///
/// [`Trace`]: super::Trace
#[derive(Debug, Default, Clone)]
pub struct DefaultOnBodyChunk {
    _priv: (),
}

impl DefaultOnBodyChunk {
    /// Create a new `DefaultOnBodyChunk`.
    pub fn new() -> Self {
        Self { _priv: () }
    }
}

impl OnBodyChunk<Bytes> for DefaultOnBodyChunk {
    #[inline]
    fn on_body_chunk(&mut self, _chunk: &Bytes, _latency: Duration, _: &Span) {
        // log::info!(
        //     "size_bytes = {}, latency = {}ms",
        //     chunk.len(),
        //     latency.as_millis()
        // )
    }
}
