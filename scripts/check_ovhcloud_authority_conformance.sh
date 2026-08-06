#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

python3 scripts/check_ovhcloud_authority_conformance.py
cargo test --locked -p cloud-sdk --test ovhcloud_authority_conformance
cargo test --locked -p cloud-sdk-reqwest --features blocking-rustls \
    shared::credentials::tests
cargo test --locked -p cloud-sdk-reqwest --features async-rustls \
    shared::credentials::tests

echo "OVHcloud authority and OAuth conformance passed."
