#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

cargo fmt --all --check
scripts/check_shell_syntax.sh
scripts/test-aws-lc-build-policy.py
scripts/check_doc_links.sh
scripts/test-doc-links.py
scripts/check_robot_live_smoke.py
scripts/test-robot-live-smoke.py
scripts/test-live-smoke-wrapper.py
scripts/test-hetzner-live-smoke-runner.py
scripts/test-platform-matrix.py
scripts/check_platform_contract.py
scripts/check_native_build_boundary.py
scripts/test-native-build-boundary.py
scripts/test-latest-tools.py
scripts/test-rustsec-advisories.py
scripts/test-dependency-review.py
scripts/check_dependency_review.py v0.97.0 0.98.0 docs/DEPENDENCY_REVIEW.md
scripts/check_review_digests.py
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
scripts/check_hetzner_client_foundation.sh
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
scripts/check_robot_api_lock.py
scripts/test-robot-api-lock.py
scripts/check_robot_form_codec.sh
scripts/check_robot_credentials.sh
scripts/check_robot_error_protocol.sh
scripts/check_robot_servers.sh
scripts/check_robot_cancellations.sh
scripts/check_robot_ips.sh
scripts/check_robot_subnets.sh
scripts/check_robot_resets.sh
scripts/check_robot_failovers.sh
scripts/check_robot_wol.sh
scripts/check_robot_boot.sh
scripts/check_robot_rdns.sh
scripts/check_robot_traffic.sh
python3 scripts/test-robot-traffic.py
scripts/check_robot_ssh_keys.sh
python3 scripts/test-robot-ssh-keys.py
scripts/check_robot_firewalls.sh
python3 scripts/test-robot-firewalls.py
scripts/check_robot_vswitches.sh
python3 scripts/test-robot-vswitches.py
scripts/check_robot_ordering.sh
python3 scripts/test-robot-ordering.py
scripts/check_robot_transactions.sh
python3 scripts/test-robot-transactions.py
scripts/check_robot_order_mutations.sh
python3 scripts/test-robot-order-mutations.py
scripts/check_robot_clients.sh
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
scripts/check_fips_deferred.py
scripts/test-fips-deferred.py
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
scripts/test-hetzner-metadata-contracts.py
python3 scripts/generate_request_contract_inventory.py
python3 scripts/test-request-contract-inventory.py
scripts/check_hetzner_changelog.py --local-only
scripts/test-hetzner-changelog.py
scripts/test-generate-cloud-model-schema.py
scripts/check_dns_response_models.sh
scripts/check_security_response_models.sh
scripts/check_storage_response_models.sh
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
scripts/generate_operation_associations.py --check
scripts/generate_typed_operation_bindings.py --check
scripts/generate_cloud_client_methods.py --check
scripts/generate_dns_client_methods.py --check
scripts/generate_security_client_methods.py --check
scripts/generate_storage_client_methods.py --check
scripts/test-cloud-client-methods.py
scripts/test-dns-client-methods.py
scripts/test-security-client-methods.py
scripts/test-storage-client-methods.py
scripts/test-operation-associations.py
scripts/test-typed-operation-bindings.py
scripts/check_typed_operation_bindings.py
scripts/check_cloud_client_methods.sh
scripts/check_dns_client_methods.sh
scripts/check_security_client_methods.sh
scripts/check_storage_client_methods.sh
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
