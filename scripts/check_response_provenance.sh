#!/usr/bin/env sh
set -eu

response_model=crates/cloud-sdk/src/transport/response.rs
privacy_fixture=crates/cloud-sdk/tests/ui/transport_response_private.rs
attempt_fixture=crates/cloud-sdk/tests/ui/response_writer_attempt_required.rs

if grep -R -n --include='*.rs' 'TransportResponse::new' \
    crates fuzz/fuzz_targets; then
    echo "response provenance: public response construction remains" >&2
    exit 1
fi

for required in \
    'pub struct ResponseBuffer' \
    'pub struct ResponseWriter' \
    'pub struct ResponseAttempt' \
    'fn bypass_attempt' \
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
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs; do
    if ! grep -Fq '.commit(' "$adapter"; then
        echo "response provenance: writer commit missing from $adapter" >&2
        exit 1
    fi
    if ! grep -Fq '.begin_attempt()' "$adapter"; then
        echo "response provenance: response attempt missing from $adapter" >&2
        exit 1
    fi
done

mock=crates/cloud-sdk-testkit/src/mock.rs
for required in '.commit(' '.is_committed()' '.begin_attempt()'; do
    if ! grep -Fq "$required" "$mock"; then
        echo "response provenance: missing $required from $mock" >&2
        exit 1
    fi
done

for regression in \
    crates/cloud-sdk-reqwest/src/blocking/tests/lifecycle.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/tests/lifecycle.rs; do
    if ! grep -Fq 'precommitted_writer_fails_before_network_access' "$regression"; then
        echo "response provenance: precommitted regression missing from $regression" >&2
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

cp "$attempt_fixture" "$privacy_dir/src/main.rs"
if cargo check --quiet --manifest-path "$privacy_dir/Cargo.toml" \
    >"$privacy_dir/stdout" 2>"$privacy_dir/stderr"; then
    echo "response provenance: direct external writer mutation compiled" >&2
    exit 1
fi
if ! grep -Fq 'method `body_mut` is private' "$privacy_dir/stderr" \
    || ! grep -Fq 'method `headers_mut` is private' "$privacy_dir/stderr" \
    || ! grep -Fq 'method `commit` is private' "$privacy_dir/stderr"; then
    echo "response provenance: writer privacy fixture failed for another reason" >&2
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
