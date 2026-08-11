use cloud_sdk::operation::{
    PermitTimestamp, PlanFingerprintBuildError, PreparationStorage, SharedPermitState,
};
use cloud_sdk_testkit::{LocalMockTransport, MockExchange, MockTransport};

use super::*;
use crate::association::Sha256PlanHasher;

#[test]
fn sensitive_post_rejects_exact_fingerprint_construction() {
    let request =
        RobotIpCancellationCreateRequest::new(ip(), RobotCancellationSchedule::On(date()));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP cancellation preparation failed"));
    let mut exact = [0xa5_u8; 4_096];

    assert!(matches!(
        build_cancellation_canonical_plan(plan(prepared, endpoint()), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);
}

#[test]
fn direct_async_server_create_digest() {
    let reason = RobotCancellationReason::new("migration")
        .unwrap_or_else(|_| unreachable!("reason fixture failed"));
    let request = RobotServerCancellationCreateRequest::new(
        number(),
        RobotCancellationSchedule::On(date()),
        Some(reason),
        RobotLocationReservationIntent::Reserve,
    );
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("server cancellation preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_cancellation_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("server cancellation digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut permit = CancellationDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("server cancellation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("server cancellation attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(SERVER_CANCELLED))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("server cancellation async execution failed"));

    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn direct_local_async_ip_create_digest() {
    let request =
        RobotIpCancellationCreateRequest::new(ip(), RobotCancellationSchedule::On(date()));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP cancellation preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_cancellation_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("IP cancellation digest failed"));
    let mut permit = CancellationDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("IP cancellation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("IP cancellation attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(IP_CANCELLED))];
    let transport = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_local_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("IP cancellation local execution failed"));

    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn shared_blocking_subnet_delete() {
    let request = RobotSubnetCancellationDeleteRequest::new(subnet());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("subnet revocation preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint =
        build_cancellation_canonical_plan(plan(prepared, endpoint), &mut fingerprint_storage)
            .unwrap_or_else(|_| unreachable!("subnet revocation fingerprint failed"));
    let mut state = SharedPermitState::new();
    let permit = CancellationSharedDestructivePermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("shared subnet permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("shared subnet attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(SUBNET_AVAILABLE))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = attempt
        .execute_blocking(
            &FixedClock,
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("shared subnet execution failed"));
    let cancellation = checked
        .decode_response()
        .unwrap_or_else(|_| unreachable!("subnet revocation decode failed"));

    assert!(!cancellation.is_cancelled());
    assert!(transport.is_complete());
}

#[test]
fn shared_async_server_create_digest() {
    let reason = RobotCancellationReason::new("migration")
        .unwrap_or_else(|_| unreachable!("reason fixture failed"));
    let request = RobotServerCancellationCreateRequest::new(
        number(),
        RobotCancellationSchedule::On(date()),
        Some(reason),
        RobotLocationReservationIntent::Reserve,
    );
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 256];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("server cancellation preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_cancellation_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("shared server digest failed"));
    let mut state = SharedPermitState::new();
    let permit = CancellationSharedDestructivePermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("shared server permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("shared server attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(SERVER_CANCELLED))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("shared server async execution failed"));

    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn mismatched_acknowledgement_is_rejected_after_permit_execution() {
    let request = RobotIpCancellationCreateRequest::new(
        ip(),
        RobotCancellationSchedule::On(
            RobotCancellationDate::new("2028-03-02")
                .unwrap_or_else(|_| unreachable!("mismatch date fixture failed")),
        ),
    );
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("IP cancellation preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_cancellation_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("IP cancellation digest failed"));
    let mut permit = CancellationDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("IP cancellation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("IP cancellation attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(IP_CANCELLED))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = attempt
        .execute_blocking(
            &FixedClock,
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("IP cancellation execution failed"));

    assert!(matches!(
        checked.decode_response(),
        Err(RobotCancellationDecodeError::MutationOutcomeMismatch)
    ));
    assert!(transport.is_complete());
}
