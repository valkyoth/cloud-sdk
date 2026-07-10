#!/usr/bin/env sh
set -eu

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/validate-file-lengths.sh
scripts/validate-modularity-policy.sh check
scripts/validate-security-policy.sh
scripts/validate-release-metadata.sh
scripts/test-pentest-binding.py
scripts/test-pentest-signing.py
scripts/check_iana_ipv6_registry.py --local-only
scripts/test-iana-ipv6-registry.py
scripts/check_hetzner_api_drift.py --local-only
scripts/test-hetzner-api-drift.py
scripts/release_crates.py --check
scripts/test-release-crates.py
cargo package -p cloud-sdk --allow-dirty
cargo package -p cloud-sdk-hetzner --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"'
cargo package -p cloud-sdk-hetzner-reqwest --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-hetzner.path="crates/cloud-sdk-hetzner"'
cargo package -p cloud-sdk-hetzner-sanitization --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-hetzner.path="crates/cloud-sdk-hetzner"'
cargo package -p cloud-sdk-hetzner-testkit --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-hetzner.path="crates/cloud-sdk-hetzner"'
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
