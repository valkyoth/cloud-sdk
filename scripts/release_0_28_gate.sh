#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

if [ -n "$(git status --porcelain=v1 --untracked-files=all)" ]; then
    echo "release gate: working tree is not clean" >&2
    exit 1
fi

scripts/validate-release-readiness.sh v0.28.0
reviewed_head="$(git rev-parse HEAD)"
scripts/checks.sh
scripts/check-provider-capabilities.py
scripts/check-custom-endpoint-docs.py
scripts/check_api_matrix_coverage.py
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
scripts/check_iana_ipv6_registry.py --fetch
scripts/check_fuzz_harness.sh --build
scripts/check_fuzz_harness.sh --smoke
scripts/check_sbom_freshness.sh

if ! cargo deny --version >/dev/null 2>&1; then
    echo "release gate: cargo-deny not installed; install the pinned version before tagging" >&2
    exit 1
fi
cargo deny check
cargo deny --manifest-path tests/reqwest-feature-unification/Cargo.toml \
    --config deny.toml --locked check advisories licenses sources
cargo deny --manifest-path fuzz/Cargo.toml \
    --config deny.toml --locked check advisories licenses sources

if ! cargo audit --version >/dev/null 2>&1; then
    echo "release gate: cargo-audit not installed; install the pinned version before tagging" >&2
    exit 1
fi
cargo audit
cargo audit --no-fetch --file tests/reqwest-feature-unification/Cargo.lock
cargo audit --no-fetch --file fuzz/Cargo.lock

if [ "$(git rev-parse HEAD)" != "$reviewed_head" ]; then
    echo "release gate: HEAD changed while checks were running" >&2
    exit 1
fi
scripts/validate-release-readiness.sh v0.28.0
