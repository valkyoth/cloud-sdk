#![no_main]

use cloud_sdk::rate_limit::{
    DelayConflictPolicy, DelaySeconds, ExcessDelayPolicy, PastTimestampPolicy, QuotaBucket,
    QuotaBucketId, QuotaBuckets, QuotaDelayPolicy, QuotaReset, RetryAfter, WallClockTimestamp,
    decide_delay,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
use cloud_sdk_hetzner::rate_limit::HetznerQuota;
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let now = WallClockTimestamp::new(read_u64(data, 0));
    let retry_bytes = data.get(8..).unwrap_or_default();
    let retry_after = RetryAfter::parse(retry_bytes, now).ok();

    let mut buckets = QuotaBuckets::new();
    let reset = match data.first().copied().unwrap_or(0) % 3 {
        0 => QuotaReset::After(DelaySeconds::new(read_u64(data, 8))),
        1 => QuotaReset::At(WallClockTimestamp::new(read_u64(data, 16))),
        _ => QuotaReset::Unknown,
    };
    if let Ok(id) = QuotaBucketId::new(b"fuzz-bucket")
        && let Ok(bucket) = QuotaBucket::new(id, read_u64(data, 24), read_u64(data, 32), reset)
    {
        let _ = buckets.try_push(bucket);
    }
    let policy = QuotaDelayPolicy::new(
        DelaySeconds::new(read_u64(data, 40)),
        if data.get(48).copied().unwrap_or(0) & 1 == 0 {
            PastTimestampPolicy::Immediate
        } else {
            PastTimestampPolicy::Reject
        },
        if data.get(49).copied().unwrap_or(0) & 1 == 0 {
            ExcessDelayPolicy::Clamp
        } else {
            ExcessDelayPolicy::Reject
        },
        match data.get(50).copied().unwrap_or(0) % 3 {
            0 => DelayConflictPolicy::RetryAfterPrecedence,
            1 => DelayConflictPolicy::Longest,
            _ => DelayConflictPolicy::RejectMismatch,
        },
    );
    let previous = (data.get(51).copied().unwrap_or(0) & 1 != 0)
        .then(|| WallClockTimestamp::new(read_u64(data, 52)));
    let _ = decide_delay(&buckets, retry_after, now, previous, policy);

    fuzz_provider_headers(data, now);
});

fn fuzz_provider_headers(data: &[u8], now: WallClockTimestamp) {
    let mut storage = [0_u8; 8_192];
    let mut headers = ResponseHeaders::new(&mut storage);
    let values = [
        ("ratelimit-limit", data.get(0..20).unwrap_or_default()),
        ("ratelimit-remaining", data.get(20..40).unwrap_or_default()),
        ("ratelimit-reset", data.get(40..60).unwrap_or_default()),
        ("retry-after", data.get(60..).unwrap_or_default()),
    ];
    for (index, (name, value)) in values.iter().enumerate() {
        if data.get(index).copied().unwrap_or(0) & 1 != 0 {
            let _ = headers.try_push(name, value, HeaderSensitivity::Public);
        }
    }
    let _ = HetznerQuota::decode(&headers, now);
}

fn read_u64(data: &[u8], start: usize) -> u64 {
    let Some(end) = start.checked_add(8) else {
        return 0;
    };
    let Some(value) = data.get(start..end) else {
        return 0;
    };
    let Ok(value) = <[u8; 8]>::try_from(value) else {
        return 0;
    };
    u64::from_le_bytes(value)
}
