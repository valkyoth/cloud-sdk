#!/usr/bin/env sh
set -eu

protocol='crates/cloud-sdk-hetzner/src/robot/protocol.rs'
decoder='crates/cloud-sdk-hetzner/src/robot/protocol/decode.rs'
tests='crates/cloud-sdk-hetzner/src/robot/protocol/tests.rs'

for file in "$protocol" "$decoder" "$tests" \
    fuzz/fuzz_targets/robot_error_protocol.rs \
    fuzz/seeds/robot_error_protocol/quota.json; do
    test -s "$file" || {
        echo "Robot protocol: missing evidence $file" >&2
        exit 1
    }
done

for contract in \
    'pub enum RobotFailure' \
    'pub enum RobotRetryDisposition' \
    'pub struct RobotInvalidInput' \
    'pub struct RobotQuota' \
    'pub fn decode_robot_failure('; do
    grep -Fq "$contract" "$protocol" "$decoder" || {
        echo "Robot protocol: missing public contract $contract" >&2
        exit 1
    }
done

grep -Fq 'Self::AuthenticationRejected | Self::InvalidInput(_) | Self::Provider(_)' "$protocol"
grep -Fq 'RobotRetryDisposition::Never' "$protocol"
grep -Fq 'if !matches!(status.get(), 400 | 403 | 404)' "$decoder"
grep -Fq 'RobotDecodeError::UnknownCode' "$decoder"
grep -Fq 'provider bytes created a transport classification' \
    fuzz/fuzz_targets/robot_error_protocol.rs

cargo check --locked -p cloud-sdk-hetzner --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features --features alloc
cargo check --locked -p cloud-sdk-hetzner --no-default-features --features serde
cargo test --locked -p cloud-sdk-hetzner --no-default-features --features serde robot::protocol
scripts/check_robot_wire_fixture.py

echo "Robot error, quota, maintenance, authentication, and transport classifications passed."
