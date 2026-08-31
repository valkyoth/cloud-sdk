#!/usr/bin/env sh
set -eu

legacy_admission=docs/dependency-admission-ssh-key.md
if [ -e "$legacy_admission" ] || [ -L "$legacy_admission" ]; then
    echo "Hetzner security response evidence contains the obsolete ssh-key admission record" >&2
    exit 1
fi

default_tree="$(
    cargo tree --locked -p cloud-sdk-hetzner --no-default-features \
        --edges normal --prefix none
)"
if printf '%s\n' "$default_tree" | grep -Eq '^(base64-ng|md-5|sha2) v'; then
    echo "Hetzner security response model dependencies entered the default graph" >&2
    exit 1
fi

serde_tree="$(
    cargo tree --locked -p cloud-sdk-hetzner --features serde \
        --edges normal --prefix none
)"
for dependency in 'base64-ng v2.0.2' 'md-5 v0.11.0' 'sha2 v0.11.0'; do
    if ! printf '%s\n' "$serde_tree" | grep -Fqx "$dependency"; then
        echo "Hetzner security response model graph is missing $dependency" >&2
        exit 1
    fi
done
if printf '%s\n' "$serde_tree" | grep -Eq '^(ssh-key|zeroize) v'; then
    echo "Hetzner security response graph contains an unadmitted parser or cleanup dependency" >&2
    exit 1
fi

python3 scripts/test-generate-cloud-model-schema.py
cargo test -p cloud-sdk-hetzner --all-features \
    serde::checked_security_resource_tests
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::certificate
cargo test -p cloud-sdk-hetzner --all-features \
    serde::models::ssh_key
cargo test -p cloud-sdk-hetzner --all-features \
    serde::checked_ssh_key_algorithm_tests
cargo test -p cloud-sdk-hetzner --all-features --test live_smoke
cargo test --manifest-path fuzz/Cargo.toml --test cloud_special_response_seeds

echo "Hetzner security response models passed graph, source, identity, secret, live-harness, and fuzz-seed checks."
