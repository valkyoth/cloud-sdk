#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

if [ -n "$(git status --porcelain=v1 --untracked-files=all)" ]; then
    echo "release gate: working tree is not clean" >&2
    exit 1
fi

scripts/validate-release-readiness.sh v0.65.0
reviewed_head="$(git rev-parse HEAD)"
scripts/checks.sh
scripts/check_dns_response_models.sh
scripts/test-generate-cloud-model-schema.py
scripts/check_provider_drift.sh
scripts/check_ovhcloud_authority_conformance.sh
scripts/check_ovhcloud_header_conformance.sh
scripts/check_ovhcloud_task_conformance.sh
scripts/check_ovhcloud_execution_probe.sh
scripts/check_hetzner_vertical_slices.sh
scripts/check_testkit_boundary.sh
scripts/check_payload_free_diagnostics.sh
scripts/check_workflow_drivers.sh
scripts/check_client_kernel.sh
scripts/check_incremental_decoding.sh
scripts/check_streaming.sh
scripts/check_local_async.sh
scripts/check_retry_strategies.sh
scripts/check_execution_permits.sh
scripts/check_quota_strategies.sh
scripts/check_hetzner_wire_migration.py
scripts/check_basic_and_signing.sh
scripts/check_robot_wire_fixture.py --fetch
scripts/check_bearer_authentication.sh
scripts/check_raw_http_executor.sh
scripts/check_atomic_encoders.sh
scripts/check_response_cleanup.sh
scripts/check_response_provenance.sh
scripts/check_header_model.sh
scripts/check_request_targets.sh
scripts/check_endpoint_policy.sh
scripts/check_http_method_domain.sh
scripts/check_provider_identities.sh
scripts/check-provider-capabilities.py
scripts/check-custom-endpoint-docs.py
scripts/check_api_matrix_coverage.py
scripts/check_prepared_operation_coverage.py
scripts/check_response_operation_coverage.py
scripts/generate_operation_associations.py --check
scripts/check_latest_tools.sh --fetch
scripts/check_doc_links.sh
scripts/test-doc-links.py
cargo test --workspace --doc --all-features
scripts/check_reqwest_webpki_roots_boundary.sh
scripts/check_reqwest_fips_boundary.sh
scripts/check_platform_matrix.sh --all
scripts/check_rust_version_matrix.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_drift.py --fetch
scripts/check_provider_drift.py --plugin provider-drift/plugins/normalized-json-v1.json \
    --lock provider-drift/providers/hetzner.lock.json \
    --observation provider-drift/providers/hetzner.observed.json --fetch-sources
scripts/check_provider_drift.py --plugin provider-drift/plugins/normalized-json-v1.json \
    --lock provider-drift/providers/ovhcloud-v2-probe.lock.json \
    --observation provider-drift/providers/ovhcloud-v2-probe.observed.json --fetch-sources
scripts/check_iana_ipv6_registry.py --fetch
scripts/check_fuzz_harness.sh --build
scripts/check_fuzz_harness.sh --smoke
scripts/check_sbom_freshness.sh

command -v cargo-deny >/dev/null 2>&1 || {
    echo "release gate: cargo-deny not installed" >&2
    exit 1
}
cargo deny check
cargo deny --manifest-path tests/reqwest-feature-unification/Cargo.toml \
    --config deny.toml --locked check advisories licenses sources
cargo deny --manifest-path fuzz/Cargo.toml --config deny.toml --locked \
    check advisories licenses sources
cargo deny --manifest-path tools/prepared-coverage-check/Cargo.toml \
    --config deny.toml --locked check advisories licenses sources

command -v cargo-audit >/dev/null 2>&1 || {
    echo "release gate: cargo-audit not installed" >&2
    exit 1
}
cargo audit
cargo audit --no-fetch --file tests/reqwest-feature-unification/Cargo.lock
cargo audit --no-fetch --file fuzz/Cargo.lock
cargo audit --no-fetch --file tools/prepared-coverage-check/Cargo.lock

if [ "$(git rev-parse HEAD)" != "$reviewed_head" ]; then
    echo "release gate: HEAD changed while checks were running" >&2
    exit 1
fi
scripts/validate-release-readiness.sh v0.65.0
