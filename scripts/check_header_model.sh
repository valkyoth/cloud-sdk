#!/usr/bin/env sh
set -eu

if grep -R -n --include='*.rs' \
    '\.header(ACCEPT\|\.header(CONTENT_TYPE' \
    crates/cloud-sdk-reqwest/src/blocking \
    crates/cloud-sdk-reqwest/src/asynchronous; then
    echo "header model: reqwest still injects provider content policy" >&2
    exit 1
fi

for reserved in \
    authorization \
    connection \
    content-length \
    host \
    proxy-authorization \
    transfer-encoding \
    upgrade; do
    if ! grep -Fq "\"$reserved\"" \
        crates/cloud-sdk/src/transport/header/mod.rs; then
        echo "header model: missing reserved request name $reserved" >&2
        exit 1
    fi
done

for required in \
    MAX_HEADER_NAME_BYTES \
    MAX_HEADER_VALUE_BYTES \
    MAX_REQUEST_HEADERS \
    MAX_REQUEST_HEADER_BYTES \
    MAX_RESPONSE_HEADERS \
    MAX_RESPONSE_HEADER_BYTES; do
    if ! grep -Fq "pub const $required" \
        crates/cloud-sdk/src/transport/header/mod.rs; then
        echo "header model: missing public bound $required" >&2
        exit 1
    fi
done

if ! grep -Fq 'capture_response_headers(response.headers())' \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    || ! grep -Fq 'capture_response_headers(response.headers())' \
        crates/cloud-sdk-reqwest/src/asynchronous/client.rs; then
    echo "header model: adapter response capture is incomplete" >&2
    exit 1
fi

cargo test -p cloud-sdk transport::header
cargo test -p cloud-sdk transport::content_type
cargo test -p cloud-sdk --doc
cargo test -p cloud-sdk-hetzner prepared::
cargo test -p cloud-sdk-testkit --all-features
cargo test -p cloud-sdk-reqwest --all-features shared::headers
cargo test -p cloud-sdk-reqwest --all-features \
    asynchronous::tests::lifecycle::async_send_future_stays_within_explicit_state_budget

echo "bounded HTTP header model checks passed."
