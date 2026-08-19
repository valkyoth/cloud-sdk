#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

if [ -n "$(git status --porcelain=v1 --untracked-files=all)" ]; then
    echo "release gate: working tree is not clean" >&2
    exit 1
fi

scripts/validate-release-readiness.sh v0.98.0
reviewed_head="$(git rev-parse HEAD)"
scripts/checks.sh
scripts/check_platform_contract.py
scripts/check_native_build_boundary.py
scripts/test-native-build-boundary.py
scripts/check_packaged_feature_graphs.sh
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --all-features --no-deps
python3 scripts/generate_request_contract_inventory.py
python3 scripts/test-request-contract-inventory.py
python3 scripts/test-hetzner-metadata-contracts.py
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
scripts/check_robot_wire_fixture.py --fetch
scripts/check_robot_api_lock.py --fetch
scripts/check_basic_and_signing.sh
scripts/check_provider_identities.sh
scripts/check_endpoint_policy.sh
scripts/check_request_targets.sh
scripts/check_atomic_encoders.sh
scripts/check_response_cleanup.sh
scripts/check_response_provenance.sh
scripts/check_raw_http_executor.sh
scripts/check_http_method_domain.sh
scripts/check_provider_drift.sh
scripts/check_hetzner_vertical_slices.sh
scripts/check_testkit_boundary.sh
scripts/check_payload_free_diagnostics.sh
scripts/check_fips_deferred.py
scripts/check-provider-capabilities.py
scripts/check-custom-endpoint-docs.py
scripts/check_api_matrix_coverage.py
scripts/check_latest_tools.sh --fetch
scripts/check_doc_links.sh
cargo test --workspace --doc --all-features
scripts/check_platform_matrix.sh --all
scripts/check_rust_version_matrix.sh
scripts/check_hetzner_upstream.sh --local-only
scripts/check_hetzner_api_surface.sh --fetch
scripts/check_provider_drift.py --plugin provider-drift/plugins/normalized-json-v1.json \
    --lock provider-drift/providers/hetzner.lock.json \
    --observation provider-drift/providers/hetzner.observed.json --fetch-sources
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
scripts/check_rustsec_advisories.sh

if [ "$(git rev-parse HEAD)" != "$reviewed_head" ]; then
    echo "release gate: HEAD changed while checks were running" >&2
    exit 1
fi
scripts/validate-release-readiness.sh v0.98.0
