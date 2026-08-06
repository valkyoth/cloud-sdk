#!/bin/sh
set -eu

cd "$(dirname "$0")/.."

python3 scripts/check_ovhcloud_authority_conformance.py
python3 scripts/test-ovhcloud-authority-conformance.py
cargo test --locked -p cloud-sdk --test ovhcloud_authority_conformance
cargo test --locked -p cloud-sdk-reqwest --features blocking-rustls \
    shared::credentials::tests
cargo test --locked -p cloud-sdk-reqwest --features blocking-rustls \
    blocking::tests::redirects_are_not_followed_or_admitted
cargo test --locked -p cloud-sdk-reqwest --features blocking-rustls \
    blocking::tests::lifecycle::blocking_expiring_refresh
cargo test --locked -p cloud-sdk-reqwest --features async-rustls \
    shared::credentials::tests
cargo test --locked -p cloud-sdk-reqwest --features async-rustls \
    asynchronous::tests::async_redirect_is_not_followed_or_admitted
cargo test --locked -p cloud-sdk-reqwest --features async-rustls \
    asynchronous::tests::lifecycle::async_expiring_rotation

echo "OVHcloud authority and OAuth conformance passed."
