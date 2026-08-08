#!/usr/bin/env sh
set -eu

default_tree="$(
    cargo tree --locked -p cloud-sdk-hetzner --no-default-features \
        --edges normal --prefix none
)"
if printf '%s\n' "$default_tree" | grep -Eq '^(md-5|ssh-key) v'; then
    echo "Hetzner security response model dependencies entered the default graph" >&2
    exit 1
fi

serde_tree="$(
    cargo tree --locked -p cloud-sdk-hetzner --features serde \
        --edges normal --prefix none
)"
for dependency in 'md-5 v0.11.0' 'ssh-key v0.6.7'; do
    if ! printf '%s\n' "$serde_tree" | grep -Fqx "$dependency"; then
        echo "Hetzner security response model graph is missing $dependency" >&2
        exit 1
    fi
done

python3 scripts/test-generate-cloud-model-schema.py
cargo test -p cloud-sdk-hetzner --all-features \
    serde::checked_security_resource_tests
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::certificate
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::ssh_key
cargo test -p cloud-sdk-hetzner --all-features --test live_smoke
cargo test --manifest-path fuzz/Cargo.toml --test cloud_special_response_seeds

echo "Hetzner security response models passed graph, source, identity, secret, live-harness, and fuzz-seed checks."
