#!/usr/bin/env sh
set -eu

python3 scripts/test-generate-cloud-model-schema.py
cargo test -p cloud-sdk-hetzner --all-features \
    serde::checked_security_resource_tests
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::certificate
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::ssh_key
cargo test -p cloud-sdk-hetzner --all-features --test live_smoke
cargo test --manifest-path fuzz/Cargo.toml --test cloud_special_response_seeds

echo "Hetzner security response models passed source, secret, live-harness, and fuzz-seed checks."
