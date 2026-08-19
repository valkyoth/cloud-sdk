//! Server metrics query models.

use super::shared::encode_server_query;
use super::{ServerEndpoint, ServerId, ServerRequestError, TimestampValue};

/// Server metric type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ServerMetricType {
    /// CPU utilization.
    Cpu,
    /// Disk throughput.
    Disk,
    /// Network throughput.
    Network,
}

/// Non-empty metric selection with duplicates impossible by construction.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerMetricTypes(u8);

impl ServerMetricTypes {
    /// Starts a selection with one metric.
    #[must_use]
    pub const fn new(metric: ServerMetricType) -> Self {
        Self(metric_bit(metric))
    }

    /// Adds a metric idempotently.
    #[must_use]
    pub const fn with(mut self, metric: ServerMetricType) -> Self {
        self.0 |= metric_bit(metric);
        self
    }

    /// Returns the canonical API value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self.0 {
            1 => "cpu",
            2 => "disk",
            3 => "cpu,disk",
            4 => "network",
            5 => "cpu,network",
            6 => "disk,network",
            7 => "cpu,disk,network",
            _ => "",
        }
    }
}

const fn metric_bit(metric: ServerMetricType) -> u8 {
    match metric {
        ServerMetricType::Cpu => 1,
        ServerMetricType::Disk => 2,
        ServerMetricType::Network => 4,
    }
}

/// Positive metrics resolution in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerMetricsStep(u32);

/// Invalid Server metrics resolution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerMetricsStepError;

impl core::fmt::Display for ServerMetricsStepError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("server metrics step is invalid")
    }
}

impl core::error::Error for ServerMetricsStepError {}

impl ServerMetricsStep {
    /// Creates a positive resolution.
    pub const fn new(seconds: u32) -> Result<Self, ServerMetricsStepError> {
        if seconds == 0 {
            return Err(ServerMetricsStepError);
        }
        Ok(Self(seconds))
    }

    /// Returns the resolution in seconds.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Server metrics request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ServerMetricsRequest<'a> {
    id: ServerId,
    metric_types: ServerMetricTypes,
    start: TimestampValue<'a>,
    end: TimestampValue<'a>,
    step: Option<ServerMetricsStep>,
}

impl<'a> ServerMetricsRequest<'a> {
    /// Creates a request with a non-empty metric set and increasing UTC range.
    pub fn try_new(
        id: ServerId,
        metric_types: ServerMetricTypes,
        start: TimestampValue<'a>,
        end: TimestampValue<'a>,
    ) -> Result<Self, ServerRequestError> {
        if start.as_str() >= end.as_str() {
            return Err(ServerRequestError::InvalidTimeRange);
        }
        Ok(Self {
            id,
            metric_types,
            start,
            end,
            step: None,
        })
    }

    /// Sets the optional resolution.
    #[must_use]
    pub const fn with_step(mut self, step: ServerMetricsStep) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ServerEndpoint {
        ServerEndpoint::Metrics(self.id)
    }

    /// Writes the deterministic query string.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, ServerRequestError> {
        encode_server_query(output, |writer, first| {
            writer.query_pair(first, "end", self.end.as_str())?;
            if let Some(step) = self.step {
                writer.query_u64(first, "step", u64::from(step.get()))?;
            }
            writer.query_pair(first, "start", self.start.as_str())?;
            writer.query_pair(first, "type", self.metric_types.as_api_str())
        })
    }
}
