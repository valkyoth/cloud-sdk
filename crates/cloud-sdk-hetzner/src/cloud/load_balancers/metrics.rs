//! Load Balancer metrics query models.

use crate::cloud::shared::CloudQueryWriter;

use super::{LoadBalancerEndpoint, LoadBalancerId, LoadBalancerRequestError};

/// Load Balancer metric type.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LoadBalancerMetricType {
    /// Current open connections.
    OpenConnections,
    /// Connections established per second.
    ConnectionsPerSecond,
    /// Requests handled per second.
    RequestsPerSecond,
    /// Inbound and outbound bandwidth.
    Bandwidth,
}

/// Non-empty, duplicate-free metric selection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerMetricTypes(u8);

impl LoadBalancerMetricTypes {
    /// Starts a selection with one metric.
    #[must_use]
    pub const fn new(metric: LoadBalancerMetricType) -> Self {
        Self(metric_bit(metric))
    }

    /// Adds a metric idempotently.
    #[must_use]
    pub const fn with(mut self, metric: LoadBalancerMetricType) -> Self {
        self.0 |= metric_bit(metric);
        self
    }

    /// Returns the canonical comma-separated API value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self.0 {
            1 => "open_connections",
            2 => "connections_per_second",
            3 => "open_connections,connections_per_second",
            4 => "requests_per_second",
            5 => "open_connections,requests_per_second",
            6 => "connections_per_second,requests_per_second",
            7 => "open_connections,connections_per_second,requests_per_second",
            8 => "bandwidth",
            9 => "open_connections,bandwidth",
            10 => "connections_per_second,bandwidth",
            11 => "open_connections,connections_per_second,bandwidth",
            12 => "requests_per_second,bandwidth",
            13 => "open_connections,requests_per_second,bandwidth",
            14 => "connections_per_second,requests_per_second,bandwidth",
            15 => "open_connections,connections_per_second,requests_per_second,bandwidth",
            _ => unreachable_metric_selection(),
        }
    }
}

const fn metric_bit(metric: LoadBalancerMetricType) -> u8 {
    match metric {
        LoadBalancerMetricType::OpenConnections => 1,
        LoadBalancerMetricType::ConnectionsPerSecond => 2,
        LoadBalancerMetricType::RequestsPerSecond => 4,
        LoadBalancerMetricType::Bandwidth => 8,
    }
}

const fn unreachable_metric_selection() -> &'static str {
    // The private field is created only through `new`, which always sets one valid bit.
    ""
}

/// Validated UTC RFC3339 timestamp used for deterministic range ordering.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerTimestamp<'a>(&'a str);

impl<'a> LoadBalancerTimestamp<'a> {
    /// Validates `YYYY-MM-DDTHH:MM:SSZ`, including calendar bounds.
    pub fn new(value: &'a str) -> Result<Self, LoadBalancerRequestError> {
        let bytes = value.as_bytes();
        if bytes.len() != 20
            || !digits(bytes, 0, 4)
            || bytes.get(4) != Some(&b'-')
            || !digits(bytes, 5, 7)
            || bytes.get(7) != Some(&b'-')
            || !digits(bytes, 8, 10)
            || bytes.get(10) != Some(&b'T')
            || !digits(bytes, 11, 13)
            || bytes.get(13) != Some(&b':')
            || !digits(bytes, 14, 16)
            || bytes.get(16) != Some(&b':')
            || !digits(bytes, 17, 19)
            || bytes.get(19) != Some(&b'Z')
        {
            return Err(LoadBalancerRequestError::InvalidText);
        }
        let Some(year) = number(bytes, 0, 4) else {
            return Err(LoadBalancerRequestError::InvalidText);
        };
        let Some(month) = number(bytes, 5, 7) else {
            return Err(LoadBalancerRequestError::InvalidText);
        };
        let Some(day) = number(bytes, 8, 10) else {
            return Err(LoadBalancerRequestError::InvalidText);
        };
        let Some(hour) = number(bytes, 11, 13) else {
            return Err(LoadBalancerRequestError::InvalidText);
        };
        let Some(minute) = number(bytes, 14, 16) else {
            return Err(LoadBalancerRequestError::InvalidText);
        };
        let Some(second) = number(bytes, 17, 19) else {
            return Err(LoadBalancerRequestError::InvalidText);
        };
        if month == 0
            || month > 12
            || day == 0
            || day > days_in_month(year, month)
            || hour > 23
            || minute > 59
            || second > 59
        {
            return Err(LoadBalancerRequestError::InvalidText);
        }
        Ok(Self(value))
    }

    /// Returns the timestamp.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

/// Positive metrics resolution in seconds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerMetricsStep(u32);

impl LoadBalancerMetricsStep {
    /// Creates a positive resolution.
    pub const fn new(seconds: u32) -> Result<Self, LoadBalancerRequestError> {
        if seconds == 0 {
            return Err(LoadBalancerRequestError::InvalidText);
        }
        Ok(Self(seconds))
    }

    /// Returns seconds.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Load Balancer metrics request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LoadBalancerMetricsRequest<'a> {
    id: LoadBalancerId,
    metric_types: LoadBalancerMetricTypes,
    start: LoadBalancerTimestamp<'a>,
    end: LoadBalancerTimestamp<'a>,
    step: Option<LoadBalancerMetricsStep>,
}

impl<'a> LoadBalancerMetricsRequest<'a> {
    /// Creates a request with a non-empty metric selection and increasing UTC range.
    pub fn try_new(
        id: LoadBalancerId,
        metric_types: LoadBalancerMetricTypes,
        start: LoadBalancerTimestamp<'a>,
        end: LoadBalancerTimestamp<'a>,
    ) -> Result<Self, LoadBalancerRequestError> {
        if start.as_str() >= end.as_str() {
            return Err(LoadBalancerRequestError::InvalidTimeRange);
        }
        Ok(Self {
            id,
            metric_types,
            start,
            end,
            step: None,
        })
    }

    /// Sets a metrics resolution.
    #[must_use]
    pub const fn with_step(mut self, step: LoadBalancerMetricsStep) -> Self {
        self.step = Some(step);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> LoadBalancerEndpoint {
        LoadBalancerEndpoint::Metrics(self.id)
    }

    /// Writes the deterministic query string.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, LoadBalancerRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        writer.pair("end", self.end.as_str())?;
        if let Some(step) = self.step {
            writer.u64_pair("step", u64::from(step.get()))?;
        }
        writer.pair("start", self.start.as_str())?;
        writer.pair("type", self.metric_types.as_api_str())?;
        Ok(writer.len())
    }
}

fn digits(bytes: &[u8], start: usize, end: usize) -> bool {
    bytes
        .get(start..end)
        .is_some_and(|value| value.iter().all(u8::is_ascii_digit))
}

fn number(bytes: &[u8], start: usize, end: usize) -> Option<u16> {
    let bytes = bytes.get(start..end)?;
    let mut value = 0_u16;
    for byte in bytes {
        value = value
            .checked_mul(10)?
            .checked_add(u16::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

const fn days_in_month(year: u16, month: u16) -> u16 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
