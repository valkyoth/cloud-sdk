//! Hetzner-owned quota and standard retry metadata decoding.

use core::fmt;

use cloud_sdk::rate_limit::{
    QuotaBucket, QuotaBucketId, QuotaBuckets, QuotaError, QuotaReset, RateLimit, RetryAfter,
    RetryAfterError, WallClockTimestamp,
};
use cloud_sdk::transport::ResponseHeaders;

/// Complete Hetzner quota header set.
pub const RATE_LIMIT_HEADERS: &[&str] =
    &["ratelimit-limit", "ratelimit-remaining", "ratelimit-reset"];
/// Standard retry instruction retained for provider policy.
pub const RETRY_AFTER_HEADER: &str = "retry-after";
const HETZNER_PROJECT_BUCKET: &[u8] = b"hetzner-project-hourly";
const LIMIT_HEADER: &str = "ratelimit-limit";
const REMAINING_HEADER: &str = "ratelimit-remaining";
const RESET_HEADER: &str = "ratelimit-reset";

/// Provider-owned quota metadata from one Hetzner response.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HetznerQuota {
    buckets: QuotaBuckets,
    retry_after: Option<RetryAfter>,
}

impl HetznerQuota {
    /// Decodes retained response headers with caller-supplied wall time.
    pub fn decode(
        headers: &ResponseHeaders<'_>,
        now: WallClockTimestamp,
    ) -> Result<Self, HetznerQuotaError> {
        decode(headers, Some(now))
    }

    #[cfg(any(feature = "serde", test))]
    pub(crate) fn decode_without_clock(
        headers: &ResponseHeaders<'_>,
    ) -> Result<Self, HetznerQuotaError> {
        decode(headers, None)
    }

    /// Returns all decoded provider buckets.
    #[must_use]
    pub const fn buckets(&self) -> &QuotaBuckets {
        &self.buckets
    }

    /// Returns the decoded standard retry instruction.
    #[must_use]
    pub const fn retry_after(&self) -> Option<RetryAfter> {
        self.retry_after
    }

    /// Reports whether neither quota nor retry metadata was supplied.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.buckets.is_empty() && self.retry_after.is_none()
    }

    /// Returns the legacy single-bucket compatibility view.
    #[must_use]
    pub fn rate_limit(&self) -> Option<RateLimit> {
        let bucket = self.buckets.iter().next()?;
        let QuotaReset::At(reset) = bucket.reset() else {
            return None;
        };
        RateLimit::new(bucket.limit(), bucket.remaining(), reset.get()).ok()
    }
}

fn decode(
    headers: &ResponseHeaders<'_>,
    now: Option<WallClockTimestamp>,
) -> Result<HetznerQuota, HetznerQuotaError> {
    let limit = headers.get(LIMIT_HEADER);
    let remaining = headers.get(REMAINING_HEADER);
    let reset = headers.get(RESET_HEADER);
    match (limit.is_some(), remaining.is_some(), reset.is_some()) {
        (false, false, false) | (true, true, true) => {}
        _ => return Err(HetznerQuotaError::PartialHeaders),
    }
    let mut buckets = QuotaBuckets::new();
    if let (Some(limit), Some(remaining), Some(reset)) = (limit, remaining, reset) {
        let id = QuotaBucketId::new(HETZNER_PROJECT_BUCKET).map_err(HetznerQuotaError::Quota)?;
        let bucket = QuotaBucket::new(
            id,
            parse_decimal(limit.value())?,
            parse_decimal(remaining.value())?,
            QuotaReset::At(WallClockTimestamp::new(parse_decimal(reset.value())?)),
        )
        .map_err(HetznerQuotaError::Quota)?;
        buckets.try_push(bucket).map_err(HetznerQuotaError::Quota)?;
    }
    let retry_after = headers
        .get(RETRY_AFTER_HEADER)
        .map(|header| parse_retry_after(header.value(), now))
        .transpose()?;
    Ok(HetznerQuota {
        buckets,
        retry_after,
    })
}

fn parse_retry_after(
    value: &[u8],
    now: Option<WallClockTimestamp>,
) -> Result<RetryAfter, HetznerQuotaError> {
    let is_obsolete_rfc850 = value
        .iter()
        .position(|byte| *byte == b',')
        .is_some_and(|index| index > 3);
    if is_obsolete_rfc850 && now.is_none() {
        return Err(HetznerQuotaError::WallClockRequired);
    }
    RetryAfter::parse(value, now.unwrap_or(WallClockTimestamp::new(0)))
        .map_err(HetznerQuotaError::RetryAfter)
}

fn parse_decimal(value: &[u8]) -> Result<u64, HetznerQuotaError> {
    if value.is_empty() || !value.iter().all(u8::is_ascii_digit) {
        return Err(HetznerQuotaError::InvalidDecimal);
    }
    let mut parsed = 0_u64;
    for byte in value {
        parsed = parsed
            .checked_mul(10)
            .and_then(|current| current.checked_add(u64::from(*byte & 0x0f)))
            .ok_or(HetznerQuotaError::InvalidDecimal)?;
    }
    Ok(parsed)
}

/// Invalid Hetzner quota or retry response metadata.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HetznerQuotaError {
    /// Only part of the required three-header Hetzner set was present.
    PartialHeaders,
    /// A Hetzner decimal header was empty, non-decimal, or overflowed.
    InvalidDecimal,
    /// The obsolete two-digit HTTP date needs caller wall time.
    WallClockRequired,
    /// Decoded bucket metadata was incoherent or exceeded a bound.
    Quota(QuotaError),
    /// Standard `Retry-After` metadata was invalid.
    RetryAfter(RetryAfterError),
}

impl fmt::Display for HetznerQuotaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::PartialHeaders => "Hetzner quota headers are incomplete",
            Self::InvalidDecimal => "Hetzner quota header is not a bounded decimal",
            Self::WallClockRequired => "obsolete Retry-After date requires caller wall time",
            Self::Quota(_) => "Hetzner quota bucket is invalid",
            Self::RetryAfter(_) => "Hetzner Retry-After value is invalid",
        })
    }
}

impl core::error::Error for HetznerQuotaError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Quota(error) => Some(error),
            Self::RetryAfter(error) => Some(error),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests;
