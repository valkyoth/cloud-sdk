#!/usr/bin/env sh
set -eu

usage() {
    echo "usage: scripts/smoke_hetzner_live.sh --check|--read-only" >&2
    exit 2
}

mode="${1:-}"
if [ "$#" -ne 1 ]; then
    usage
fi

case "$mode" in
--check)
    cargo test -p cloud-sdk-hetzner --test live_smoke --all-features
    ;;
--read-only)
    if [ -z "${CLOUD_SDK_HETZNER_TOKEN_FILE:-}" ]; then
        echo "live smoke: CLOUD_SDK_HETZNER_TOKEN_FILE is required" >&2
        exit 2
    fi
    if [ -n "${CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE:-}" ]; then
        echo "live smoke: destructive opt-in is forbidden in read-only mode" >&2
        exit 2
    fi
    CLOUD_SDK_HETZNER_LIVE_MODE=read-only \
        cargo test -p cloud-sdk-hetzner --test live_smoke --all-features \
        read_only_catalog_smoke -- --exact --ignored --nocapture --test-threads=1
    ;;
*)
    usage
    ;;
esac
