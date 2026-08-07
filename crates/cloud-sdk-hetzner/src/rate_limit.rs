//! Hetzner-owned quota and standard retry metadata decoding.

use core::fmt;

use cloud_sdk::rate_limit::{
    QuotaError, RateLimit, RetryAfter, RetryAfterError, WallClockTimestamp,
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
    buckets: [HetznerQuotaBucket; 1],
    bucket_len: u8,
    retry_after: Option<RetryAfter>,
}

/// Hetzner's single project-hour quota bucket in compact inline storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HetznerQuotaBucket {
    limit: u64,
    remaining: u64,
    reset: WallClockTimestamp,
}

impl HetznerQuotaBucket {
    const EMPTY: Self = Self {
        limit: 0,
        remaining: 0,
        reset: WallClockTimestamp::new(0),
    };

    /// Returns the stable provider bucket identity.
    #[must_use]
    pub const fn id(&self) -> &'static [u8] {
        HETZNER_PROJECT_BUCKET
    }

    /// Returns the hourly request limit.
    #[must_use]
    pub const fn limit(&self) -> u64 {
        self.limit
    }

    /// Returns the requests remaining in the current provider window.
    #[must_use]
    pub const fn remaining(&self) -> u64 {
        self.remaining
    }

    /// Returns the absolute provider reset timestamp.
    #[must_use]
    pub const fn reset(&self) -> WallClockTimestamp {
        self.reset
    }

    /// Reports whether this bucket has no requests remaining.
    #[must_use]
    pub const fn is_exhausted(&self) -> bool {
        self.remaining == 0
    }
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
    pub fn buckets(&self) -> &[HetznerQuotaBucket] {
        self.buckets
            .get(..usize::from(self.bucket_len))
            .unwrap_or_default()
    }

    /// Returns the decoded standard retry instruction.
    #[must_use]
    pub const fn retry_after(&self) -> Option<RetryAfter> {
        self.retry_after
    }

    /// Reports whether neither quota nor retry metadata was supplied.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.bucket_len == 0 && self.retry_after.is_none()
    }

    /// Returns the legacy single-bucket compatibility view.
    #[must_use]
    pub fn rate_limit(&self) -> Option<RateLimit> {
        let bucket = self.buckets().first()?;
        RateLimit::new(bucket.limit(), bucket.remaining(), bucket.reset().get()).ok()
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
    let mut buckets = [HetznerQuotaBucket::EMPTY; 1];
    let mut bucket_len = 0_u8;
    if let (Some(limit), Some(remaining), Some(reset)) = (limit, remaining, reset) {
        let limit = parse_decimal(limit.value())?;
        let remaining = parse_decimal(remaining.value())?;
        if limit == 0 {
            return Err(HetznerQuotaError::Quota(QuotaError::LimitZero));
        }
        if remaining > limit {
            return Err(HetznerQuotaError::Quota(QuotaError::RemainingExceedsLimit));
        }
        let slot = buckets
            .first_mut()
            .ok_or(HetznerQuotaError::Quota(QuotaError::TooManyBuckets))?;
        *slot = HetznerQuotaBucket {
            limit,
            remaining,
            reset: WallClockTimestamp::new(parse_decimal(reset.value())?),
        };
        bucket_len = 1;
    }
    let retry_after = headers
        .get(RETRY_AFTER_HEADER)
        .map(|header| parse_retry_after(header.value(), now))
        .transpose()?;
    Ok(HetznerQuota {
        buckets,
        bucket_len,
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
