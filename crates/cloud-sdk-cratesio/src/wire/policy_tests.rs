use alloc::{format, string::String};
use core::time::Duration;

#[cfg(feature = "std")]
use crate::std as test_std;

use super::{
    API_REQUEST_INTERVAL, ApiSchedule, IdentifyingUserAgent, MAX_USER_AGENT_BYTES, ScheduleError,
};

#[test]
fn user_agent_requires_explicit_bounded_single_line_identity() {
    for valid in [
        "release-bot/1.0 (admin@example.org)",
        "inventory/2 (https://example.org/contact)",
    ] {
        let value = IdentifyingUserAgent::new(valid)
            .unwrap_or_else(|_| unreachable!("valid user agent fixture"));
        assert_eq!(value.as_str(), valid);
        assert!(!format!("{value:?}").contains("example.org"));
    }
    for invalid in [
        "",
        "cloud-sdk",
        "app/1",
        "app/1 ()",
        "app/1 (a)",
        "app/1 (a@)",
        "app/1 (@example.org)",
        "app/1 (https://)",
        "app/1 (https://user@example.org)",
        "app/1 (https://example.org/?token=secret)",
        "app/1 (a@example.org)\r\nX: bad",
        "app/1 (a@example.org)\n",
        "app/1 (a@example.org)\0",
        "app/1 (x(y)@example.org)",
        "app/1 (http://example.org)",
        "app/1 (a@b@example.org)",
        "app/1 (a@example.org) extra",
    ] {
        assert!(IdentifyingUserAgent::new(invalid).is_err());
    }
    let prefix = "a/1 (";
    let suffix = "@example.org)";
    let valid = format!(
        "{prefix}{}{suffix}",
        "x".repeat(
            MAX_USER_AGENT_BYTES
                .saturating_sub(prefix.len())
                .saturating_sub(suffix.len())
        )
    );
    assert!(IdentifyingUserAgent::new(&valid).is_ok());
    let over = valid.replacen("a/1", "aa/1", 1);
    assert!(IdentifyingUserAgent::new(&over).is_err());
    let unicode = String::from("a/1 (\u{2603}@example.org)");
    assert!(IdentifyingUserAgent::new(&unicode).is_err());
}

#[test]
fn scheduling_enforces_exact_intervals_without_burst_reservations() {
    assert_eq!(API_REQUEST_INTERVAL, Duration::from_secs(1));
    let mut rate = ApiSchedule::new();
    assert_eq!(rate.try_start(Duration::ZERO), Ok(()));
    assert_eq!(
        rate.try_start(Duration::ZERO),
        Err(ScheduleError::Wait(Duration::from_secs(1)))
    );
    assert_eq!(
        rate.try_start(Duration::from_millis(999)),
        Err(ScheduleError::Wait(Duration::from_millis(1)))
    );
    assert_eq!(rate.try_start(Duration::from_secs(1)), Ok(()));
    assert_eq!(
        rate.try_start(Duration::ZERO),
        Err(ScheduleError::ClockRollback)
    );
    assert_eq!(rate.try_start(Duration::from_secs(30)), Ok(()));
    assert_eq!(
        rate.try_start(Duration::from_secs(30)),
        Err(ScheduleError::Wait(Duration::from_secs(1)))
    );
    rate.defer_until(Duration::from_secs(90));
    rate.defer_until(Duration::from_secs(2));
    assert_eq!(
        rate.try_start(Duration::from_secs(31)),
        Err(ScheduleError::Wait(Duration::from_secs(59)))
    );
    assert_eq!(rate.try_start(Duration::from_secs(90)), Ok(()));
}

#[test]
fn scheduling_overflow_never_admits_an_attempt() {
    let mut schedule = ApiSchedule::new();
    assert_eq!(
        schedule.try_start(Duration::MAX),
        Err(ScheduleError::Overflow)
    );
    assert_eq!(
        schedule.try_start(Duration::MAX),
        Err(ScheduleError::Overflow)
    );
    assert_eq!(
        schedule.try_start(Duration::ZERO),
        Err(ScheduleError::ClockRollback)
    );
}

#[cfg(feature = "std")]
#[test]
fn concurrent_workers_share_one_non_bursting_schedule() {
    use test_std::sync::{Barrier, Mutex};
    let schedule = Mutex::new(ApiSchedule::new());
    let barrier = Barrier::new(16);
    test_std::thread::scope(|scope| {
        let handles: alloc::vec::Vec<_> = (0..16)
            .map(|_| {
                scope.spawn(|| {
                    barrier.wait();
                    schedule
                        .lock()
                        .unwrap_or_else(|_| unreachable!("test scheduler poisoned"))
                        .try_start(Duration::ZERO)
                })
            })
            .collect();
        let mut admitted = 0_usize;
        for handle in handles {
            match handle
                .join()
                .unwrap_or_else(|_| unreachable!("scheduler worker failed"))
            {
                Ok(()) => admitted = admitted.saturating_add(1),
                Err(error) => assert_eq!(error, ScheduleError::Wait(Duration::from_secs(1))),
            }
        }
        assert_eq!(admitted, 1);
    });
}

#[cfg(feature = "std")]
#[test]
fn official_gate_shares_state_passes_identity_and_hides_transport_errors() {
    use super::{OfficialApiGate, OfficialCallError};
    use core::error::Error;
    let ua = IdentifyingUserAgent::new("test/1 (tests@example.org)")
        .unwrap_or_else(|_| unreachable!("user agent fixture"));
    let first = OfficialApiGate::new(ua);
    let second = OfficialApiGate::new(ua);
    let result = first.try_call::<(), _>(|identity| {
        assert_eq!(identity.as_str(), "test/1 (tests@example.org)");
        assert!(matches!(
            second.try_call::<(), ()>(|_| unreachable!("gate admitted a nested call")),
            Err(OfficialCallError::Schedule(ScheduleError::Unavailable))
        ));
        Err("payload must not leak")
    });
    let error = match result {
        Err(error) => error,
        Ok(()) => unreachable!("transport failure must propagate"),
    };
    assert!(!format!("{error:?} {error}").contains("payload must not leak"));
    assert!(error.source().is_none());
    assert!(matches!(
        second.try_call::<(), ()>(|_| unreachable!("gate bypassed its quiet period")),
        Err(OfficialCallError::Schedule(ScheduleError::Wait(_)))
    ));
}
