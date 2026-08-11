use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorage, PreparedRequest, ReplayPolicy,
};
use cloud_sdk::transport::EndpointIdentity;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

use super::*;
use crate::endpoint::official_robot_endpoint_identity;
use crate::robot::server::RobotServerNumber;

mod matrix;
mod unpolled;

pub(super) const SERVER_CANCELLED: &[u8] = br#"{"cancellation":{"server_ip":"192.0.2.10","server_ipv6_net":"2001:db8::","server_number":321,"server_name":"server-1","earliest_cancellation_date":"2028-02-29","cancelled":true,"reservation_possible":true,"reserved":true,"cancellation_date":"2028-03-01","cancellation_reason":"migration"}}"#;
pub(super) const IP_CANCELLED: &[u8] = br#"{"cancellation":{"ip":"192.0.2.10","server_number":"321","earliest_cancellation_date":"2028-02-29","cancelled":true,"cancellation-date":"2028-03-01"}}"#;
pub(super) const SUBNET_AVAILABLE: &[u8] = br#"{"cancellation":{"ip":"2001:db8::","mask":"64","server_number":321,"earliest_cancellation_date":"2028-02-29","cancelled":false,"cancellation_date":null}}"#;

pub(super) struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn direct_blocking_server_delete() {
    let request = RobotServerCancellationDeleteRequest::new(number());
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("cancellation preparation failed"));
    let expected = expected_request(prepared.as_untyped());
    let endpoint = endpoint();
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint =
        build_cancellation_canonical_plan(plan(prepared, endpoint), &mut fingerprint_storage)
            .unwrap_or_else(|_| unreachable!("cancellation fingerprint failed"));
    let mut permit = CancellationDestructivePermit::new(
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("cancellation permit failed"));
    let attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("cancellation attempt failed"));
    let empty =
        FixtureBody::new(&[]).unwrap_or_else(|_| unreachable!("empty response fixture failed"));
    let exchanges = [MockExchange::new(expected, ResponseFixture::success(empty))];
    let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut response_body = [];
    let mut response_headers = [0_u8; 1];
    let checked = attempt
        .execute_blocking(
            &FixedClock,
            &transport,
            &mut response_body,
            &mut response_headers,
        )
        .unwrap_or_else(|_| unreachable!("cancellation execution failed"));

    assert!(checked.decode_response().is_ok());
    assert!(transport.is_complete());
}

pub(super) fn plan<'storage, 'request, R>(
    prepared: PreparedCancellation<'storage, 'request, R>,
    endpoint: EndpointIdentity<'static>,
) -> CancellationPlanConfirmation<'static, 'storage, 'request, R> {
    let context = PermitContext::new(b"v0.79 Robot cancellation permit fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed"));
    CancellationPlanConfirmation::new(
        prepared,
        endpoint,
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        None,
        None,
    )
}

pub(super) fn expected_request(prepared: PreparedRequest<'_>) -> ExpectedRequest<'_> {
    let request = prepared.transport_request();
    ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers())
}

pub(super) fn json_fixture(body: &'static [u8]) -> ResponseFixture<'static> {
    let body =
        FixtureBody::new(body).unwrap_or_else(|_| unreachable!("JSON response fixture failed"));
    ResponseFixture::success(body).with_content_type("application/json")
}

pub(super) fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match Future::poll(future.as_mut(), &mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => unreachable!("deterministic mock future remained pending"),
    }
}

pub(super) fn endpoint() -> EndpointIdentity<'static> {
    official_robot_endpoint_identity()
        .unwrap_or_else(|_| unreachable!("official Robot endpoint failed"))
}

pub(super) fn number() -> RobotServerNumber {
    RobotServerNumber::new(321).unwrap_or_else(|_| unreachable!("server number fixture failed"))
}

pub(super) fn ip() -> RobotIpAddress {
    RobotIpAddress::new("192.0.2.10").unwrap_or_else(|_| unreachable!("IP fixture failed"))
}

pub(super) fn subnet() -> RobotSubnetAddress {
    RobotSubnetAddress::new("2001:db8::").unwrap_or_else(|_| unreachable!("subnet fixture failed"))
}

pub(super) fn date() -> RobotCancellationDate {
    RobotCancellationDate::new("2028-03-01").unwrap_or_else(|_| unreachable!("date fixture failed"))
}
