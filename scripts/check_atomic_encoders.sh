#!/usr/bin/env sh
set -eu

require_text() {
    file="$1"
    text="$2"
    if ! grep -Fq "$text" "$file"; then
        echo "atomic encoders: $file is missing $text" >&2
        exit 1
    fi
}

require_text crates/cloud-sdk/src/buffer/encoder.rs \
    "pub fn encode_snapshot_bounded"
require_text crates/cloud-sdk/src/buffer/encoder.rs \
    "let required = measure_snapshot_bounded"
require_text crates/cloud-sdk/src/buffer/encoder.rs \
    "SnapshotEncoder::verifying"
require_text crates/cloud-sdk/src/buffer/encoder.rs "sanitize_bytes(target)"
require_text crates/cloud-sdk/src/operation/storage.rs \
    "pub struct PreparationStorageGuard"
require_text crates/cloud-sdk/src/operation/storage.rs \
    "pub struct OwnedPreparationStorage"
require_text crates/cloud-sdk-hetzner/src/prepared/json.rs "encode_snapshot_bounded"
require_text crates/cloud-sdk-hetzner/src/query.rs "measure_snapshot_bounded"

if grep -Eq '(^|[^A-Za-z])(DefaultHasher|SipHasher|std::hash|core::hash)' \
    crates/cloud-sdk/src/buffer/encoder.rs \
    crates/cloud-sdk-hetzner/src/prepared/json.rs; then
    echo "atomic encoders: request equivalence must not use non-cryptographic Hash" >&2
    exit 1
fi

cargo test -p cloud-sdk --all-features operation::storage::tests
cargo test -p cloud-sdk buffer::encoder::tests
cargo test -p cloud-sdk-hetzner --all-features \
    actions::tests::global_action_paths_and_queries_fail_closed_on_small_buffers
cargo test -p cloud-sdk-hetzner --all-features \
    prepared::tests::cleanup_guard_retains_storage_until_transport_use_then_clears_all_bytes

echo "atomic encoders: transactional and cleanup contracts passed"
