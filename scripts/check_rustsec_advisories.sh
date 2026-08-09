#!/usr/bin/env sh
set -eu

tmp_base="${TMPDIR:-/tmp}"
audit_root="$(mktemp -d "${tmp_base}/cloud-sdk-rustsec.XXXXXX")"
audit_db="${audit_root}/advisory-db"
trap 'rm -rf -- "$audit_root"' EXIT HUP INT TERM

# A fresh checkout prevents deleted RustSec draft files from surviving a fetch
# as untracked files and poisoning a later advisory scan.
cargo audit --db "$audit_db"
cargo audit --db "$audit_db" --no-fetch \
    --file tests/reqwest-feature-unification/Cargo.lock
cargo audit --db "$audit_db" --no-fetch --file fuzz/Cargo.lock
cargo audit --db "$audit_db" --no-fetch \
    --file tools/prepared-coverage-check/Cargo.lock
