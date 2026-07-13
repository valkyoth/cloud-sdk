#!/usr/bin/env sh
set -eu

root_dir="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd -P)"
cd "$root_dir"

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
        -p cloud-sdk \
        -p cloud-sdk-hetzner \
        -p cloud-sdk-testkit \
        --features cloud-sdk/alloc,cloud-sdk-hetzner/serde,cloud-sdk-testkit/alloc
}

check_native() {
    cargo check --locked --workspace --all-targets --all-features
}

check_default_boundary() {
    dependency_tree="$(
        cargo tree --locked --workspace --no-default-features \
            --edges normal --prefix none
    )"
    forbidden='^(reqwest|tokio|hyper|hyper-util|tower|tower-http|rustls|rustls-platform-verifier|aws-lc-rs|aws-lc-sys|mio|socket2|windows-sys) v'
    if printf '%s\n' "$dependency_tree" | grep -Eq "$forbidden"; then
        echo "platform matrix: default features activate an OS or transport dependency" >&2
        printf '%s\n' "$dependency_tree" | grep -E "$forbidden" >&2
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
