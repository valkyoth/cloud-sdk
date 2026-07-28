#!/usr/bin/env sh
set -eu

response_model=crates/cloud-sdk/src/transport/response.rs

if grep -R -n --include='*.rs' 'TransportResponse::new' \
    crates fuzz/fuzz_targets; then
    echo "response provenance: public response construction remains" >&2
    exit 1
fi

for required in \
    'pub struct ResponseBuffer' \
    'pub struct ResponseWriter' \
    'pub struct TransportResponse' \
    'fn from_commit' \
    "inspect: impl for<'response> FnOnce(TransportResponse" \
    'sanitize_response_storage(self.writer.storage)'; do
    if ! grep -Fq "$required" "$response_model"; then
        echo "response provenance: missing response contract $required" >&2
        exit 1
    fi
done

for transport in \
    crates/cloud-sdk/src/transport.rs \
    crates/cloud-sdk/src/transport/asynchronous.rs; do
    if ! grep -Fq 'response: &' "$transport" \
        || ! grep -Fq 'mut ResponseWriter' "$transport"; then
        echo "response provenance: transport does not receive sealed writer" >&2
        exit 1
    fi
done

for adapter in \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs \
    crates/cloud-sdk-testkit/src/mock.rs; do
    if ! grep -Fq '.commit(' "$adapter"; then
        echo "response provenance: writer commit missing from $adapter" >&2
        exit 1
    fi
    if ! grep -Fq '.is_committed()' "$adapter"; then
        echo "response provenance: precommitted writer check missing from $adapter" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features --test response_provenance
cargo test --locked -p cloud-sdk --doc
cargo test --locked -p cloud-sdk-testkit --all-features
cargo test --locked -p cloud-sdk-reqwest --all-features lifecycle
cargo check --locked --manifest-path fuzz/Cargo.toml --bin checked_response

echo "sealed response provenance checks passed."
