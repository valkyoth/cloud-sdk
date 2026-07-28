#!/usr/bin/env sh
set -eu

response=crates/cloud-sdk/src/transport/response.rs
policy=crates/cloud-sdk/src/operation/policy.rs

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

for required in \
    "Result<Option<ResponseContentType<'response>>, super::ContentTypeError>" \
    'map_err(|_| ResponsePolicyError::InvalidContentType)'; do
    if ! grep -Fq "$required" "$response" "$policy"; then
        echo "header model: malformed response content type can collapse to absence" >&2
        exit 1
    fi
done

for client in \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs; do
    for required in \
        'capture_response_headers(' \
        'response_writer' \
        '.headers_mut()'; do
        if ! grep -Fq "$required" "$client"; then
            echo "header model: adapter response capture is incomplete in $client" >&2
            exit 1
        fi
    done
done

cargo test -p cloud-sdk transport::header
cargo test -p cloud-sdk transport::content_type
cargo test -p cloud-sdk operation::response_tests
cargo test -p cloud-sdk --doc
cargo test -p cloud-sdk-hetzner prepared::
cargo test -p cloud-sdk-testkit --all-features
cargo test -p cloud-sdk-reqwest --all-features shared::headers
cargo test -p cloud-sdk-reqwest --all-features \
    asynchronous::tests::lifecycle::async_send_future_stays_within_explicit_state_budget

echo "bounded HTTP header model checks passed."
