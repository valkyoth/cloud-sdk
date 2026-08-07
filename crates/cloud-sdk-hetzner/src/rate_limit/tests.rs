use cloud_sdk::rate_limit::{
    DelayConflictPolicy, DelaySeconds, ExcessDelayPolicy, PastTimestampPolicy, QuotaDelayPolicy,
    QuotaReset, RetryAfter, WallClockTimestamp, decide_delay,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};

use super::{HetznerQuota, HetznerQuotaError};

fn headers<'a>(storage: &'a mut [u8], values: &[(&str, &[u8])]) -> ResponseHeaders<'a> {
    let mut headers = ResponseHeaders::new(storage);
    for (name, value) in values {
        assert_eq!(
            headers.try_push(name, value, HeaderSensitivity::Public),
            Ok(())
        );
    }
    headers
}

#[test]
fn large_quota_accessors_borrow_instead_of_copying_the_aggregate() {
    let _: fn(&HetznerQuota) -> Option<RetryAfter> = HetznerQuota::retry_after;
    let _: fn(&HetznerQuota) -> Option<cloud_sdk::rate_limit::RateLimit> = HetznerQuota::rate_limit;
    assert!(core::mem::size_of::<HetznerQuota>() <= 128);
}

#[test]
fn decodes_complete_provider_bucket_and_retry_after() {
    let mut storage = [0_u8; 8_192];
    let headers = headers(
        &mut storage,
        &[
            ("ratelimit-limit", b"3600"),
            ("ratelimit-remaining", b"3599"),
            ("ratelimit-reset", b"42"),
            ("retry-after", b"10"),
        ],
    );
    let decoded = HetznerQuota::decode(&headers, WallClockTimestamp::new(1_000));
    assert!(decoded.is_ok());
    let Ok(decoded) = decoded else {
        unreachable!("security fixture construction failed")
    };
    assert_eq!(decoded.buckets().len(), 1);
    assert_eq!(decoded.rate_limit().map(|value| value.limit()), Some(3600));
    assert_eq!(
        decoded.retry_after(),
        Some(RetryAfter::Delay(cloud_sdk::rate_limit::DelaySeconds::new(
            10
        )))
    );
}

#[test]
fn compact_quota_converts_into_complete_rollback_aware_delay_policy() {
    let mut storage = [0_u8; 8_192];
    let headers = headers(
        &mut storage,
        &[
            ("ratelimit-limit", b"3600"),
            ("ratelimit-remaining", b"0"),
            ("ratelimit-reset", b"42"),
            ("retry-after", b"10"),
        ],
    );
    let decoded = HetznerQuota::decode(&headers, WallClockTimestamp::new(1));
    let Ok(decoded) = decoded else {
        unreachable!("quota conversion fixture failed")
    };
    let converted = decoded.to_quota_buckets();
    let Ok(converted) = converted else {
        unreachable!("compact quota conversion failed")
    };
    let Some(bucket) = converted.iter().next() else {
        unreachable!("converted quota bucket is missing")
    };
    assert_eq!(bucket.id().as_bytes(), b"hetzner-project-hourly");
    assert_eq!(bucket.limit(), 3600);
    assert_eq!(bucket.remaining(), 0);
    assert_eq!(bucket.reset(), QuotaReset::At(WallClockTimestamp::new(42)));

    let policy = QuotaDelayPolicy::new(
        DelaySeconds::new(300),
        PastTimestampPolicy::Reject,
        ExcessDelayPolicy::Reject,
        DelayConflictPolicy::Longest,
    );
    let decision = decide_delay(
        &converted,
        decoded.retry_after(),
        WallClockTimestamp::new(1),
        None,
        policy,
    );
    let Ok(Some(decision)) = decision else {
        unreachable!("complete delay policy rejected valid quota")
    };
    assert_eq!(decision.delay(), DelaySeconds::new(41));
    assert!(
        decide_delay(
            &converted,
            decoded.retry_after(),
            WallClockTimestamp::new(1),
            Some(WallClockTimestamp::new(2)),
            policy,
        )
        .is_err()
    );
}

#[test]
fn rejects_partial_invalid_and_incoherent_provider_headers() {
    for values in [
        &[("ratelimit-limit", b"3600".as_slice())][..],
        &[
            ("ratelimit-limit", b"3600".as_slice()),
            ("ratelimit-remaining", b"x".as_slice()),
            ("ratelimit-reset", b"42".as_slice()),
        ][..],
        &[
            ("ratelimit-limit", b"10".as_slice()),
            ("ratelimit-remaining", b"11".as_slice()),
            ("ratelimit-reset", b"42".as_slice()),
        ][..],
    ] {
        let mut storage = [0_u8; 8_192];
        let result =
            HetznerQuota::decode(&headers(&mut storage, values), WallClockTimestamp::new(0));
        assert!(result.is_err());
    }
}

#[test]
fn obsolete_retry_date_requires_external_clock_on_legacy_decoder() {
    let mut storage = [0_u8; 8_192];
    let values = &[("retry-after", b"Sunday, 06-Nov-94 08:49:37 GMT".as_slice())];
    let headers = headers(&mut storage, values);
    assert_eq!(
        HetznerQuota::decode_without_clock(&headers),
        Err(HetznerQuotaError::WallClockRequired)
    );
    assert!(HetznerQuota::decode(&headers, WallClockTimestamp::new(1_767_225_600)).is_ok());
}
