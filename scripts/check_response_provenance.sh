#!/usr/bin/env sh
set -eu

response_model=crates/cloud-sdk/src/transport/response.rs
attempt_model=crates/cloud-sdk/src/transport/response/attempt.rs
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

for required in \
    'pub struct ResponseAttempt' \
    'pub struct AsyncResponseStaging' \
    'pub struct ResponseCompletion' \
    'pub(crate) fn staging' \
    'pub fn commit_completion'; do
    if ! grep -Fq "$required" "$attempt_model"; then
        echo "response provenance: missing async response contract $required" >&2
        exit 1
    fi
done

for driver in \
    crates/cloud-sdk/src/transport/asynchronous.rs \
    crates/cloud-sdk/src/authentication/transport.rs \
    crates/cloud-sdk/src/transport/raw.rs \
    crates/cloud-sdk/src/transport/raw/local_async.rs; do
    for required in '.begin_attempt()' 'attempt.staging()' '.commit_completion(completion)'; do
        if ! grep -Fq "$required" "$driver"; then
            echo "response provenance: async driver is missing $required in $driver" >&2
            exit 1
        fi
    done
done

for transport in \
    crates/cloud-sdk/src/transport/asynchronous.rs \
    crates/cloud-sdk/src/authentication/transport.rs \
    crates/cloud-sdk/src/transport/raw.rs; do
    if ! grep -Fq "response: AsyncResponseStaging<'writer, 'buffer>" "$transport" \
        || ! grep -Fq 'Result<ResponseCompletion, Self::Error>' "$transport"; then
        echo "response provenance: async transport bypasses staging in $transport" >&2
        exit 1
    fi
done

raw_engine=crates/cloud-sdk-reqwest/src/shared/raw_hyper.rs
for required in 'trait RawResponseSink' 'ResponseCompletion::new('; do
    if ! grep -Fq "$required" "$raw_engine"; then
        echo "response provenance: shared raw engine is missing $required" >&2
        exit 1
    fi
done
if grep -Fq '.commit(' "$raw_engine"; then
    echo "response provenance: async raw engine can commit directly" >&2
    exit 1
fi
blocking_raw=crates/cloud-sdk-reqwest/src/blocking/raw.rs
for required in '.begin_attempt()' '.commit_completion(completion)'; do
    if ! grep -Fq "$required" "$blocking_raw"; then
        echo "response provenance: blocking raw adapter is missing $required" >&2
        exit 1
    fi
done
for adapter in \
    crates/cloud-sdk-reqwest/src/blocking/client.rs \
    crates/cloud-sdk-reqwest/src/asynchronous/client.rs; do
    if ! grep -Fq '.execute_authenticated(' "$adapter"; then
        echo "response provenance: authenticated adapter bypasses shared attempt path" >&2
        exit 1
    fi
done

if ! grep -Fq 'send_driver_rolls_back_staging_when_cancelled' \
    crates/cloud-sdk/src/transport/local_async_tests.rs; then
    echo "response provenance: Send cancellation regression is missing" >&2
    exit 1
fi

mock=crates/cloud-sdk-testkit/src/mock.rs
for required in '.commit_completion(' '.is_committed()' '.begin_attempt()'; do
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
