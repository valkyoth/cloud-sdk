#!/usr/bin/env sh
set -eu

response_model=crates/cloud-sdk/src/transport/response.rs
privacy_fixture=crates/cloud-sdk/tests/ui/transport_response_private.rs

if grep -R -n --include='*.rs' 'TransportResponse::new' \
    crates fuzz/fuzz_targets; then
    echo "response provenance: public response construction remains" >&2
    exit 1
fi

for required in \
    'pub struct ResponseBuffer' \
    'pub struct ResponseWriter' \
    'pub struct TransportResponse' \
    'fn from_commit' \
    "inspect: impl for<'response> FnOnce(TransportResponse" \
    'sanitize_response_storage(self.writer.storage,'; do
    if ! grep -Fq "$required" "$response_model"; then
        echo "response provenance: missing response contract $required" >&2
        exit 1
    fi
done

for transport in \
    crates/cloud-sdk/src/transport.rs \
    crates/cloud-sdk/src/transport/asynchronous.rs; do
    if ! grep -Fq 'response: &' "$transport" \
        || ! grep -Fq 'mut ResponseWriter' "$transport"; then
        echo "response provenance: transport does not receive sealed writer" >&2
        exit 1
    fi
done

for adapter in \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs \
    crates/cloud-sdk-testkit/src/mock.rs; do
    if ! grep -Fq '.commit(' "$adapter"; then
        echo "response provenance: writer commit missing from $adapter" >&2
        exit 1
    fi
    if ! grep -Fq '.is_committed()' "$adapter"; then
        echo "response provenance: precommitted writer check missing from $adapter" >&2
        exit 1
    fi
done

privacy_dir="$(mktemp -d)"
trap 'rm -rf "$privacy_dir"' EXIT HUP INT TERM
mkdir -p "$privacy_dir/src"
cp "$privacy_fixture" "$privacy_dir/src/main.rs"
printf '%s\n' \
    '[package]' \
    'name = "transport-response-privacy-check"' \
    'version = "0.0.0"' \
    'edition = "2024"' \
    'publish = false' \
    '' \
    '[workspace]' \
    '' \
    '[dependencies]' \
    "cloud-sdk = { path = \"$(pwd)/crates/cloud-sdk\" }" \
    >"$privacy_dir/Cargo.toml"

if cargo check --quiet --manifest-path "$privacy_dir/Cargo.toml" \
    >"$privacy_dir/stdout" 2>"$privacy_dir/stderr"; then
    echo "response provenance: external response construction compiled" >&2
    exit 1
fi
if ! grep -Fq 'error[E0451]' "$privacy_dir/stderr" \
    && ! grep -Fq 'due to private fields' "$privacy_dir/stderr"; then
    echo "response provenance: privacy fixture did not fail on private fields" >&2
    cat "$privacy_dir/stderr" >&2
    exit 1
fi
if grep -Fq 'error[E0063]' "$privacy_dir/stderr"; then
    echo "response provenance: privacy fixture is missing response fields" >&2
    cat "$privacy_dir/stderr" >&2
    exit 1
fi

cargo test --locked -p cloud-sdk --all-features --test response_provenance
cargo test --locked -p cloud-sdk --all-features --test response_cleanup
cargo test --locked -p cloud-sdk --doc
cargo test --locked -p cloud-sdk-testkit --all-features
cargo test --locked -p cloud-sdk-reqwest --all-features lifecycle
cargo check --locked --manifest-path fuzz/Cargo.toml --bin checked_response

echo "sealed response provenance checks passed."
