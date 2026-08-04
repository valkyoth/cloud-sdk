#!/usr/bin/env sh
set -eu

core=crates/cloud-sdk/src/operation/permit
prepared=crates/cloud-sdk/src/operation/prepared.rs
authenticated=crates/cloud-sdk/src/authentication/transport.rs
provider=crates/cloud-sdk-hetzner/src/association

for required in \
    'pub struct CanonicalPlanFingerprint' \
    'pub struct PlanFingerprintDigest' \
    'pub struct PlanConfirmation' \
    'pub struct SharedPermitState' \
    'pub struct PermitAttempt' \
    'pub trait PermitClock'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "execution permits: missing contract $required" >&2
        exit 1
    fi
done

for permit in MutationPermit DestructivePermit CostPermit; do
    grep -Fq "$permit," "$core/direct.rs"
done
for permit in SharedMutationPermit SharedDestructivePermit SharedCostPermit; do
    grep -Fq "$permit," "$core/shared.rs"
done

for required in \
    'cloud-sdk/plan-confirm/v1\0' \
    'ConstantTimeEq' \
    'DigestRollback' \
    'PlanChange::NoOp' \
    'DeliveryPhase::PossiblySent' \
    'PendingReconciliation' \
    'ClockRollback' \
    'StaleGeneration' \
    'IdempotencyMismatch'; do
    if ! grep -R -Fq "$required" "$core"; then
        echo "execution permits: missing fail-closed contract $required" >&2
        exit 1
    fi
done

grep -Fq 'PreparedExecutionError::AuthorizationRequired' "$prepared"
grep -Fq 'pub(crate) const fn authenticated_request' "$prepared"
grep -Fq 'pub(crate) const fn new' "$authenticated"
if grep -Fq 'pub const fn prepared(' "$core/state.rs"; then
    echo 'execution permits: permit attempt exposes a reusable prepared request' >&2
    exit 1
fi
grep -Fq 'Some(self.subject.endpoint())' "$core/state.rs"
grep -Fq 'elapsed >= u64::from(self.duration)' crates/cloud-sdk/src/operation/permit.rs
if [ "$(grep -Fc 'self.ensure_fresh(clock.now(),' "$core/state.rs")" -ne 3 ]; then
    echo 'execution permits: every execution mode must recheck dispatch time' >&2
    exit 1
fi
grep -Fq 'confirmed_endpoint_is_exact_within_an_admitted_official_set' \
    "$core/tests/dispatch_tests.rs"
grep -Fq 'expiry_is_exclusive_and_rechecked_at_blocking_dispatch' \
    "$core/tests/dispatch_tests.rs"
grep -Fq 'send_async_samples_time_when_first_polled' \
    "$core/tests/dispatch_tests.rs"
grep -Fq 'local_async_samples_time_when_first_polled' \
    "$core/tests/dispatch_tests.rs"
grep -Fq 'shared_attempt_rechecks_expiry_at_dispatch' \
    "$core/tests/dispatch_tests.rs"
grep -Fq 'method: Method::Get' crates/cloud-sdk/src/pagination/link.rs
grep -Fq 'pub trait ReadOnlyOperation' "$provider/policy.rs"
grep -Fq 'O: ReadOnlyOperation' "$provider/prepared.rs"
grep -Fq 'read_only_operation!($marker, $permit)' "$provider/markers.rs"

for contract in \
    'Direct permits are neither `Copy` nor `Clone`' \
    'state is `PossiblySent`' \
    '`reconcile_not_applied`' \
    'The SDK has no clock, random source, price feed' \
    '`expires_at` is exclusive' \
    'Confirming endpoint A never authorizes endpoint B' \
    '`AuthenticatedRequest` construction and prepared-request extraction are'; do
    if ! grep -Fq "$contract" docs/EXECUTION_PERMITS.md; then
        echo "execution permits: missing documented boundary $contract" >&2
        exit 1
    fi
done

cargo test --locked -p cloud-sdk --all-features operation::permit
cargo test --locked -p cloud-sdk --doc --all-features
cargo test --locked -p cloud-sdk-hetzner --doc --all-features
cargo test --locked -p cloud-sdk-testkit --all-features prepared
cargo check --locked -p cloud-sdk --no-default-features
cargo check --locked -p cloud-sdk-hetzner --no-default-features

echo 'Plan-confirm permit identity, lifecycle, provider gating, docs, and no_std checks passed.'
