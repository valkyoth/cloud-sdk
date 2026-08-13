use alloc::{format, vec};
use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::authentication::{CREDENTIAL_BINDING_BYTES, CredentialBinding};
use cloud_sdk::operation::{
    AttemptBudget, ExecutionPermitError, OperationImpact, PermitClock, PermitContext,
    PermitTimestamp, PermitValidity, PlanChange, PlanFingerprintBuildError, PlanFingerprintScope,
    PreparationStorage, PrepareOperation, ReplayPolicy, RequestBodySensitivity, RequestSemantics,
    RetryEligibility, SharedPermitState,
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

const RESPONSE: &[u8] =
    br#"{"wol":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321}}"#;

struct FixedClock(u64);
impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(self.0)
    }
}

#[test]
fn prepares_exact_discovery_and_capability_checked_send() {
    let get = RobotWolGetRequest::new(number(321));
    let mut get_target = [0_u8; 64];
    let mut get_body = [0_u8; 1];
    assert_prepared(
        get.prepare(PreparationStorage::new(&mut get_target, &mut get_body))
            .unwrap_or_else(|_| unreachable!("WOL get preparation failed")),
        Method::Get,
        "robot_get_wol",
        OperationImpact::ReadOnly,
        RequestSemantics::Safe,
        RetryEligibility::ExplicitPolicy,
        false,
    );

    let authorized = authorized();
    let send = RobotWolSendRequest::from_checked(&authorized, RobotWolIntent::Send);
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = send
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("WOL send preparation failed"));
    assert!(prepared.inner.authorization_evidence_required());
    assert_prepared(
        prepared.inner,
        Method::Post,
        "robot_send_wol",
        OperationImpact::Mutation,
        RequestSemantics::NonIdempotent,
        RetryEligibility::Never,
        true,
    );
}

#[test]
fn strict_identity_rejects_aliases_unknown_fields_and_family_confusion() {
    let wol = decode(number(321), RESPONSE)
        .unwrap_or_else(|_| unreachable!("source WOL response failed"));
    assert_eq!(wol.server_number(), &number(321));
    assert_eq!(
        wol.with_server_ipv4(u32::from),
        u32::from_be_bytes([192, 0, 2, 10])
    );
    assert!(!format!("{wol:?}").contains("192.0.2"));

    assert_eq!(
        decode(number(322), RESPONSE).err(),
        Some(RobotWolDecodeError::ResponseIdentityMismatch)
    );
    let unknown = text(RESPONSE).replace(
        "\"server_number\":321",
        "\"server_number\":321,\"future\":true",
    );
    assert_eq!(
        decode(number(321), unknown.as_bytes()).err(),
        Some(RobotWolDecodeError::InvalidEnvelope)
    );
    let ipv4_alias = text(RESPONSE).replace("192.0.2.10", "192.000.2.10");
    assert_eq!(
        decode(number(321), ipv4_alias.as_bytes()).err(),
        Some(RobotWolDecodeError::InvalidAddress)
    );
    let family = text(RESPONSE).replace("2001:db8::", "192.0.2.11");
    assert_eq!(
        decode(number(321), family.as_bytes()).err(),
        Some(RobotWolDecodeError::InvalidAddress)
    );
}

#[test]
fn failed_preparation_clears_all_caller_storage() {
    let request = RobotWolGetRequest::new(number(u64::MAX));
    let mut target = [0xa5_u8; 4];
    let mut body = [0x5a_u8; 7];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0; 4]);
    assert_eq!(body, [0; 7]);
}

#[test]
fn authenticated_discovery_mints_short_lived_capability() {
    let request = RobotWolGetRequest::new(number(321));
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("WOL discovery preparation failed"));
    let expected = expected_request(prepared.inner);
    let exchanges = [MockExchange::new(expected, json_fixture(RESPONSE))];
    let transport = MockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(binding(0x6b));
    let mut body = [0_u8; MAX_ROBOT_WOL_RESPONSE_BYTES];
    let mut headers = [0_u8; 128];
    let authorized = prepared
        .execute_authorizing_blocking(&FixedClock(1_000), &transport, &mut body, &mut headers)
        .unwrap_or_else(|_| unreachable!("authenticated WOL discovery failed"));
    assert_eq!(
        authorized.observed_at(),
        PermitTimestamp::from_seconds(1_000)
    );
    assert_eq!(
        authorized.expires_at(),
        PermitTimestamp::from_seconds(1_030)
    );
    assert!(transport.is_complete());
}

#[test]
fn capability_evidence_rejects_wrong_lineage_future_time_and_expiry() {
    let authorized = authorized();
    assert_eq!(
        authorized.validate_at(binding(0x6b), PermitTimestamp::from_seconds(101)),
        Err(ExecutionPermitError::CredentialMismatch)
    );
    assert_eq!(
        authorized.validate_at(binding(0x5a), PermitTimestamp::from_seconds(99)),
        Err(ExecutionPermitError::NotYetValid)
    );
    assert!(
        authorized
            .validate_at(binding(0x5a), PermitTimestamp::from_seconds(129))
            .is_ok()
    );
    assert_eq!(
        authorized.validate_at(binding(0x5a), PermitTimestamp::from_seconds(130)),
        Err(ExecutionPermitError::Expired)
    );
}

#[test]
fn send_requires_evidence_digest_and_single_use_mutation_permit() {
    let authorized = authorized();
    let send = RobotWolSendRequest::from_checked(&authorized, RobotWolIntent::Send);
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = send
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("WOL send preparation failed"));
    let expected = expected_request(prepared.inner);
    let mut exact = [0xa5_u8; 2_048];
    assert!(matches!(
        build_robot_wol_canonical_plan(plan(prepared), &mut exact),
        Err(PlanFingerprintBuildError::AuthorizationEvidenceRequired)
    ));
    assert_eq!(exact, [0_u8; 2_048]);

    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = send
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("second WOL send preparation failed"));
    let mut scratch = [0xa5_u8; 2_048];
    let mut digest = [0x5a_u8; 32];
    let fingerprint =
        build_robot_wol_plan_digest(plan(prepared), &mut scratch, &mut digest, &Sha256PlanHasher)
            .unwrap_or_else(|_| unreachable!("WOL plan digest failed"));
    assert_eq!(scratch, [0_u8; 2_048]);
    let mut permit =
        RobotWolMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("WOL mutation permit failed"));
    let attempt = permit
        .begin_for(fingerprint.subject(), PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("WOL attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(RESPONSE))];
    let transport = MockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(binding(0x5a));
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = attempt
        .execute_blocking(
            &FixedClock(102),
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("WOL execution failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
    assert!(permit.begin(PermitTimestamp::from_seconds(103)).is_err());
}

#[test]
fn shared_local_async_permit_preserves_send_association() {
    let authorized = authorized();
    let send = RobotWolSendRequest::from_checked(&authorized, RobotWolIntent::Send);
    let mut target = [0_u8; 64];
    let mut body = [0_u8; 1];
    let prepared = send
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("WOL send preparation failed"));
    let expected = expected_request(prepared.inner);
    let mut scratch = [0_u8; 2_048];
    let mut digest = [0_u8; 32];
    let fingerprint =
        build_robot_wol_plan_digest(plan(prepared), &mut scratch, &mut digest, &Sha256PlanHasher)
            .unwrap_or_else(|_| unreachable!("WOL shared digest failed"));
    let mut state = SharedPermitState::new();
    let permit = RobotWolSharedMutationPermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("WOL shared permit failed"));
    let attempt = permit
        .begin_for(fingerprint.subject(), PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("WOL shared attempt failed"));
    let exchanges = [MockExchange::new(expected, json_fixture(RESPONSE))];
    let transport = LocalMockTransport::new(&exchanges)
        .with_endpoint(endpoint())
        .with_credential_binding(binding(0x5a));
    let mut response_body = [0_u8; 512];
    let mut response_headers = [0_u8; 128];
    let checked = ready(attempt.execute_local_async(
        &FixedClock(102),
        &transport,
        &mut response_body,
        &mut response_headers,
    ))
    .unwrap_or_else(|_| unreachable!("local WOL execution failed"));
    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

fn assert_prepared(
    prepared: cloud_sdk::operation::PreparedRequest<'_>,
    method: Method,
    operation_id: &str,
    impact: OperationImpact,
    semantics: RequestSemantics,
    retry: RetryEligibility,
    expects_form: bool,
) {
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), "/wol/321");
    assert!(prepared.transport_request().body().is_empty());
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(prepared.metadata().semantics(), semantics);
    assert_eq!(prepared.metadata().retry_eligibility(), retry);
    assert_eq!(prepared.body_sensitivity(), RequestBodySensitivity::Public);
    assert_eq!(
        prepared.response_policy().max_body_bytes(),
        MAX_ROBOT_WOL_RESPONSE_BYTES
    );
    assert_eq!(
        prepared.raw_response_policy().body_limit(StatusCode::OK),
        MAX_ROBOT_WOL_RESPONSE_BYTES
    );
    let headers = prepared.transport_request().headers();
    assert_eq!(headers.as_slice().len(), if expects_form { 2 } else { 1 });
}

fn authorized() -> AuthorizedRobotWol {
    AuthorizedRobotWol::new(
        decode(number(321), RESPONSE).unwrap_or_else(|_| unreachable!("WOL fixture failed")),
        binding(0x5a),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("WOL evidence fixture failed"))
}

fn decode(number: RobotServerNumber, body: &[u8]) -> Result<RobotWol, RobotWolDecodeError> {
    let request = RobotWolGetRequest::new(number);
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("WOL decode preparation failed"));
    with_json(prepared, body, |checked| checked.decode_response())
}

fn with_json<R, O>(
    prepared: PreparedRobotWol<'_, '_, R>,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotWol<'_, '_, R>) -> O,
) -> O {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!());
    attempt
        .headers_mut()
        .unwrap_or_else(|_| unreachable!())
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .unwrap_or_else(|_| unreachable!());
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!())
        .copy_from_slice(body);
    attempt
        .commit(StatusCode::OK, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!());
    drop(attempt);
    decode(
        prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!()),
    )
}

fn plan<'storage, 'request, 'state>(
    prepared: PreparedRobotWol<'storage, 'request, RobotWolSendRequest<'state>>,
) -> RobotWolPlanConfirmation<'static, 'storage, 'request, RobotWolSendRequest<'state>> {
    RobotWolPlanConfirmation::new(
        prepared,
        endpoint(),
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"v0.84 Robot WOL fixture").unwrap_or_else(|_| unreachable!()),
        PermitValidity::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(125),
        )
        .unwrap_or_else(|_| unreachable!()),
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).unwrap_or_else(|_| unreachable!()),
        PlanChange::ChangesState,
        None,
        None,
    )
}

fn number(value: u64) -> RobotServerNumber {
    RobotServerNumber::new(value).unwrap_or_else(|_| unreachable!("server number failed"))
}
fn binding(byte: u8) -> CredentialBinding {
    CredentialBinding::new([byte; CREDENTIAL_BINDING_BYTES]).unwrap_or_else(|_| unreachable!())
}
fn endpoint() -> EndpointIdentity<'static> {
    official_robot_endpoint_identity().unwrap_or_else(|_| unreachable!())
}
fn expected_request(prepared: cloud_sdk::operation::PreparedRequest<'_>) -> ExpectedRequest<'_> {
    let request = prepared.transport_request();
    ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers())
}
fn json_fixture(body: &[u8]) -> ResponseFixture<'_> {
    ResponseFixture::success(FixtureBody::new(body).unwrap_or_else(|_| unreachable!()))
        .with_content_type("application/json")
}
fn text(value: &[u8]) -> &str {
    core::str::from_utf8(value).unwrap_or_else(|_| unreachable!())
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
