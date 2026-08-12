use alloc::{format, vec};
use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::authentication::{CREDENTIAL_BINDING_BYTES, CredentialBinding};
use cloud_sdk::operation::{
    AttemptBudget, OperationImpact, PermitClock, PermitContext, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintBuildError, PlanFingerprintScope, PreparationStorage,
    PrepareOperation, ReplayPolicy, RequestBodySensitivity, RequestSemantics, RetryEligibility,
    SharedPermitState,
};
use cloud_sdk::transport::{
    EndpointIdentity, HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport, ResponseFixture,
};

use super::*;
use crate::association::Sha256PlanHasher;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::RobotServerNumber;

mod remediation;

const SUMMARY: &str = r#"{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"type":["sw","hw","man"]}"#;
const DETAIL: &[u8] = br#"{"reset":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"type":["sw","hw","man"],"operating_status":"not supported"}}"#;
const ACTION: &[u8] =
    br#"{"reset":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","type":"hw"}}"#;

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

struct TimestampClock(u64);

impl PermitClock for TimestampClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(self.0)
    }
}

#[test]
fn checked_state_rejects_unadvertised_reset_types() {
    let reset = detail();
    assert!(matches!(
        RobotResetExecuteRequest::from_checked(
            &reset,
            RobotResetIntent::Execute(RobotResetType::PowerLong),
        ),
        Err(RobotResetRequestError::UnsupportedCapability)
    ));
    assert!(reset.reset().supports(RobotResetType::Software));
    assert!(reset.reset().operating_status().is_not_supported());
    assert!(!format!("{reset:?}").contains("192.0.2"));
}

#[test]
fn strict_models_reject_duplicates_unknown_types_and_identity_mismatch() {
    let list = format!("[{{\"reset\":{SUMMARY}}}]");
    let decoded =
        decode_list(list.as_bytes()).unwrap_or_else(|_| unreachable!("reset list fixture failed"));
    assert_eq!(decoded.len(), 1);
    let first = decoded
        .as_slice()
        .first()
        .unwrap_or_else(|| unreachable!("reset list fixture became empty"));
    assert_eq!(first.types().len(), 3);

    let duplicate = format!("[{{\"reset\":{SUMMARY}}},{{\"reset\":{SUMMARY}}}]");
    assert_eq!(
        decode_list(duplicate.as_bytes()).err(),
        Some(RobotResetDecodeError::InvalidList)
    );
    let duplicate_type = text(DETAIL).replace("\"sw\",\"hw\"", "\"sw\",\"sw\"");
    assert_eq!(
        decode_get(number(321), duplicate_type.as_bytes()).err(),
        Some(RobotResetDecodeError::InvalidResetTypes)
    );
    let unknown = text(DETAIL).replace("\"hw\"", "\"future\"");
    assert_eq!(
        decode_get(number(321), unknown.as_bytes()).err(),
        Some(RobotResetDecodeError::InvalidResetTypes)
    );
    assert_eq!(
        decode_get(number(322), DETAIL).err(),
        Some(RobotResetDecodeError::ResponseIdentityMismatch)
    );
}

#[test]
fn action_accepts_source_example_but_binds_all_available_identity() {
    let reset = detail();
    let request = RobotResetExecuteRequest::from_checked(
        &reset,
        RobotResetIntent::Execute(RobotResetType::Hardware),
    )
    .unwrap_or_else(|_| unreachable!("advertised reset was rejected"));
    let action = decode_action(&request, ACTION)
        .unwrap_or_else(|_| unreachable!("source example action failed"));
    assert!(action.server_number().is_none());
    assert_eq!(action.reset_type(), RobotResetType::Hardware);

    let table_shape = text(ACTION).replace("\"type\"", "\"server_number\":321,\"type\"");
    assert!(decode_action(&request, table_shape.as_bytes()).is_ok());
    let wrong_type = text(ACTION).replace("\"hw\"", "\"sw\"");
    assert_eq!(
        decode_action(&request, wrong_type.as_bytes()).err(),
        Some(RobotResetDecodeError::MutationOutcomeMismatch)
    );
    let wrong_ip = text(ACTION).replace("192.0.2.10", "192.0.2.11");
    assert_eq!(
        decode_action(&request, wrong_ip.as_bytes()).err(),
        Some(RobotResetDecodeError::ResponseIdentityMismatch)
    );
}

#[test]
fn failed_preparation_clears_complete_caller_storage() {
    let reset = detail();
    let request = RobotResetExecuteRequest::from_checked(
        &reset,
        RobotResetIntent::Execute(RobotResetType::Hardware),
    )
    .unwrap_or_else(|_| unreachable!("advertised reset was rejected"));
    let mut target = [0xa5_u8; 4];
    let mut body = [0x5a_u8; 4];
    assert!(
        request
            .prepare_bound(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0; 4]);
    assert_eq!(body, [0; 4]);
}

#[test]
fn sensitive_reset_requires_digest_and_executes_with_direct_permit() {
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
    let endpoint = endpoint();
    let mut exact = [0xa5_u8; 4_096];
    assert!(matches!(
        build_robot_reset_canonical_plan(plan(prepared, endpoint), &mut exact),
        Err(PlanFingerprintBuildError::SensitiveBodyRequiresDigest)
    ));
    assert_eq!(exact, [0_u8; 4_096]);

    let mut target = [0_u8; 128];
    let mut body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("reset preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_reset_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("reset digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut permit =
        RobotResetDestructivePermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("destructive permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(ACTION))];
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
        .unwrap_or_else(|_| unreachable!("reset execution failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

#[test]
fn shared_local_permit_preserves_action_association() {
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
    let endpoint = endpoint();
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_robot_reset_plan_digest(
        plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("reset digest failed"));
    let mut state = SharedPermitState::new();
    let permit = RobotResetSharedDestructivePermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("shared permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("shared attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(ACTION))];
    let transport = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_local_async(
        &FixedClock,
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("local reset execution failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

fn number(value: u64) -> RobotServerNumber {
    RobotServerNumber::new(value).unwrap_or_else(|_| unreachable!("server number failed"))
}

fn detail() -> AuthorizedRobotReset {
    let reset =
        decode_get(number(321), DETAIL).unwrap_or_else(|_| unreachable!("detail fixture failed"));
    AuthorizedRobotReset::new(
        reset,
        credential_binding(0x5a),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("authorization fixture failed"))
}

fn credential_binding(byte: u8) -> CredentialBinding {
    CredentialBinding::new([byte; CREDENTIAL_BINDING_BYTES])
        .unwrap_or_else(|_| unreachable!("credential binding fixture failed"))
}

#[allow(clippy::too_many_arguments)]
fn assert_prepared<O>(
    operation: O,
    method: Method,
    target: &str,
    body: &[u8],
    operation_id: &str,
    impact: OperationImpact,
    semantics: RequestSemantics,
    retry: RetryEligibility,
    sensitivity: RequestBodySensitivity,
    maximum_response_bytes: usize,
) where
    O: PrepareOperation<Error = RobotResetRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 128];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("request preparation failed"));
    assert_prepared_request(
        prepared,
        method,
        target,
        body,
        operation_id,
        impact,
        semantics,
        retry,
        sensitivity,
        maximum_response_bytes,
    );
}

#[allow(clippy::too_many_arguments)]
fn assert_prepared_request(
    prepared: cloud_sdk::operation::PreparedRequest<'_>,
    method: Method,
    target: &str,
    body: &[u8],
    operation_id: &str,
    impact: OperationImpact,
    semantics: RequestSemantics,
    retry: RetryEligibility,
    sensitivity: RequestBodySensitivity,
    maximum_response_bytes: usize,
) {
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(prepared.transport_request().body(), body);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(prepared.metadata().semantics(), semantics);
    assert_eq!(prepared.metadata().retry_eligibility(), retry);
    assert_eq!(prepared.body_sensitivity(), sensitivity);
    assert_eq!(
        prepared.response_policy().max_body_bytes(),
        maximum_response_bytes
    );
    assert_eq!(
        prepared.raw_response_policy().body_limit(StatusCode::OK),
        maximum_response_bytes
    );
}

fn decode_list(body: &[u8]) -> Result<RobotResetList, RobotResetDecodeError> {
    let request = RobotResetListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_get(number: RobotServerNumber, body: &[u8]) -> Result<RobotReset, RobotResetDecodeError> {
    let request = RobotResetGetRequest::new(number);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn decode_action(
    request: &RobotResetExecuteRequest<'_>,
    body: &[u8],
) -> Result<RobotResetAction, RobotResetDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("action preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn with_json<R, O>(
    prepared: PreparedRobotReset<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotReset<'_, '_, R>) -> O,
) -> O {
    let mut response_storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!("response headers failed"))
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!("content type failed"));
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("response body failed"))
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    let checked = prepared
        .validate_response(response)
        .unwrap_or_else(|_| unreachable!("response policy failed"));
    decode(checked)
}

fn plan<'storage, 'request, 'state>(
    prepared: PreparedRobotReset<'storage, 'request, RobotResetExecuteRequest<'state>>,
    endpoint: EndpointIdentity<'static>,
) -> RobotResetPlanConfirmation<'static, 'storage, 'request, RobotResetExecuteRequest<'state>> {
    plan_until(prepared, endpoint, 125)
}

fn plan_until<'storage, 'request, 'state>(
    prepared: PreparedRobotReset<'storage, 'request, RobotResetExecuteRequest<'state>>,
    endpoint: EndpointIdentity<'static>,
    expires_at: u64,
) -> RobotResetPlanConfirmation<'static, 'storage, 'request, RobotResetExecuteRequest<'state>> {
    RobotResetPlanConfirmation::new(
        prepared,
        endpoint,
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"v0.82 Robot reset fixture")
            .unwrap_or_else(|_| unreachable!("permit context failed")),
        PermitValidity::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(expires_at),
        )
        .unwrap_or_else(|_| unreachable!("permit validity failed")),
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed")),
        PlanChange::ChangesState,
        None,
        None,
    )
}

fn endpoint() -> EndpointIdentity<'static> {
    official_robot_endpoint_identity().unwrap_or_else(|_| unreachable!("endpoint failed"))
}

fn expected_request(prepared: cloud_sdk::operation::PreparedRequest<'_>) -> ExpectedRequest<'_> {
    let request = prepared.transport_request();
    ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers())
}

fn json_fixture(body: &[u8]) -> ResponseFixture<'_> {
    ResponseFixture::success(
        FixtureBody::new(body).unwrap_or_else(|_| unreachable!("fixture body failed")),
    )
    .with_content_type("application/json")
}

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => unreachable!("local mock future unexpectedly pending"),
    }
}

fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!("fixture lost UTF-8"))
}
