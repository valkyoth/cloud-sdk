#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/test-aws-lc-build-policy.py
scripts/check_doc_links.sh
scripts/test-doc-links.py
scripts/test-live-smoke-wrapper.py
scripts/test-hetzner-live-smoke-runner.py
scripts/test-platform-matrix.py
scripts/test-latest-tools.py
scripts/test-dependency-review.py
scripts/check_dependency_review.py v0.65.0 docs/DEPENDENCY_REVIEW_0.66.0.md
scripts/test-publishable-readmes.py
scripts/check-release-plan-structure.py
scripts/test-release-plan-structure.py
scripts/test-provider-identities.py
scripts/check_provider_identities.sh
scripts/check_http_method_domain.sh
scripts/check_request_targets.sh
scripts/check_pagination_strategies.sh
scripts/check_quota_strategies.sh
scripts/check_retry_strategies.sh
scripts/check_execution_permits.sh
scripts/check_client_kernel.sh
scripts/check_payload_free_diagnostics.sh
scripts/check_workflow_drivers.sh
scripts/check_local_async.sh
scripts/check_streaming.sh
scripts/check_incremental_decoding.sh
scripts/check_header_model.sh
scripts/check_response_provenance.sh
scripts/check_response_cleanup.sh
scripts/check_atomic_encoders.sh
scripts/check_raw_http_executor.sh
scripts/check_bearer_authentication.sh
scripts/check_basic_and_signing.sh
scripts/check-provider-capabilities.py
scripts/test-provider-capabilities.py
scripts/check-custom-endpoint-docs.py
scripts/test-custom-endpoint-docs.py
scripts/test-fuzz-aws-lc-tree.py
scripts/check_fuzz_harness.sh --metadata
scripts/validate-file-lengths.sh
scripts/validate-modularity-policy.sh check
scripts/validate-security-policy.sh
scripts/check_serde_boundary.sh
scripts/check_sanitization_boundary.sh
scripts/check_testkit_boundary.sh
scripts/check_platform_matrix.sh --default-boundary
scripts/check_reqwest_boundary.sh
scripts/check_reqwest_webpki_roots_boundary.sh
scripts/check_reqwest_fips_boundary.sh
scripts/smoke_hetzner_live.sh --check
scripts/check_publishable_readmes.sh
scripts/validate-release-metadata.sh
scripts/test-release-readiness.sh
scripts/test-complete-spdx-sbom.py
scripts/test-sbom-freshness.sh
scripts/check_iana_ipv6_registry.py --local-only
scripts/test-iana-ipv6-registry.py
scripts/check_hetzner_api_drift.py --local-only
scripts/test-hetzner-api-drift.py
scripts/test-generate-cloud-model-schema.py
scripts/check_dns_response_models.sh
scripts/check_security_response_models.sh
scripts/check_provider_drift.sh
scripts/check_ovhcloud_authority_conformance.sh
scripts/check_ovhcloud_header_conformance.sh
scripts/check_ovhcloud_task_conformance.sh
scripts/check_ovhcloud_execution_probe.sh
scripts/check_hetzner_vertical_slices.sh
scripts/check_api_matrix_coverage.py
scripts/test-api-matrix-coverage.py
cargo clippy --manifest-path tools/prepared-coverage-check/Cargo.toml \
    --locked --all-targets -- -D warnings
cargo test --manifest-path tools/prepared-coverage-check/Cargo.toml --locked
cargo run --quiet --locked \
    --manifest-path tools/prepared-coverage-check/Cargo.toml \
    --bin fail-closed-test-check -- \
    crates \
    fuzz/tests \
    tests/reqwest-feature-unification/src
scripts/check_prepared_operation_coverage.py
scripts/check_response_operation_coverage.py
scripts/generate_operation_associations.py --check
scripts/test-operation-associations.py
scripts/check_hetzner_wire_migration.py
scripts/test-response-operation-coverage.py
scripts/test-generate-response-operations.py
scripts/test-prepared-operation-coverage.py
scripts/test-hetzner-wire-migration.py
scripts/release_crates.py --check
scripts/validate-release-train.py
scripts/test-release-crates.py
scripts/test-release-train.py
scripts/test-release-state.py
cargo package -p cloud-sdk --allow-dirty \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"'
cargo package -p cloud-sdk-hetzner --allow-dirty --features serde \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"' \
    --config 'patch.crates-io.cloud-sdk-reqwest.path="crates/cloud-sdk-reqwest"' \
    --config 'patch.crates-io.cloud-sdk-sanitization.path="crates/cloud-sdk-sanitization"' \
    --config 'patch.crates-io.cloud-sdk-testkit.path="crates/cloud-sdk-testkit"'
CARGO_TARGET_DIR=/dev/null scripts/check_packaged_reqwest_tests.sh
cargo package -p cloud-sdk-sanitization --allow-dirty
cargo package -p cloud-sdk-testkit --allow-dirty \
    --config 'patch.crates-io.cloud-sdk.path="crates/cloud-sdk"'
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace
cargo test --workspace --doc --all-features
cargo test --workspace --all-features
