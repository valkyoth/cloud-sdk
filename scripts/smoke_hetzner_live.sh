#!/usr/bin/env sh
set -eu

usage() {
    echo "usage: scripts/smoke_hetzner_live.sh --check|--prepare" >&2
    exit 2
}

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

mode="${1:-}"
if [ "$#" -ne 1 ]; then
    usage
fi
reject_credential_environment

script_dir=${0%/*}
if [ "$script_dir" = "$0" ]; then
    script_dir=.
fi
script_dir="$(CDPATH= cd -- "$script_dir" && pwd -P)" || exit 1
repo_root="$(CDPATH= cd -- "$script_dir/.." && pwd -P)" || exit 1
cd "$repo_root"

artifact_dir="$repo_root/target/cloud-sdk-live-smoke"
staging_dir="$artifact_dir/staging"

require_clean_worktree() {
    if [ -n "$(/usr/bin/git status --porcelain=v1 --untracked-files=all)" ]; then
        echo "live smoke: clean worktree required" >&2
        exit 1
    fi
}

sha256_file() {
    if [ -x /usr/bin/sha256sum ]; then
        line="$(/usr/bin/sha256sum -- "$1")" || return 1
    elif [ -x /usr/bin/shasum ]; then
        line="$(/usr/bin/shasum -a 256 -- "$1")" || return 1
    else
        echo "live smoke: trusted SHA-256 utility unavailable" >&2
        return 1
    fi
    digest=${line%% *}
    if [ "${#digest}" -ne 64 ]; then
        return 1
    fi
    case "$digest" in
    *[!0-9a-f]*) return 1 ;;
    esac
    printf '%s\n' "$digest"
}

prepare_artifact() {
    require_clean_worktree

    /usr/bin/mkdir -p "$artifact_dir" "$staging_dir"
    messages="$(/usr/bin/mktemp "$artifact_dir/cargo-messages.XXXXXX")"
    staged_artifact="$(/usr/bin/mktemp "$artifact_dir/live-smoke.XXXXXX")"
    staged_runner="$(/usr/bin/mktemp "$artifact_dir/runner.XXXXXX")"
    staged_launcher="$(/usr/bin/mktemp "$artifact_dir/launcher.XXXXXX")"
    staged_manifest="$(/usr/bin/mktemp "$artifact_dir/manifest.XXXXXX")"
    cleanup() {
        /usr/bin/rm -f \
            "$messages" "$staged_artifact" "$staged_runner" \
            "$staged_launcher" "$staged_manifest"
    }
    trap cleanup EXIT HUP INT TERM

    cargo test -p cloud-sdk-hetzner --test live_smoke --all-features \
        --no-run --message-format=json-render-diagnostics >"$messages"
    test_binary="$($repo_root/scripts/find-cargo-test-executable.py \
        "$messages" live_smoke)"
    if [ ! -f "$test_binary" ]; then
        echo "live smoke: Cargo test executable is not a regular file" >&2
        exit 1
    fi

    /usr/bin/install -m 0555 -- "$test_binary" "$staged_artifact"
    /usr/bin/install -m 0444 -- \
        "$repo_root/scripts/hetzner-live-smoke-runner.py" "$staged_runner"
    /usr/bin/install -m 0555 -- \
        "$repo_root/scripts/cloud-sdk-hetzner-smoke" "$staged_launcher"
    artifact_digest="$(sha256_file "$staged_artifact")"
    runner_digest="$(sha256_file "$staged_runner")"
    launcher_digest="$(sha256_file "$staged_launcher")"
    reviewed_head="$(/usr/bin/git rev-parse HEAD)"
    {
        printf 'format=2\n'
        printf 'commit=%s\n' "$reviewed_head"
        printf 'artifact_sha256=%s\n' "$artifact_digest"
        printf 'runner_sha256=%s\n' "$runner_digest"
        printf 'launcher_sha256=%s\n' "$launcher_digest"
    } >"$staged_manifest"
    /usr/bin/chmod 0444 "$staged_manifest"

    /usr/bin/mv -f -- "$staged_artifact" "$staging_dir/live_smoke"
    /usr/bin/mv -f -- "$staged_runner" "$staging_dir/runner.py"
    /usr/bin/mv -f -- "$staged_launcher" "$staging_dir/cloud-sdk-hetzner-smoke"
    /usr/bin/mv -f -- "$staged_manifest" "$staging_dir/manifest"
    trap - EXIT HUP INT TERM
    cleanup
    echo "live smoke: untrusted staging bundle prepared for $reviewed_head"
    echo "live smoke: install it through the documented privileged sealing step"
}

case "$mode" in
--check)
    cargo test -p cloud-sdk-hetzner --test live_smoke --all-features
    ;;
--prepare)
    prepare_artifact
    ;;
*)
    usage
    ;;
esac
