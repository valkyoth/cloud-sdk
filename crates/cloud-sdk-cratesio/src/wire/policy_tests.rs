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
    let prefix = "a";
    let suffix = "/1 (ops@example.org)";
    let valid = format!(
        "{prefix}{}{suffix}",
        "x".repeat(
            MAX_USER_AGENT_BYTES
                .saturating_sub(prefix.len())
                .saturating_sub(suffix.len())
        )
    );
    assert!(IdentifyingUserAgent::new(&valid).is_ok());
    let over = format!("a{valid}");
    assert!(IdentifyingUserAgent::new(&over).is_err());
    let unicode = String::from("a/1 (\u{2603}@example.org)");
    assert!(IdentifyingUserAgent::new(&unicode).is_err());
}

#[test]
fn user_agent_rejects_malformed_email_atoms_and_dns_labels() {
    for contact in [
        "ops@example..org",
        "a:b@example.org",
        ".ops@example.org",
        "ops.@example.org",
        "op..s@example.org",
        "ops@-example.org",
        "ops@example-.org",
        "ops@exam_ple.org",
        "ops@.example.org",
        "ops@example.org.",
        "ops@example",
        "ops@[127.0.0.1]",
        "\"ops\"@example.org",
        "https://example..org/contact",
        "https://-example.org/contact",
        "https://example-.org/contact",
    ] {
        assert!(
            IdentifyingUserAgent::new(&format!("app/1 ({contact})")).is_err(),
            "malformed contact accepted: {contact}"
        );
    }
    for contact in [
        "first.last+ops@example.org",
        "!#$%&'*+-/=?^_`{|}~@example.org",
        "OPS@sub-domain.Example.org",
        "ops@1.2",
        "https://sub-domain.example.org/contact",
    ] {
        assert!(IdentifyingUserAgent::new(&format!("app/1 ({contact})")).is_ok());
    }
}

#[test]
fn user_agent_enforces_email_local_and_contact_label_length_limits() {
    for (length, accepted) in [(63, true), (64, true), (65, false)] {
        let value = format!("app/1 ({}@example.org)", "a".repeat(length));
        assert_eq!(IdentifyingUserAgent::new(&value).is_ok(), accepted);
    }
    for (length, accepted) in [(1, true), (63, true), (64, false)] {
        let label = "a".repeat(length);
        for contact in [
            format!("ops@{label}.org"),
            format!("ops@example.{label}"),
            format!("https://{label}.org/contact"),
            format!("https://example.{label}/contact"),
        ] {
            let value = format!("app/1 ({contact})");
            assert_eq!(IdentifyingUserAgent::new(&value).is_ok(), accepted);
        }
    }
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
    let _serial = super::TEST_GATE_LOCK
        .lock()
        .unwrap_or_else(|_| unreachable!("test gate lock"));
    super::reset_test_gate();
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

#[cfg(feature = "std")]
#[test]
fn official_gate_unwind_closes_later_attempts() {
    let _serial = super::TEST_GATE_LOCK
        .lock()
        .unwrap_or_else(|_| unreachable!("test gate lock"));
    super::reset_test_gate();
    let identity = IdentifyingUserAgent::new("test/1 (tests@example.org)")
        .unwrap_or_else(|_| unreachable!("identity fixture"));
    let gate = super::OfficialApiGate::new(identity);
    let result = test_std::panic::catch_unwind(|| {
        gate.try_call::<(), ()>(|_| unreachable!("intentional adapter unwind fixture"))
    });
    assert!(result.is_err());
    assert!(matches!(
        gate.try_call::<(), ()>(|_| unreachable!("poisoned gate must not dispatch")),
        Err(super::OfficialCallError::Schedule(
            ScheduleError::Unavailable
        ))
    ));
    super::reset_test_gate();
}
