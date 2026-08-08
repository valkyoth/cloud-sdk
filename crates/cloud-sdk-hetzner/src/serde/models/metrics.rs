//! Exact, bounded server and load-balancer metrics responses.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use super::{ExactDecimal, ResponseModelError, UtcTimestamp, checked_text, value_text};
use crate::serde::strict_json::Value;

const MAX_METRIC_SERIES: usize = 512;
const MAX_METRIC_POINTS_PER_SERIES: usize = 16_384;
const MAX_METRIC_POINTS_TOTAL: usize = 16_384;

/// One exact timestamp/value pair from a metrics response.
#[derive(Eq, PartialEq)]
pub struct MetricPoint {
    timestamp: ExactDecimal,
    value: String,
}

impl MetricPoint {
    /// Returns the exact provider timestamp number.
    #[must_use]
    pub const fn timestamp(&self) -> &ExactDecimal {
        &self.timestamp
    }

    /// Returns the decimal metric value text without lossy conversion.
    #[must_use]
    pub fn value(&self) -> &str {
        &self.value
    }

    /// Fallibly copies this bounded point.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        Ok(Self {
            timestamp: self.timestamp.try_clone()?,
            value: checked_text(&self.value, 256)?,
        })
    }
}

impl fmt::Debug for MetricPoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("MetricPoint([redacted])")
    }
}

/// Named, bounded metrics time series.
#[derive(Eq, PartialEq)]
pub struct MetricSeries {
    name: String,
    points: Vec<MetricPoint>,
}

impl MetricSeries {
    /// Returns the provider series name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns bounded metric points.
    #[must_use]
    pub fn points(&self) -> &[MetricPoint] {
        &self.points
    }

    /// Fallibly copies this bounded series.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        let mut points = Vec::new();
        points
            .try_reserve_exact(self.points.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for point in &self.points {
            points.push(point.try_clone()?);
        }
        Ok(Self {
            name: checked_text(&self.name, 256)?,
            points,
        })
    }
}

impl fmt::Debug for MetricSeries {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MetricSeries")
            .field("name", &"[redacted]")
            .field("point_count", &self.points.len())
            .finish()
    }
}

/// Validated server or load-balancer metrics result.
#[derive(Eq, PartialEq)]
pub struct Metrics {
    start: UtcTimestamp,
    end: UtcTimestamp,
    step: ExactDecimal,
    series: Vec<MetricSeries>,
}

impl Metrics {
    /// Returns the requested range start.
    #[must_use]
    pub fn start(&self) -> &str {
        self.start.as_str()
    }

    /// Returns the requested range end.
    #[must_use]
    pub fn end(&self) -> &str {
        self.end.as_str()
    }

    /// Returns the positive exact provider sampling step.
    #[must_use]
    pub const fn step(&self) -> &ExactDecimal {
        &self.step
    }

    /// Returns the bounded time series.
    #[must_use]
    pub fn series(&self) -> &[MetricSeries] {
        &self.series
    }

    /// Fallibly copies the complete bounded metrics result.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        let mut series = Vec::new();
        series
            .try_reserve_exact(self.series.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for item in &self.series {
            series.push(item.try_clone()?);
        }
        Ok(Self {
            start: self.start.try_clone()?,
            end: self.end.try_clone()?,
            step: self.step.try_clone()?,
            series,
        })
    }
}

impl fmt::Debug for Metrics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Metrics")
            .field("range", &"[redacted]")
            .field("step", &"[redacted]")
            .field("series_count", &self.series.len())
            .finish()
    }
}

pub(crate) fn parse_metrics(value: &mut Value) -> Result<Metrics, ResponseModelError> {
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let start = timestamp_field(fields.get("start"))?;
    let end = timestamp_field(fields.get("end"))?;
    let step_value = fields
        .get_mut("step")
        .ok_or(ResponseModelError::MissingField)?;
    if !step_value
        .as_f64()
        .is_some_and(|step| step.is_finite() && step > 0.0)
    {
        return Err(ResponseModelError::InvalidNumber);
    }
    let step = ExactDecimal::take(step_value)?;
    let time_series = fields
        .get_mut("time_series")
        .ok_or(ResponseModelError::MissingField)?
        .as_object_mut()
        .ok_or(ResponseModelError::WrongType)?;
    if time_series.len() > MAX_METRIC_SERIES {
        return Err(ResponseModelError::TooManyItems);
    }

    let mut total_points = 0_usize;
    let mut series = Vec::new();
    series
        .try_reserve_exact(time_series.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for (name, value) in time_series.iter_mut() {
        let name = checked_text(name.as_str(), 256)?;
        let series_fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
        let values = series_fields
            .get_mut("values")
            .ok_or(ResponseModelError::MissingField)?
            .as_array_mut()
            .ok_or(ResponseModelError::WrongType)?;
        if values.len() > MAX_METRIC_POINTS_PER_SERIES {
            return Err(ResponseModelError::TooManyItems);
        }
        total_points = total_points
            .checked_add(values.len())
            .filter(|total| *total <= MAX_METRIC_POINTS_TOTAL)
            .ok_or(ResponseModelError::TooManyItems)?;
        let mut points = Vec::new();
        points
            .try_reserve_exact(values.len())
            .map_err(|_| ResponseModelError::Allocation)?;
        for point in values {
            let pair = point.as_array_mut().ok_or(ResponseModelError::WrongType)?;
            let [timestamp, value] = pair else {
                return Err(ResponseModelError::EnvelopeMismatch);
            };
            if !timestamp
                .as_f64()
                .is_some_and(|timestamp| timestamp.is_finite() && timestamp >= 0.0)
            {
                return Err(ResponseModelError::InvalidNumber);
            }
            points.push(MetricPoint {
                timestamp: ExactDecimal::take(timestamp)?,
                value: value_text(value, 256)?,
            });
        }
        series.push(MetricSeries { name, points });
    }
    Ok(Metrics {
        start,
        end,
        step,
        series,
    })
}

fn timestamp_field(value: Option<&Value>) -> Result<UtcTimestamp, ResponseModelError> {
    let value = value.ok_or(ResponseModelError::MissingField)?;
    value
        .try_with_str(UtcTimestamp::try_new)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
}

#[cfg(test)]
mod tests {
    use alloc::format;
    use alloc::string::String;
    use core::fmt::Write as _;

    use super::{MAX_METRIC_POINTS_TOTAL, parse_metrics};
    use crate::serde::models::ResponseModelError;
    use crate::serde::strict_json::parse;

    #[test]
    fn metrics_preserve_exact_numbers_and_redact_diagnostics() {
        let input = br#"{"start":"2026-08-08T00:00:00Z","end":"2026-08-08T01:00:00Z","step":60.000000000000001,"time_series":{"cpu":{"values":[[1712345678.123456789,"0.100000000000000001"]]}}}"#;
        let value = parse(input);
        let Ok(mut value) = value else {
            unreachable!("metrics fixture failed to parse")
        };
        let metrics = parse_metrics(&mut value);
        let Ok(metrics) = metrics else {
            unreachable!("metrics fixture failed validation")
        };
        assert_eq!(metrics.step().as_str(), "60.000000000000001");
        let Some(point) = metrics
            .series()
            .first()
            .and_then(|series| series.points().first())
        else {
            unreachable!("metrics fixture lost its point")
        };
        assert_eq!(point.timestamp().as_str(), "1712345678.123456789");
        assert_eq!(point.value(), "0.100000000000000001");
        let debug = format!("{metrics:?}");
        assert!(!debug.contains("1712345678"));
        assert!(!debug.contains("0.100000"));
    }

    #[test]
    fn metrics_enforce_utc_ranges_positive_steps_and_aggregate_points() {
        for invalid in [
            br#"{"start":"2026-08-08T00:00:00+00:00","end":"2026-08-08T01:00:00Z","step":60,"time_series":{}}"#.as_slice(),
            br#"{"start":"2026-08-08T00:00:00Z","end":"2026-08-08T01:00:00Z","step":0,"time_series":{}}"#,
            br#"{"start":"2025-02-29T00:00:00Z","end":"2026-08-08T01:00:00Z","step":60,"time_series":{}}"#,
        ] {
            let Ok(mut value) = parse(invalid) else {
                unreachable!("invalid model fixture was not valid JSON")
            };
            assert!(parse_metrics(&mut value).is_err());
        }

        let mut time_series = String::new();
        let mut next = 0_usize;
        for series in 0..5 {
            if series != 0 {
                time_series.push(',');
            }
            write!(&mut time_series, "\"series{series}\":{{\"values\":[")
                .unwrap_or_else(|_| unreachable!("series fixture formatting failed"));
            for point in 0..3_277 {
                if point != 0 {
                    time_series.push(',');
                }
                write!(&mut time_series, "[{next},\"1\"]")
                    .unwrap_or_else(|_| unreachable!("point fixture formatting failed"));
                next += 1;
            }
            time_series.push_str("]}");
        }
        assert_eq!(next, MAX_METRIC_POINTS_TOTAL + 1);
        let input = format!(
            "{{\"start\":\"2026-08-08T00:00:00Z\",\"end\":\"2026-08-08T01:00:00Z\",\"step\":60,\"time_series\":{{{time_series}}}}}"
        );
        let Ok(mut value) = parse(input.as_bytes()) else {
            unreachable!("aggregate metrics fixture failed to parse")
        };
        assert_eq!(
            parse_metrics(&mut value),
            Err(ResponseModelError::TooManyItems)
        );
    }
}
