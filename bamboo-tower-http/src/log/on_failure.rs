use std::{fmt, time::Duration};

use log::Level;
use tower_http::{trace::OnFailure, LatencyUnit};
use tracing::Span;

/// The default [`OnFailure`] implementation used by [`Trace`].
///
/// [`Trace`]: super::Trace
#[derive(Clone, Debug)]
pub struct DefaultOnFailure {
    level: Level,
    latency_unit: LatencyUnit,
}

impl Default for DefaultOnFailure {
    fn default() -> Self {
        Self {
            level: log::Level::Info,
            latency_unit: LatencyUnit::Millis,
        }
    }
}

impl DefaultOnFailure {
    /// Create a new `DefaultOnFailure`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the [`Level`] used for [tracing events].
    ///
    /// Defaults to [`Level::ERROR`].
    ///
    /// [tracing events]: https://docs.rs/tracing/latest/tracing/#events
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
}

impl<FailureClass> OnFailure<FailureClass> for DefaultOnFailure
    where FailureClass: fmt::Display,
{
    fn on_failure(&mut self, failure_classification: FailureClass, latency: Duration, _: &Span) {
        log::error!(
            "{}, latency: {:?}, response failed",
           failure_classification,
            latency,
        );
    }
}
