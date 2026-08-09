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
    'bearer|redirect|proxy|decompress|retry::' \
    crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs; then
    echo "raw HTTP executor: provider auth, redirect, proxy, decoding, or retry entered raw engine" >&2
    exit 1
fi

raw_engine=crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs
for required in \
    'pub(crate) trait RawResponseSink' \
    'response: &mut impl RawResponseSink' \
    'Result<ResponseCompletion, RawTransportFailure>' \
    'self.execute_inner(request, policy, None, response)' \
    'self.execute_inner(request, policy, Some(authorization), response)' \
    'authorization.set_sensitive(true)' \
    'headers.insert(AUTHORIZATION, authorization)'; do
    if ! grep -Fq "$required" "$raw_engine"; then
        echo "raw HTTP executor: raw/authenticated ownership split is incomplete" >&2
        exit 1
    fi
done
if grep -Fq '.commit(' "$raw_engine"; then
    echo "raw HTTP executor: raw engine can commit caller response storage" >&2
    exit 1
fi

blocking_raw=crates/cloud-sdk-reqwest/src/blocking/raw.rs
for required in \
    '.begin_attempt()' \
    '.commit_completion(completion)'; do
    if ! grep -Fq "$required" "$blocking_raw"; then
        echo "raw HTTP executor: blocking adapter does not own response commit" >&2
        exit 1
    fi
done

async_raw=crates/cloud-sdk-reqwest/src/asynchronous/raw.rs
for required in \
    'AsyncResponseStaging' \
    'Result<ResponseCompletion, RawTransportFailure>'; do
    if ! grep -Fq "$required" "$async_raw"; then
        echo "raw HTTP executor: async adapter does not use staged completion" >&2
        exit 1
    fi
done
if grep -Fq 'ResponseWriter' "$async_raw"; then
    echo "raw HTTP executor: async adapter received committing response access" >&2
    exit 1
fi

for raw_client in \
    crates/cloud-sdk-reqwest/src/blocking/raw.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/raw.rs; do
    if ! grep -Fq '.execute(request, policy,' "$raw_client"; then
        echo "raw HTTP executor: credential-free client does not use the no-auth entry" >&2
        exit 1
    fi
done

for feature in \
    blocking-rustls \
    blocking-rustls-webpki-roots \
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
