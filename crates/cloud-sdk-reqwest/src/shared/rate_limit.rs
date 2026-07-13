use cloud_sdk::rate_limit::RateLimit;
use reqwest::header::{HeaderMap, HeaderValue};

use super::TransportError;

const LIMIT: &str = "ratelimit-limit";
const REMAINING: &str = "ratelimit-remaining";
const RESET: &str = "ratelimit-reset";

pub(crate) fn parse_rate_limit(headers: &HeaderMap) -> Result<Option<RateLimit>, TransportError> {
    let limit = headers.get(LIMIT);
    let remaining = headers.get(REMAINING);
    let reset = headers.get(RESET);
    if limit.is_none() && remaining.is_none() && reset.is_none() {
        return Ok(None);
    }
    let limit = parse_decimal(limit.ok_or(TransportError::InvalidRateLimitHeaders)?)?;
    let remaining = parse_decimal(remaining.ok_or(TransportError::InvalidRateLimitHeaders)?)?;
    let reset = parse_decimal(reset.ok_or(TransportError::InvalidRateLimitHeaders)?)?;
    RateLimit::new(limit, remaining, reset)
        .map(Some)
        .map_err(|_| TransportError::InvalidRateLimitHeaders)
}

fn parse_decimal(value: &HeaderValue) -> Result<u64, TransportError> {
    let bytes = value.as_bytes();
    if bytes.is_empty() {
        return Err(TransportError::InvalidRateLimitHeaders);
    }
    let mut parsed = 0_u64;
    for byte in bytes {
        if !byte.is_ascii_digit() {
            return Err(TransportError::InvalidRateLimitHeaders);
        }
        parsed = parsed
            .checked_mul(10)
            .and_then(|value| value.checked_add(u64::from(*byte & 0x0f)))
            .ok_or(TransportError::InvalidRateLimitHeaders)?;
    }
    Ok(parsed)
}

#[cfg(test)]
mod tests {
    use super::{LIMIT, REMAINING, RESET, parse_rate_limit};
    use crate::shared::TransportError;
    use reqwest::header::{HeaderMap, HeaderValue};

    #[test]
    fn accepts_absent_or_coherent_headers() {
        assert_eq!(parse_rate_limit(&HeaderMap::new()), Ok(None));
        let headers = headers("3600", "3599", "42");
        let parsed = parse_rate_limit(&headers);
        assert!(parsed.is_ok());
        let Some(rate_limit) = parsed.ok().flatten() else {
            return;
        };
        assert_eq!(rate_limit.limit(), 3600);
        assert_eq!(rate_limit.remaining(), 3599);
        assert_eq!(rate_limit.reset_epoch_seconds(), 42);
    }

    #[test]
    fn rejects_partial_nondecimal_overflow_and_incoherent_headers() {
        let mut partial = HeaderMap::new();
        partial.insert(LIMIT, HeaderValue::from_static("3600"));
        assert_eq!(
            parse_rate_limit(&partial),
            Err(TransportError::InvalidRateLimitHeaders)
        );
        for values in [
            ("+3600", "1", "42"),
            ("3600", "3601", "42"),
            ("0", "0", "42"),
            ("18446744073709551616", "1", "42"),
        ] {
            assert_eq!(
                parse_rate_limit(&headers(values.0, values.1, values.2)),
                Err(TransportError::InvalidRateLimitHeaders)
            );
        }
    }

    fn headers(limit: &'static str, remaining: &'static str, reset: &'static str) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(LIMIT, HeaderValue::from_static(limit));
        headers.insert(REMAINING, HeaderValue::from_static(remaining));
        headers.insert(RESET, HeaderValue::from_static(reset));
        headers
    }
}
