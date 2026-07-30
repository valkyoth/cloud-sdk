#!/usr/bin/env sh
set -eu

cargo check -p cloud-sdk --no-default-features
cargo test -p cloud-sdk --no-default-features authentication::signing

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
grep -Fq 'base64-ng = { version = "=1.3.9", default-features = false }' Cargo.toml

package_files="$(cargo package -p cloud-sdk-hetzner --allow-dirty --list)"
if printf '%s\n' "$package_files" | grep -Fq 'robot-wire'; then
    echo "Basic/signing: Robot fixture entered a publishable package" >&2
    exit 1
fi

scripts/check_robot_wire_fixture.py
scripts/test-robot-wire-fixture.py

echo "Basic authentication, signing input, and Robot wire checks passed."
