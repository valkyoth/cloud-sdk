#!/usr/bin/env sh
set -eu

cargo check -p cloud-sdk --no-default-features
cargo test -p cloud-sdk --no-default-features authentication::signing
cargo test -p cloud-sdk --features std authentication::signing

signing='crates/cloud-sdk/src/authentication/signing.rs'
for contract in \
    'cloud-sdk-signing-v2\0' \
    'fn encode_canonical_host' \
    'endpoint.canonical_host()' \
    'fn digest_algorithm(&self)' \
    'pub fn new_hashed' \
    'pub fn sign_into' \
    'pub struct SigningFreshness'; do
    if ! grep -Fq "$contract" "$signing"; then
        echo "Basic/signing: missing v2 signing contract $contract" >&2
        exit 1
    fi
done
if grep -Fq 'endpoint.host().as_bytes()' "$signing"; then
    echo "Basic/signing: presentation host entered canonical signing bytes" >&2
    exit 1
fi
for forbidden in \
    'cloud-sdk-signing-v1' \
    'pub struct SigningBodyDigest' \
    'pub fn sign_with'; do
    if grep -R -Fq "$forbidden" crates/cloud-sdk/src; then
        echo "Basic/signing: insecure or obsolete signing contract remains: $forbidden" >&2
        exit 1
    fi
done
context_source='crates/cloud-sdk/src/authentication/signing/context.rs'
if ! grep -Fq 'pub struct SigningContext' "$context_source"; then
    echo "Basic/signing: missing signing-context contract SigningContext" >&2
    exit 1
fi
for contract in SigningKeyId SigningDigestAlgorithm SigningAlgorithm; do
    if ! grep -Fq "    $contract," "$context_source"; then
        echo "Basic/signing: missing generated signing-context contract $contract" >&2
        exit 1
    fi
done
for contract in \
    'pub struct SignedRequest' \
    'pub enum SigningOutputError'; do
    if ! grep -Fq "$contract" \
        crates/cloud-sdk/src/authentication/signing/output.rs; then
        echo "Basic/signing: missing signing-output contract $contract" >&2
        exit 1
    fi
done

cargo test -p cloud-sdk-reqwest --no-default-features \
    --features blocking-rustls basic
cargo test -p cloud-sdk-reqwest --no-default-features \
    --features async-rustls basic
cargo test -p cloud-sdk-reqwest --all-features basic

default_tree="$(
    cargo tree --locked -p cloud-sdk-reqwest --no-default-features \
        --edges normal --prefix none
)"
if printf '%s\n' "$default_tree" | grep -Eq '^base64-ng v'; then
    echo "Basic/signing: base64-ng entered the default reqwest graph" >&2
    exit 1
fi

std_tree="$(
    cargo tree --locked -p cloud-sdk-reqwest --no-default-features \
        --features std --edges normal --prefix none
)"
if printf '%s\n' "$std_tree" | grep -Eq '^base64-ng v'; then
    echo "Basic/signing: base64-ng entered the std-only reqwest graph" >&2
    exit 1
fi

manifest='crates/cloud-sdk-reqwest/Cargo.toml'
grep -Fq 'base64-ng = { workspace = true, optional = true }' "$manifest"
grep -Fq '"dep:base64-ng"' "$manifest"
grep -Fq 'base64-ng = { version = "=2.0.1", default-features = false }' Cargo.toml

package_files="$(cargo package -p cloud-sdk-hetzner --allow-dirty --list)"
if printf '%s\n' "$package_files" | grep -Eq 'robot-(wire|api)'; then
    echo "Basic/signing: Robot fixture entered a publishable package" >&2
    exit 1
fi

scripts/check_robot_wire_fixture.py
scripts/test-robot-wire-fixture.py

echo "Basic authentication, signing input, and Robot wire checks passed."
