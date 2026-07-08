#!/usr/bin/env sh
set -eu

mode="${1:---local-only}"
case "$mode" in
--local-only)
    test -s docs/API_MATRIX.md
    test -s docs/SPEC_LOCK.md
    ;;
*)
    echo "usage: scripts/check_hetzner_upstream.sh --local-only" >&2
    exit 2
    ;;
esac
