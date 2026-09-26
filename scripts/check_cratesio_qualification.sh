#!/usr/bin/env sh
set -eu

. scripts/enforce_bundled_aws_lc.sh

# All mutation tests are local fixtures. No live credentials are consumed here.
scripts/checks.sh
scripts/check_fuzz_harness.sh --smoke
scripts/check_rust_version_matrix.sh
scripts/check_platform_matrix.sh --all
scripts/check_packaged_feature_graphs.sh
python3 scripts/check_cratesio_archives.py
scripts/check_sbom_freshness.sh
scripts/check_rustsec_advisories.sh
cargo deny check
python3 scripts/check_cratesio_source_lock.py --fetch
python3 scripts/check_cratesio_drift.py --fetch
printf '%s\n' "crates.io qualification passed; stop for pentest and GitHub acceptance."
