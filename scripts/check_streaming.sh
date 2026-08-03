#!/usr/bin/env sh
set -eu

core=crates/cloud-sdk/src/transport/streaming
testkit=crates/cloud-sdk-testkit/src/stream.rs

for required in \
    'pub enum StreamKind' \
    'pub enum StreamFraming' \
    'pub enum StreamSinkMode' \
    'pub struct StreamLimits' \
    'pub struct StreamAttempt' \
    'pub enum StreamReplayability' \
    'pub trait BlockingStreamSource' \
    'pub trait AsyncStreamSource' \
    'pub trait LocalAsyncStreamSource'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "streaming: missing core contract $required" >&2
        exit 1
    fi
done

for required in \
    'pub struct StreamFixtureSource' \
    'pub struct StreamFixtureSink' \
    'MAX_STREAM_FIXTURE_CHUNKS'; do
    if ! grep -Fq "$required" "$testkit"; then
        echo "streaming: missing testkit contract $required" >&2
        exit 1
    fi
done

for contract in \
    'never retry' \
    'caller-owned cancellation' \
    'RollbackRequired' \
    'no read-ahead'; do
    if ! grep -Fq "$contract" docs/STREAMING.md; then
        echo "streaming: missing documented boundary $contract" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features transport::streaming
cargo test --locked -p cloud-sdk-testkit --all-features stream
cargo test --locked -p cloud-sdk --doc --all-features
cargo test --locked -p cloud-sdk-testkit --doc --all-features
cargo check --locked -p cloud-sdk --no-default-features
cargo check --locked -p cloud-sdk-testkit --no-default-features
cargo clippy --locked -p cloud-sdk -p cloud-sdk-testkit \
    --all-targets --all-features -- -D warnings

echo "Streaming contracts, boundaries, fixtures, and doctests passed."
