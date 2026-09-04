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
scripts/checks.sh

if [ "$(git rev-parse HEAD)" != "$reviewed_head" ]; then
    echo "release gate: HEAD changed while checks were running" >&2
    exit 1
fi

scripts/validate-release-readiness.sh v1.1.0
