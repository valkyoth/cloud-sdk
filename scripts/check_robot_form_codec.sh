#!/usr/bin/env sh
set -eu

test -s crates/cloud-sdk-hetzner/src/robot/form.rs
test -s crates/cloud-sdk-hetzner/src/robot/form/tests.rs
test -s fuzz/fuzz_targets/robot_form.rs
test -s fuzz/seeds/robot_form/repeated-and-controls.txt
test -s docs/ROBOT_WIRE_SOURCE_LOCK.md

grep -Fq 'server%5B%5D=123.123.123.123&server%5B%5D=123.123.123.124' \
    crates/cloud-sdk-hetzner/src/robot/form/tests.rs
grep -Fq 'application/x-www-form-urlencoded' \
    crates/cloud-sdk-hetzner/src/robot/form.rs
grep -Fq 'complete destination is volatile-cleared' \
    docs/ROBOT_WIRE_SOURCE_LOCK.md

cargo test --locked -p cloud-sdk-hetzner --all-features robot::form::tests

echo "Robot form codec source lock, bounds, cleanup, and tests passed."
