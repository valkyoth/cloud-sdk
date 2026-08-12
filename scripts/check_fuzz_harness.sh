#!/usr/bin/env sh
set -eu

mode="${1:---metadata}"
toolchain="nightly-2026-07-26"
cargo_fuzz_version="0.13.2"
targets="buffer_writers request_targets action_requests labels_dns pagination quota_retry retry_policy pagination_opaque provider_links action_polling response_envelopes response_content_type checked_response cloud_special_responses raw_response_parser raw_http1_wire incremental_json robot_form robot_error_protocol robot_server_response robot_ip_parser robot_cancellation_response robot_ip_response robot_subnet_response robot_reset_response"

check_layout() {
    cargo fmt --manifest-path fuzz/Cargo.toml -- --check
    cargo metadata --manifest-path fuzz/Cargo.toml --locked --no-deps \
        --format-version 1 >/dev/null

    cargo tree --manifest-path fuzz/Cargo.toml --locked --color never \
        --edges normal --prefix none |
        scripts/check-fuzz-aws-lc-tree.py

    manifest_targets="$(
        sed -n 's/^name = "\([a-z0-9_]*\)"$/\1/p' fuzz/Cargo.toml |
            tail -n 25 |
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
cargo test --locked --manifest-path fuzz/Cargo.toml --tests

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
        max_len=16384
        if [ "$target" = raw_http1_wire ]; then
            max_len=66560
        elif [ "$target" = robot_error_protocol ]; then
            max_len=65537
        fi
        cargo "+${toolchain}" fuzz run "$target" "$corpus" -- \
            -runs=64 "-max_len=${max_len}" -timeout=10
    done
    ;;
*)
    echo "usage: scripts/check_fuzz_harness.sh [--metadata|--build|--smoke]" >&2
    exit 2
    ;;
esac

echo "fuzz harness: ${mode} passed for 25 targets"
