#!/usr/bin/env sh
set -eu

root_dir="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)"
cd "$root_dir"
. scripts/enforce_bundled_aws_lc.sh

portable_targets="
x86_64-unknown-linux-gnu
aarch64-unknown-linux-gnu
x86_64-pc-windows-msvc
x86_64-unknown-freebsd
x86_64-apple-darwin
aarch64-apple-darwin
aarch64-linux-android
aarch64-apple-ios
wasm32-unknown-unknown
thumbv7em-none-eabihf
"

usage() {
    echo "usage: $0 --portable TARGET | --native | --default-boundary | --all" >&2
    exit 2
}

require_no_extra_arguments() {
    if [ "$#" -ne 0 ]; then
        usage
    fi
}

is_portable_target() {
    candidate="$1"
    for target in $portable_targets; do
        if [ "$candidate" = "$target" ]; then
            return 0
        fi
    done
    return 1
}

require_installed_target() {
    target="$1"
    if ! command -v rustup >/dev/null 2>&1; then
        echo "platform matrix: rustup not found on PATH" >&2
        exit 2
    fi
    if ! installed_targets="$(rustup target list --installed)"; then
        echo "platform matrix: rustup could not list installed targets" >&2
        exit 2
    fi
    if ! printf '%s\n' "$installed_targets" | grep -Fxq "$target"; then
        echo "platform matrix: Rust target is not installed: $target" >&2
        echo "install it with: rustup target add $target" >&2
        exit 2
    fi
}

check_portable_target() {
    target="$1"
    if ! is_portable_target "$target"; then
        echo "platform matrix: unsupported portable target: $target" >&2
        exit 2
    fi
    require_installed_target "$target"

    cargo check --locked --target "$target" --no-default-features \
        -p cloud-sdk \
        -p cloud-sdk-hetzner \
        -p cloud-sdk-sanitization \
        -p cloud-sdk-testkit
    cargo check --locked --target "$target" --no-default-features \
        -p cloud-sdk --features alloc
    cargo check --locked --target "$target" --no-default-features \
        -p cloud-sdk-sanitization --features alloc
    cargo check --locked --target "$target" --no-default-features \
        -p cloud-sdk-hetzner --features alloc
    cargo check --locked --target "$target" --no-default-features \
        -p cloud-sdk-hetzner --features serde
    cargo check --locked --target "$target" --no-default-features \
        -p cloud-sdk-testkit --features alloc

    case "$target" in
    aarch64-linux-android|aarch64-apple-ios|wasm32-unknown-unknown|thumbv7em-none-eabihf)
        diagnostic="cloud-sdk-reqwest transport features are unsupported on this target"
        output="$(mktemp "${TMPDIR:-/tmp}/cloud-sdk-unsupported.XXXXXX")"
        if cargo check --locked --target "$target" --no-default-features \
            -p cloud-sdk-reqwest --features blocking-rustls >"$output" 2>&1; then
            rm -f -- "$output"
            echo "platform matrix: unsupported transport compiled for $target" >&2
            exit 1
        fi
        if ! grep -Fq "$diagnostic" "$output"; then
            cat "$output" >&2
            rm -f -- "$output"
            echo "platform matrix: missing unsupported transport diagnostic" >&2
            exit 1
        fi
        rm -f -- "$output"
        ;;
    esac
}

check_native() {
    cargo check --locked --all-targets --all-features \
        -p cloud-sdk \
        -p cloud-sdk-hetzner \
        -p cloud-sdk-sanitization \
        -p cloud-sdk-testkit
    cargo check --locked --all-targets --no-default-features \
        -p cloud-sdk-reqwest
    cargo check --locked --all-targets --no-default-features \
        -p cloud-sdk-reqwest --features std
    for feature in \
        blocking-rustls \
        blocking-rustls-webpki-roots \
        async-rustls; do
        cargo check --locked --all-targets --no-default-features \
            -p cloud-sdk-reqwest --features "$feature"
        cargo test --locked --no-default-features \
            -p cloud-sdk-reqwest --features "$feature"
    done
    cargo check --locked --all-targets --no-default-features \
        -p cloud-sdk-reqwest \
        --features blocking-rustls,blocking-rustls-webpki-roots,async-rustls
    cargo test --locked --all-features -p cloud-sdk-reqwest
    cargo test --locked -p cloud-sdk-hetzner --test live_smoke --all-features
}

check_default_boundary() {
    dependency_tree="$(
        cargo tree --locked --workspace --target all \
            --edges normal --prefix none
    )"
    allowed='^(cloud-sdk|cloud-sdk-hetzner|cloud-sdk-reqwest|cloud-sdk-sanitization|cloud-sdk-testkit|ovhcloud-v2-probe|sanitization|subtle) v'
    unexpected="$(
        printf '%s\n' "$dependency_tree" \
            | grep -E '^[^[:space:]]+ v' \
            | grep -Ev "$allowed" \
            || true
    )"
    if [ -n "$unexpected" ]; then
        echo "platform matrix: unexpected default dependency" >&2
        printf '%s\n' "$unexpected" >&2
        exit 1
    fi
}

mode="${1:-}"
case "$mode" in
--portable)
    if [ "$#" -ne 2 ]; then
        usage
    fi
    check_portable_target "$2"
    ;;
--native)
    shift
    require_no_extra_arguments "$@"
    check_native
    ;;
--default-boundary)
    shift
    require_no_extra_arguments "$@"
    check_default_boundary
    ;;
--all)
    shift
    require_no_extra_arguments "$@"
    check_default_boundary
    for target in $portable_targets; do
        check_portable_target "$target"
    done
    check_native
    ;;
*) usage ;;
esac
