#!/usr/bin/env sh
set -eu

core=crates/cloud-sdk/src/rate_limit.rs
provider=crates/cloud-sdk-hetzner/src/rate_limit.rs
checked=crates/cloud-sdk-hetzner/src/serde/checked.rs
reqwest=crates/cloud-sdk-reqwest/src

for required in \
    'pub struct DelaySeconds' \
    'pub struct WallClockTimestamp' \
    'pub enum RetryAfter' \
    'pub struct QuotaBucket' \
    'pub struct QuotaBuckets' \
    'pub struct QuotaDelayPolicy' \
    'pub fn decide_delay'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/rate_limit; then
        echo "quota strategies: missing core contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub struct HetznerQuota' \
    'pub fn decode(' \
    'PartialHeaders' \
    'WallClockRequired' \
    'ratelimit-limit' \
    'retry-after'; do
    if ! grep -Fq "$required" "$provider"; then
        echo "quota strategies: missing provider decoder contract $required" >&2
        exit 1
    fi
done

for required in 'pub fn quota' 'decode_response_at' 'HetznerDecodeError::Quota'; do
    if ! grep -Fq "$required" "$checked"; then
        echo "quota strategies: missing checked response contract $required" >&2
        exit 1
    fi
done

if grep -R -Eq 'parse_rate_limit|InvalidRateLimitHeaders' "$reqwest"; then
    echo 'quota strategies: reqwest still owns provider quota decoding' >&2
    exit 1
fi

cargo test --locked -p cloud-sdk --all-features rate_limit
cargo test --locked -p cloud-sdk-hetzner --all-features rate_limit
cargo test --locked -p cloud-sdk-hetzner --all-features checked_success_and_error_retain_provider_owned_quota
cargo check --locked -p cloud-sdk --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features
cargo check --locked --manifest-path fuzz/Cargo.toml --bin quota_retry

printf '%s\n' 'Quota strategy, provider ownership, checked response, and no_std checks passed.'
