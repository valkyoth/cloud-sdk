#!/usr/bin/env sh
set -eu

cleanup=crates/cloud-sdk/src/transport/cleanup.rs
response=crates/cloud-sdk/src/transport/response.rs
retained=crates/cloud-sdk/src/transport/retained.rs
workspace=crates/cloud-sdk/src/transport/workspace.rs
headers=crates/cloud-sdk/src/transport/header/response.rs
content_type=crates/cloud-sdk/src/transport/content_type.rs
strict_json=crates/cloud-sdk-hetzner/src/serde/strict_json.rs
model_guard=crates/cloud-sdk-hetzner/src/serde/models/wipe_string.rs
model_root=crates/cloud-sdk-hetzner/src/serde/models.rs
dns_zone=crates/cloud-sdk-hetzner/src/serde/models/dns/zone/parser.rs
dns_rrset=crates/cloud-sdk-hetzner/src/serde/models/dns/rrset.rs

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
    'headers: ResponseHeaders<' \
    'ResponseHeaders::new(header_storage)' \
    'sanitize_response_storage(self.writer.storage, self.additive)'; do
    if ! grep -Fq "$required" "$response"; then
        echo "response cleanup: missing response owner contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub struct RetainedResponseMetadata' \
    "request_id: SecretBuffer<'storage>" \
    'sanitize_value(&mut self.request_id_len)'; do
    if ! grep -Fq "$required" "$retained"; then
        echo "response cleanup: missing retained-state contract $required" >&2
        exit 1
    fi
done

if grep -Eq 'bytes: \[u8;|request_id: \[u8;' "$headers" "$retained"; then
    echo "response cleanup: sensitive response bytes use movable fixed arrays" >&2
    exit 1
fi

for required in \
    "pub struct ResponseContentType<'a>" \
    "value: ContentType<'a>"; do
    if ! grep -Fq "$required" "$content_type"; then
        echo "response cleanup: response content type is not a borrowed stable view" >&2
        exit 1
    fi
done

if grep -Fq 'content_type: Option<ResponseContentType' "$response"; then
    echo "response cleanup: movable response metadata retains content-type bytes" >&2
    exit 1
fi

for required in \
    'struct ProtectedKey(String)' \
    'sanitize_string(&mut self.0)'; do
    if ! grep -Fq "$required" "$strict_json"; then
        echo "response cleanup: missing protected JSON-key contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub(super) struct WipeString(String)' \
    'pub(super) struct WipeStrings(Vec<String>)' \
    'sanitize_string(&mut self.0)' \
    'sanitize_string(value)'; do
    if ! grep -Fq "$required" "$model_guard"; then
        echo "response cleanup: missing fallible model-parser guard $required" >&2
        exit 1
    fi
done

for contract in \
    "$model_root:let mut labels = Self(Vec::new())" \
    "$model_root:let key = WipeString::new(" \
    "$dns_zone:let name = WipeString::new(" \
    "$dns_zone:let address = WipeString::new(" \
    "$dns_zone:let mut output = WipeStrings::with_capacity(values.len())?" \
    "$dns_rrset:let id = WipeString::new(" \
    "$dns_rrset:let raw = WipeString::new(" \
    "$dns_rrset:let value = WipeString::new("; do
    file=${contract%%:*}
    required=${contract#*:}
    if ! grep -Fq "$required" "$file"; then
        echo "response cleanup: missing error-path ownership contract $required" >&2
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
    'pub fn retain_metadata_into'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/operation; then
        echo "response cleanup: missing request-ID policy contract $required" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features --test response_cleanup
cargo test --locked -p cloud-sdk --all-features transport::retained
cargo test --locked -p cloud-sdk --all-features transport::workspace
cargo test --locked -p cloud-sdk-hetzner --all-features serde::checked
cargo test --locked -p cloud-sdk-hetzner --all-features \
    serde::strict_json::tests::object_keys_use_capacity_wiping_storage
cargo check --locked --manifest-path fuzz/Cargo.toml --bin checked_response

echo "mandatory response cleanup checks passed."
