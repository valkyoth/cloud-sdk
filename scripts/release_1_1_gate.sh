#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

scripts/validate-release-readiness.sh v1.1.0
reviewed_head="$(git rev-parse HEAD)"
scripts/check_latest_tools.sh --fetch
scripts/check_exact_dependency_pins.py --fetch
scripts/check_cratesio_drift.py --fetch
scripts/check_cratesio_source_lock.py --fetch
scripts/check_cratesio_endpoints.py --fetch
python3 scripts/check_cratesio_request_policy.py --fetch
python3 scripts/generate_cratesio_discovery_fixtures.py
python3 scripts/generate_cratesio_catalog.py
python3 scripts/generate_cratesio_personal.py
python3 scripts/generate_cratesio_tokens.py
python3 scripts/test-cratesio-settings.py
python3 scripts/generate_cratesio_settings.py
python3 scripts/test-cratesio-ownership.py
python3 scripts/check_cratesio_ownership.py
python3 scripts/test-cratesio-yank.py
python3 scripts/check_cratesio_yank.py
python3 scripts/test-cratesio-publish.py
python3 scripts/generate_cratesio_publish.py
python3 scripts/test-cratesio-trusted-publishing.py
python3 scripts/generate_cratesio_trusted_publishing.py
cargo test --locked --release -p cloud-sdk-cratesio --no-default-features --features std \
    trusted_publishing::tests::assertions::preflight_on_bounded_stack -- --exact
scripts/check_cratesio_candidate.sh
scripts/check_release_governance.py --live
scripts/check_release_provenance.py
RUSTDOCFLAGS="-D warnings" cargo doc --locked --workspace --all-features --no-deps
scripts/check_robot_wire_fixture.py --fetch
scripts/check_iana_ipv6_registry.py --fetch
for manifest in fuzz/Cargo.toml tests/reqwest-feature-unification/Cargo.toml tools/prepared-coverage-check/Cargo.toml; do
    cargo deny --manifest-path "$manifest" --config deny.toml --locked check advisories licenses sources
done
python3 scripts/check_cratesio_execution_coverage.py

if [ "$(git rev-parse HEAD)" != "$reviewed_head" ]; then
    echo "release gate: HEAD changed while checks were running" >&2
    exit 1
fi

scripts/validate-release-readiness.sh v1.1.0
