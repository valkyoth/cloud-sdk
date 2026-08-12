use super::*;

#[test]
fn prepares_exact_source_locked_requests_and_policies() {
    assert_prepared(
        RobotResetListRequest::new(),
        Method::Get,
        "/reset",
        b"",
        "robot_list_resets",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        MAX_ROBOT_RESET_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotResetGetRequest::new(number(321)),
        Method::Get,
        "/reset/321",
        b"",
        "robot_get_reset",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        RequestBodySensitivity::Public,
        MAX_ROBOT_RESET_DETAIL_RESPONSE_BYTES,
    );
    let checked = detail();
    let execute = RobotResetExecuteRequest::from_checked(
        &checked,
        RobotResetIntent::Execute(RobotResetType::Hardware),
    )
    .unwrap_or_else(|_| unreachable!("advertised reset was rejected"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = execute
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("reset preparation failed"));
    assert!(prepared.inner.authorization_evidence_required());
    assert_prepared_request(
        prepared.inner,
        Method::Post,
        "/reset/321",
        b"type=hw",
        "robot_execute_reset",
        OperationImpact::Destructive,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_RESET_ACTION_RESPONSE_BYTES,
    );
}

#[test]
fn only_authenticated_detail_execution_mints_short_lived_authority() {
    let request = RobotResetGetRequest::new(number(321));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("detail preparation failed"));
    let expected = expected_request(prepared.inner);
    let exchanges = [MockExchange::new(expected, json_fixture(DETAIL))];
    let transport = MockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(credential_binding(0x6b));
    let mut response_body = [0_u8; MAX_ROBOT_RESET_DETAIL_RESPONSE_BYTES];
    let mut response_headers = [0_u8; 128];
    let authorized = prepared
        .execute_authorizing_blocking(
            &TimestampClock(1_000),
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("authenticated preflight failed"));
    assert_eq!(
        authorized.observed_at(),
        PermitTimestamp::from_seconds(1_000)
    );
    assert_eq!(
        authorized.expires_at(),
        PermitTimestamp::from_seconds(1_030)
    );
    assert!(authorized.reset().supports(RobotResetType::Hardware));
    assert!(transport.is_complete());
}

#[test]
fn async_and_local_preflights_preserve_authenticated_lineage() {
    let request = RobotResetGetRequest::new(number(321));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("async detail preparation failed"));
    let expected = expected_request(prepared.inner);
    let exchanges = [MockExchange::new(expected, json_fixture(DETAIL))];
    let transport = MockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(credential_binding(0x6b));
    let mut response_body = [0_u8; MAX_ROBOT_RESET_DETAIL_RESPONSE_BYTES];
    let mut response_headers = [0_u8; 128];
    let authorized = ready(prepared.execute_authorizing_async(
        &TimestampClock(2_000),
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("async preflight failed"));
    assert_eq!(
        authorized.expires_at(),
        PermitTimestamp::from_seconds(2_030)
    );
    assert!(transport.is_complete());

    let request = RobotResetGetRequest::new(number(321));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("local detail preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let exchanges = [MockExchange::new(expected, json_fixture(DETAIL))];
    let transport = LocalMockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(credential_binding(0x7c));
    let mut response_body = [0_u8; MAX_ROBOT_RESET_DETAIL_RESPONSE_BYTES];
    let mut response_headers = [0_u8; 128];
    let authorized = ready(prepared.execute_authorizing_local_async(
        &TimestampClock(3_000),
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("local preflight failed"));
    assert_eq!(
        authorized.expires_at(),
        PermitTimestamp::from_seconds(3_030)
    );
    assert!(transport.is_complete());
}

#[test]
fn plan_validity_cannot_outlive_authenticated_preflight() {
    let reset = detail();
    let request = RobotResetExecuteRequest::from_checked(
        &reset,
        RobotResetIntent::Execute(RobotResetType::Hardware),
    )
    .unwrap_or_else(|_| unreachable!("advertised reset was rejected"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("reset preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    assert!(matches!(
        build_robot_reset_plan_digest(
            plan_until(prepared, endpoint(), 131),
            &mut scratch,
            &mut digest,
            &Sha256PlanHasher,
        ),
        Err(PlanFingerprintBuildError::AuthorizationEvidenceValidityMismatch)
    ));
    assert_eq!(scratch, [0; 4_096]);
    assert_eq!(digest, [0; 32]);
}

#[test]
fn dispatch_rejects_foreign_credentials_and_expired_preflight_before_network() {
    assert_dispatch_authorization_rejected(
        credential_binding(0x6b),
        TimestampClock(102),
        cloud_sdk::operation::ExecutionPermitError::CredentialMismatch,
    );
    assert_dispatch_authorization_rejected(
        credential_binding(0x5a),
        TimestampClock(130),
        cloud_sdk::operation::ExecutionPermitError::Expired,
    );
}

#[test]
fn reset_list_item_limit_is_exact() {
    for (count, expected_ok) in [
        (MAX_ROBOT_RESET_LIST_ITEMS - 1, true),
        (MAX_ROBOT_RESET_LIST_ITEMS, true),
        (MAX_ROBOT_RESET_LIST_ITEMS + 1, false),
    ] {
        let body = reset_list_fixture(count);
        assert_eq!(decode_list(body.as_bytes()).is_ok(), expected_ok, "{count}");
    }
}

fn assert_dispatch_authorization_rejected(
    transport_binding: CredentialBinding,
    clock: TimestampClock,
    expected_error: cloud_sdk::operation::ExecutionPermitError,
) {
    let reset = detail();
    let request = RobotResetExecuteRequest::from_checked(
        &reset,
        RobotResetIntent::Execute(RobotResetType::Hardware),
    )
    .unwrap_or_else(|_| unreachable!("advertised reset was rejected"));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("reset preparation failed"));
    let expected = expected_request(prepared.inner);
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_reset_plan_digest(
        plan(prepared, endpoint()),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("reset digest failed"));
    let mut permit =
        RobotResetDestructivePermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("destructive permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(ACTION))];
    let transport = MockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(transport_binding);
    let mut response_body = [0xa5_u8; 512];
    let mut response_headers = [0x5a_u8; 128];
    let error = attempt
        .execute_blocking(
            &clock,
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .err()
        .unwrap_or_else(|| unreachable!("invalid authorization reached transport"));
    assert!(matches!(
        error.execution(),
        cloud_sdk::operation::PreparedExecutionError::AuthorizationInvalid(actual)
            if *actual == expected_error
    ));
    assert_eq!(response_body, [0; 512]);
    assert_eq!(response_headers, [0; 128]);
    assert_eq!(transport.remaining(), 1);
}

fn reset_list_fixture(count: usize) -> alloc::string::String {
    use core::fmt::Write;

    let mut body = alloc::string::String::new();
    body.try_reserve(count.saturating_mul(160))
        .unwrap_or_else(|_| unreachable!("boundary fixture allocation failed"));
    body.push('[');
    for index in 0..count {
        if index != 0 {
            body.push(',');
        }
        let second = index / 254;
        let fourth = (index % 254)
            .checked_add(1)
            .unwrap_or_else(|| unreachable!("fixture IPv4 component overflowed"));
        let identity = index
            .checked_add(1)
            .unwrap_or_else(|| unreachable!("fixture identity overflowed"));
        write!(
            body,
            "{{\"reset\":{{\"server_ip\":\"198.51.{second}.{fourth}\",\"server_ipv6_net\":\"2001:db8::{:x}\",\"server_number\":{},\"type\":[\"sw\"]}}}}",
            identity,
            identity
        )
        .unwrap_or_else(|_| unreachable!("boundary fixture formatting failed"));
    }
    body.push(']');
    body
}
