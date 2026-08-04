#!/usr/bin/env sh
set -eu

action_dir="crates/cloud-sdk/src/action_polling"
pagination_dir="crates/cloud-sdk/src/pagination"

for required in \
    'pub struct ActionPollLimits' \
    'pub struct ActionPoller' \
    'pub trait PollBackoff' \
    'pub enum PollControl' \
    'pub enum ProgressPolicy' \
    'pub struct ProviderTimeObservation'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/action_polling.rs "$action_dir"; then
        echo "workflow drivers: missing action contract: $required" >&2
        exit 1
    fi
done

for required in \
    'pub trait PageStrategy' \
    'pub struct PagerDriver' \
    'pub enum PagerControl'; do
    if ! grep -R -Fq "$required" crates/cloud-sdk/src/pagination.rs "$pagination_dir"; then
        echo "workflow drivers: missing pager contract: $required" >&2
        exit 1
    fi
done

if grep -R -Eq 'std::|tokio|sleep\(|spawn\(|Instant::now|SystemTime' \
    crates/cloud-sdk/src/action_polling.rs \
    "$action_dir/backoff.rs" "$action_dir/context.rs" \
    "$action_dir/driver.rs" "$action_dir/progress.rs" \
    crates/cloud-sdk/src/pagination/driver.rs; then
    echo "workflow drivers: core acquired a clock, runtime, sleep, or std dependency" >&2
    exit 1
fi

if grep -R -Eq 'PollPolicy|PollDecision|core::time::Duration' \
    crates/cloud-sdk/src/action_polling.rs \
    "$action_dir/backoff.rs" "$action_dir/context.rs" \
    "$action_dir/driver.rs" "$action_dir/progress.rs"; then
    echo "workflow drivers: legacy policy-coupled polling remains" >&2
    exit 1
fi

for test_name in \
    observation_limit_is_unconditional_and_terminal \
    busy_loop_delay_and_cumulative_budgets_fail_closed \
    regressions_require_explicit_bounded_resets \
    provider_wall_clock_rollback_never_extends_monotonic_budgets \
    pager_sequences_requests_responses_and_completion \
    pager_cancellation_and_strategy_failures_are_fail_closed; do
    if ! grep -R -Fq "fn $test_name" "$action_dir" "$pagination_dir/tests"; then
        echo "workflow drivers: missing regression $test_name" >&2
        exit 1
    fi
done

cargo test -p cloud-sdk action_polling --all-features
cargo test -p cloud-sdk pagination::tests::driver --all-features
cargo test -p cloud-sdk --doc --all-features
cargo check -p cloud-sdk --no-default-features

echo "Pager and action workflow driver checks passed."
