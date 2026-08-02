#!/usr/bin/env sh
set -eu

core=crates/cloud-sdk/src/retry
prepared=crates/cloud-sdk/src/operation/prepared.rs
provider=crates/cloud-sdk-hetzner/src/prepared/operation.rs
adapter=crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs

for required in \
    'pub struct CanonicalFingerprint' \
    'pub struct FingerprintDigest' \
    'pub struct RetrySubject' \
    'pub trait FingerprintHasher' \
    'pub struct IdempotencyIntent' \
    'pub struct IdempotencyBinding' \
    'pub struct MaxAttempts' \
    'pub struct RetryPolicy' \
    'pub struct RetryController' \
    'pub struct RetryPermit' \
    'pub struct MonotonicInstant' \
    'pub struct MonotonicDuration'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "retry strategies: missing core contract $required" >&2
        exit 1
    fi
done

for required in \
    'cloud-sdk/retry-fingerprint/v2\0' \
    'FingerprintKind::Exact' \
    'Sha256,' \
    'Blake3,' \
    'sanitize_bytes(output)' \
    'canonical_host_field' \
    'sanitize_bytes(source)' \
    'ConstantTimeEq' \
    'execute_blocking' \
    'execute_async' \
    'ReplayPolicyMismatch' \
    'has_same_retry_policy' \
    'observe_monotonic' \
    'EndpointNotAdmitted' \
    'CumulativeDelayOverflow' \
    'MonotonicRollback' \
    'DeliveryPhase::PossiblySent'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "retry strategies: missing fail-closed contract $required" >&2
        exit 1
    fi
done

grep -Fq 'pub enum BodyReplayability' "$prepared"
grep -Fq 'has_same_header_policy' "$prepared"
grep -Fq 'header.sensitivity()' "$core/fingerprint.rs"
grep -Fq '.with_replayable_body()' "$provider"
grep -Fq '.retry_canceled_requests(false)' "$adapter"
if grep -Fq '.retry_canceled_requests(true)' "$adapter"; then
    echo 'retry strategies: raw transport enables an independent retry owner' >&2
    exit 1
fi

for contract in \
    'does not imply provider-side deduplication' \
    'neither `Copy` nor `Clone`' \
    'Unknown transport delivery' \
    'Rust `Hash`, CRC'; do
    if ! grep -Fq "$contract" docs/RETRY_AND_IDEMPOTENCY.md; then
        echo "retry strategies: missing documented boundary $contract" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features retry
cargo test --locked -p cloud-sdk-hetzner --all-features prepared
cargo test --locked -p cloud-sdk-testkit --all-features prepared
cargo check --locked -p cloud-sdk --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features
cargo check --locked --manifest-path fuzz/Cargo.toml --bin retry_policy
cargo clippy --locked -p cloud-sdk -p cloud-sdk-hetzner -p cloud-sdk-testkit \
    --all-targets --all-features -- -D warnings

printf '%s\n' 'Retry identity, idempotency, budget, provider, adapter, and no_std checks passed.'
