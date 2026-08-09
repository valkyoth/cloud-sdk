#!/usr/bin/env sh
set -eu

# Compatibility entry point retained for historical release gates. Active
# releases keep FIPS absent until the separately reviewed Brynja boundary is
# ready.
exec scripts/check_fips_deferred.py
