#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

temporary="$(mktemp -d "${TMPDIR:-/tmp}/cloud-sdk-packages.XXXXXX")"
trap 'rm -rf -- "$temporary"' EXIT HUP INT TERM
export CARGO_TARGET_DIR="$temporary/target"

cargo package --locked -p cloud-sdk --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'
cargo package --locked -p cloud-sdk-sanitization --allow-dirty --all-features
cargo package --locked -p cloud-sdk-testkit --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"'
cargo package --locked -p cloud-sdk-hetzner --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-reqwest.path="crates/cloud-sdk-reqwest"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"' \
    --config 'patch.crates-io.cloud-sdk-testkit.path="crates/cloud-sdk-testkit"'
cargo package --locked -p cloud-sdk-cratesio --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'
scripts/check_packaged_reqwest_tests.sh

printf '%s\n' "All six publishable package graphs passed Cargo verification."
