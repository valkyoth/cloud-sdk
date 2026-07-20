#!/usr/bin/env sh
set -eu

mode="${1:---fetch}"
if [ "$mode" != "--fetch" ] && [ "$mode" != "--local-only" ]; then
    echo "usage: scripts/check_latest_tools.sh [--fetch|--local-only]" >&2
    exit 2
fi

check_output() {
    label="$1"
    expected="$2"
    shift 2
    output="$("$@")"
    if ! printf '%s\n' "$output" | grep -Fq "$expected"; then
        echo "tool freshness: $label does not report pinned version $expected" >&2
        printf '%s\n' "$output" >&2
        exit 1
    fi
}

check_output Rust 'rustc 1.97.1 ' rustc --version
check_output cargo-deny 'cargo-deny 0.20.2' cargo deny --version
check_output cargo-audit 'cargo-audit-audit 0.22.2' cargo audit --version
check_output cargo-sbom 'cargo-sbom 0.10.0' cargo sbom --version
check_output cargo-fuzz 'cargo-fuzz 0.13.2' cargo fuzz --version

if [ "$mode" = "--local-only" ]; then
    echo "Pinned Rust and Cargo tool installations match repository policy."
    exit 0
fi

check_registry() {
    crate="$1"
    expected="$2"
    result="$(cargo search --registry crates-io --limit 1 "$crate")"
    actual="$(printf '%s\n' "$result" | sed -n "s/^${crate} = \"\([^\"]*\)\".*/\1/p")"
    if [ "$actual" != "$expected" ]; then
        echo "tool freshness: crates.io reports $crate ${actual:-unknown}; expected $expected" >&2
        exit 1
    fi
}

check_registry cargo-deny 0.20.2
check_registry cargo-audit 0.22.2
check_registry cargo-sbom 0.10.0
check_registry cargo-fuzz 0.13.2

echo "Pinned Cargo security and fuzz tools are current on crates.io."
