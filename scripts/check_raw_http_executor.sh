#!/usr/bin/env sh
set -eu

for contract in \
    'pub trait BlockingRawHttpExecutor' \
    'pub trait AsyncRawHttpExecutor' \
    'pub struct RawResponsePolicy' \
    'pub enum DeliveryPhase' \
    'pub struct TransportFailure'; do
    if ! grep -R -Fq "$contract" crates/cloud-sdk/src; then
        echo "raw HTTP executor: missing core contract $contract" >&2
        exit 1
    fi
done

for adapter in RawBlockingClient RawAsyncClient RawHyperClient; do
    if ! grep -R -Fq "struct $adapter" crates/cloud-sdk-reqwest/src; then
        echo "raw HTTP executor: missing adapter $adapter" >&2
        exit 1
    fi
done

for policy in \
    'http1_max_headers' \
    'http1_max_buf_size' \
    'pool_max_idle_per_host(0)' \
    'retry_canceled_requests(false)' \
    'hyper::ext::on_informational' \
    'frame.is_trailers()'; do
    if ! grep -Fq "$policy" crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs; then
        echo "raw HTTP executor: missing wire policy $policy" >&2
        exit 1
    fi
done

if grep -Ei \
    'authorization|bearer|redirect|proxy|decompress|retry::' \
    crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs; then
    echo "raw HTTP executor: implicit auth, redirect, proxy, decoding, or retry entered raw engine" >&2
    exit 1
fi

for feature in \
    blocking-rustls \
    blocking-rustls-webpki-roots \
    blocking-rustls-fips \
    async-rustls; do
    cargo check -p cloud-sdk-reqwest --no-default-features --features "$feature"
done

cargo test -p cloud-sdk --no-default-features transport::raw
cargo test -p cloud-sdk-testkit --all-features raw_fault
cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls blocking::tests::raw_executor
cargo test -p cloud-sdk-reqwest --no-default-features \
    --features async-rustls asynchronous::tests::raw_executor

test -s docs/RAW_HTTP_EXECUTOR.md
test -s docs/MIGRATION_0.40.0.md
