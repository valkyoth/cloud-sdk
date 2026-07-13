#!/usr/bin/env sh
set -eu

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/test-live-smoke-wrapper.py
scripts/test-hetzner-live-smoke-runner.py
scripts/test-platform-matrix.py
scripts/validate-file-lengths.sh
scripts/validate-modularity-policy.sh check
scripts/validate-security-policy.sh
scripts/check_serde_boundary.sh
scripts/check_sanitization_boundary.sh
scripts/check_testkit_boundary.sh
scripts/check_platform_matrix.sh --default-boundary
scripts/check_reqwest_boundary.sh
scripts/smoke_hetzner_live.sh --check
scripts/validate-release-metadata.sh
scripts/test-release-readiness.sh
scripts/test-sbom-freshness.sh
scripts/check_iana_ipv6_registry.py --local-only
scripts/test-iana-ipv6-registry.py
scripts/check_hetzner_api_drift.py --local-only
scripts/test-hetzner-api-drift.py
scripts/release_crates.py --check
scripts/test-release-crates.py
cargo package -p cloud-sdk --allow-dirty
cargo package -p cloud-sdk-hetzner --allow-dirty --features serde \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-reqwest.path="crates/cloud-sdk-reqwest"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"' \
    --config 'patch.crates-io.cloud-sdk-testkit.path="crates/cloud-sdk-testkit"'
cargo package -p cloud-sdk-reqwest --allow-dirty --all-features \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'
cargo package -p cloud-sdk-sanitization --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"'
cargo package -p cloud-sdk-testkit --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"'
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
