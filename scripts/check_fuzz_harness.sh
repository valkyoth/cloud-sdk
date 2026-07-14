#!/usr/bin/env sh
set -eu

mode="${1:---metadata}"
toolchain="nightly-2026-07-13"
cargo_fuzz_version="0.13.2"
targets="buffer_writers request_targets action_requests labels_dns pagination action_polling response_envelopes"

check_layout() {
    cargo fmt --manifest-path fuzz/Cargo.toml -- --check
    cargo metadata --manifest-path fuzz/Cargo.toml --locked --no-deps \
        --format-version 1 >/dev/null

    manifest_targets="$(
        sed -n 's/^name = "\([a-z_]*\)"$/\1/p' fuzz/Cargo.toml |
            tail -n 7 |
            tr '\n' ' ' |
            sed 's/ $//'
    )"
    if [ "$manifest_targets" != "$targets" ]; then
        echo "fuzz harness: target list does not match the reviewed manifest" >&2
        exit 1
    fi

    for target in $targets; do
        if [ ! -s "fuzz/fuzz_targets/${target}.rs" ]; then
            echo "fuzz harness: missing target fuzz/fuzz_targets/${target}.rs" >&2
            exit 1
        fi
        if [ ! -d "fuzz/seeds/${target}" ]; then
            echo "fuzz harness: missing seed directory fuzz/seeds/${target}" >&2
            exit 1
        fi
        if ! find "fuzz/seeds/${target}" -type f -size +0c | grep -q .; then
            echo "fuzz harness: ${target} has no nonempty seed" >&2
            exit 1
        fi
    done

    if git ls-files fuzz/artifacts fuzz/corpus | grep -q .; then
        echo "fuzz harness: generated corpus or crash artifacts are tracked" >&2
        exit 1
    fi
}

require_fuzz_tooling() {
    installed="$(cargo fuzz --version 2>/dev/null || true)"
    if [ "$installed" != "cargo-fuzz ${cargo_fuzz_version}" ]; then
        echo "fuzz harness: cargo-fuzz ${cargo_fuzz_version} is required" >&2
        exit 1
    fi
    if ! rustup run "$toolchain" rustc --version >/dev/null 2>&1; then
        echo "fuzz harness: Rust ${toolchain} is required" >&2
        exit 1
    fi
}

check_layout

case "$mode" in
--metadata)
    ;;
--build)
    require_fuzz_tooling
    cargo "+${toolchain}" fuzz build
    ;;
--smoke)
    require_fuzz_tooling
    temporary="$(mktemp -d)"
    trap 'rm -rf "$temporary"' EXIT HUP INT TERM
    for target in $targets; do
        corpus="${temporary}/${target}"
        mkdir "$corpus"
        cp -R "fuzz/seeds/${target}/." "$corpus"
        cargo "+${toolchain}" fuzz run "$target" "$corpus" -- \
            -runs=64 -max_len=16384 -timeout=10
    done
    ;;
*)
    echo "usage: scripts/check_fuzz_harness.sh [--metadata|--build|--smoke]" >&2
    exit 2
    ;;
esac

echo "fuzz harness: ${mode} passed for 7 targets"
