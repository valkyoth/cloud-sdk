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

raw_parser=crates/cloud-sdk-reqwest/src/shared/raw.rs
raw_engine=crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs
for required in \
    'inspect_response_head(' \
    'policy.admits_header(' \
    '.try_push('; do
    if ! grep -Fq "$required" "$raw_parser"; then
        echo "header model: raw response admission is incomplete" >&2
        exit 1
    fi
done
for required in \
    'response_writer.headers_mut()' \
    'inspect_response_head('; do
    if ! grep -Fq "$required" "$raw_engine"; then
        echo "header model: raw adapter response capture is incomplete" >&2
        exit 1
    fi
done
for client in \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs; do
    if ! grep -Fq '.execute_authenticated(' "$client"; then
        echo "header model: authenticated client bypasses raw capture in $client" >&2
        exit 1
    fi
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
