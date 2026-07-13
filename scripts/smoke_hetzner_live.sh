#!/usr/bin/env sh
set -eu

usage() {
    echo "usage: scripts/smoke_hetzner_live.sh --check|--prepare|--read-only" >&2
    exit 2
}

artifact_dir="target/cloud-sdk-live-smoke"
artifact="$artifact_dir/live_smoke"
manifest="$artifact_dir/live_smoke.manifest"

reject_credential_environment() {
    if [ -n "${CLOUD_SDK_HETZNER_TOKEN_FILE:-}" ]; then
        echo "live smoke: token file must not be available during Cargo execution" >&2
        exit 2
    fi
    if [ -n "${CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE:-}" ]; then
        echo "live smoke: destructive opt-in is forbidden" >&2
        exit 2
    fi
}

require_clean_worktree() {
    if [ -n "$(git status --porcelain)" ]; then
        echo "live smoke: clean worktree required" >&2
        exit 1
    fi
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    elif command -v shasum >/dev/null 2>&1; then
        shasum -a 256 "$1" | awk '{print $1}'
    else
        echo "live smoke: SHA-256 tool unavailable" >&2
        return 1
    fi
}

prepare_artifact() {
    reject_credential_environment
    require_clean_worktree

    mkdir -p "$artifact_dir"
    messages="$(mktemp "$artifact_dir/cargo-messages.XXXXXX")"
    staged_artifact="$(mktemp "$artifact_dir/live-smoke.XXXXXX")"
    staged_manifest="$(mktemp "$artifact_dir/manifest.XXXXXX")"
    cleanup() {
        rm -f "$messages" "$staged_artifact" "$staged_manifest"
    }
    trap cleanup EXIT HUP INT TERM

    cargo test -p cloud-sdk-hetzner --test live_smoke --all-features \
        --no-run --message-format=json-render-diagnostics >"$messages"
    test_binary="$(scripts/find-cargo-test-executable.py "$messages" live_smoke)"
    if [ ! -f "$test_binary" ]; then
        echo "live smoke: Cargo test executable is not a regular file" >&2
        exit 1
    fi

    install -m 0555 -- "$test_binary" "$staged_artifact"
    digest="$(sha256_file "$staged_artifact")"
    reviewed_head="$(git rev-parse HEAD)"
    {
        printf 'format=1\n'
        printf 'commit=%s\n' "$reviewed_head"
        printf 'sha256=%s\n' "$digest"
    } >"$staged_manifest"
    chmod 0444 "$staged_manifest"

    mv -f -- "$staged_artifact" "$artifact"
    mv -f -- "$staged_manifest" "$manifest"
    trap - EXIT HUP INT TERM
    cleanup
    echo "live smoke: sealed executable prepared for $reviewed_head"
}

run_read_only() {
    token_file="${CLOUD_SDK_HETZNER_TOKEN_FILE:-}"
    destructive="${CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE:-}"
    unset CLOUD_SDK_HETZNER_TOKEN_FILE CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE

    if [ -z "$token_file" ]; then
        echo "live smoke: CLOUD_SDK_HETZNER_TOKEN_FILE is required" >&2
        exit 2
    fi
    if [ -n "$destructive" ]; then
        echo "live smoke: destructive opt-in is forbidden in read-only mode" >&2
        exit 2
    fi
    require_clean_worktree
    if [ ! -x "$artifact" ] || [ -w "$artifact" ] || [ ! -f "$manifest" ]; then
        echo "live smoke: sealed executable unavailable; run --prepare without a token" >&2
        exit 1
    fi

    manifest_format="$(sed -n 's/^format=//p' "$manifest")"
    manifest_commit="$(sed -n 's/^commit=//p' "$manifest")"
    manifest_digest="$(sed -n 's/^sha256=//p' "$manifest")"
    current_head="$(git rev-parse HEAD)"
    actual_digest="$(sha256_file "$artifact")"
    if [ "$manifest_format" != "1" ] \
        || [ "$manifest_commit" != "$current_head" ] \
        || [ "$manifest_digest" != "$actual_digest" ]; then
        echo "live smoke: sealed executable verification failed; run --prepare again" >&2
        exit 1
    fi

    exec /usr/bin/env -i \
        PATH=/usr/bin:/bin \
        CLOUD_SDK_HETZNER_LIVE_MODE=read-only \
        CLOUD_SDK_HETZNER_TOKEN_FILE="$token_file" \
        "$artifact" read_only_catalog_smoke --exact --ignored --nocapture --test-threads=1
}

mode="${1:-}"
if [ "$#" -ne 1 ]; then
    usage
fi

case "$mode" in
--check)
    reject_credential_environment
    cargo test -p cloud-sdk-hetzner --test live_smoke --all-features
    ;;
--prepare)
    prepare_artifact
    ;;
--read-only)
    run_read_only
    ;;
*)
    usage
    ;;
esac
