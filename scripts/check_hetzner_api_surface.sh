#!/usr/bin/env sh
set -eu

mode="${1:---local-only}"

case "$mode" in
--local-only)
    scripts/check_hetzner_api_drift.py --local-only
    scripts/check_robot_api_lock.py
    scripts/check_hetzner_changelog.py --local-only
    ;;
--fetch)
    scripts/check_hetzner_api_drift.py --fetch
    scripts/check_robot_api_lock.py --fetch
    scripts/check_hetzner_changelog.py --fetch
    ;;
*)
    echo "usage: scripts/check_hetzner_api_surface.sh [--local-only|--fetch]" >&2
    exit 2
    ;;
esac

printf '%s\n' 'All tracked Hetzner API sources are current.'
