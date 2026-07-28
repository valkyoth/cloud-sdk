#!/usr/bin/env sh
set -eu

cleanup=crates/cloud-sdk/src/transport/cleanup.rs
response=crates/cloud-sdk/src/transport/response.rs
retained=crates/cloud-sdk/src/transport/retained.rs
workspace=crates/cloud-sdk/src/transport/workspace.rs

if find crates fuzz/fuzz_targets -type f -name '*.rs' -exec \
    grep -HnE '\.fill\(0(_u8)?\)' {} +; then
    echo "response cleanup: ordinary zero fill bypasses the audited primitive" >&2
    exit 1
fi

for required in \
    'cloud_sdk_sanitization::sanitize_bytes(storage)' \
    'MandatoryFinalClear { storage }' \
    'cloud_sdk_sanitization::sanitize_bytes(self.storage)'; do
    if ! grep -Fq "$required" "$cleanup"; then
        echo "response cleanup: missing mandatory cleanup contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub fn with_additive_sanitizer' \
    'sanitize_response_storage(self.writer.storage, self.additive)'; do
    if ! grep -Fq "$required" "$response"; then
        echo "response cleanup: missing response owner contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub struct RetainedResponseMetadata' \
    'sanitize_bytes(&mut self.request_id)' \
    'sanitize_value(&mut self.request_id_len)'; do
    if ! grep -Fq "$required" "$retained"; then
        echo "response cleanup: missing retained-state contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub struct ResponseDecodeWorkspace' \
    'sanitize_bytes(&mut self.decoder)' \
    'sanitize_bytes(&mut self.cursor)' \
    'sanitize_bytes(&mut self.provider_link)'; do
    if ! grep -Fq "$required" "$workspace"; then
        echo "response cleanup: missing workspace contract $required" >&2
        exit 1
    fi
done

for required in \
    'RequestIdPolicy::Retain' \
    'RequestIdPolicy::Protected' \
    'RequestIdPolicy::Discard' \
    'pub fn retain_metadata'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/operation; then
        echo "response cleanup: missing request-ID policy contract $required" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features --test response_cleanup
cargo test --locked -p cloud-sdk --all-features transport::retained
cargo test --locked -p cloud-sdk --all-features transport::workspace
cargo test --locked -p cloud-sdk-hetzner --all-features serde::checked
cargo check --locked --manifest-path fuzz/Cargo.toml --bin checked_response

echo "mandatory response cleanup checks passed."
