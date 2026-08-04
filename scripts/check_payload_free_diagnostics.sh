#!/usr/bin/env sh
set -eu

diagnostics_dir="crates/cloud-sdk/src/diagnostics"
event_file="$diagnostics_dir/event.rs"
observer_file="$diagnostics_dir/observer.rs"
execution_file="crates/cloud-sdk/src/client/execution.rs"

for required in \
    'pub enum DiagnosticErrorCategory' \
    'pub enum DiagnosticRetryCategory' \
    'pub enum DiagnosticRequestId' \
    'pub struct DiagnosticContext' \
    'pub struct DiagnosticResponse' \
    'pub enum DiagnosticEvent' \
    'pub trait DiagnosticObserver' \
    'pub struct NoopDiagnosticObserver'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/diagnostics.rs "$diagnostics_dir"; then
        echo "payload-free diagnostics: missing contract: $required" >&2
        exit 1
    fi
done

for method in \
    'execute_blocking_observed' \
    'execute_async_observed' \
    'execute_local_async_observed'; do
    if ! grep -Fq "$method" "$execution_file"; then
        echo "payload-free diagnostics: missing client method: $method" >&2
        exit 1
    fi
done

if grep -Eq 'String|Vec<|&str|&\[u8\]|RequestTarget|TransportRequest|Header|Body|Cursor|Message' \
    "$event_file"; then
    echo "payload-free diagnostics: event schema admits a dynamic or payload-bearing type" >&2
    exit 1
fi

if ! grep -Fq 'fn observe(&self, event: DiagnosticEvent)' "$observer_file"; then
    echo "payload-free diagnostics: observer is not a shared event callback" >&2
    exit 1
fi

if grep -R -Eq 'tracing|log::|println!|eprintln!' \
    crates/cloud-sdk/src/diagnostics.rs "$diagnostics_dir" "$execution_file"; then
    echo "payload-free diagnostics: core diagnostics perform implicit logging" >&2
    exit 1
fi

cargo test -p cloud-sdk --all-features diagnostics
cargo test -p cloud-sdk --all-features client::tests
cargo test -p cloud-sdk --test diagnostics

printf '%s\n' 'payload-free diagnostics checks passed.'
