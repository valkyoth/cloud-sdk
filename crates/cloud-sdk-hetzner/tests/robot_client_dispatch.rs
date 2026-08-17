//! Cancellation and reentrant-dispatch evidence for the Robot client.

use core::cell::Cell;
use core::future::Future;
use core::sync::atomic::{AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, AuthenticatedRequest, BoundCredentialTransport,
    CredentialAttemptError, CredentialAttemptStatus, CredentialBinding,
};
use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorageGuard, ReplayPolicy,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, BoundTransport, DeliveryClassified, DeliveryPhase, EndpointIdentity,
    EndpointIdentityError, EndpointScheme, ResponseCompletion,
};
use cloud_sdk_hetzner::client::{
    RobotClient, RobotClientExecutionError, RobotClientLifecycleError,
    RobotMutationClientExecutionError, RobotMutationPermit, RobotPermitClientExecutionError,
    build_robot_mutation_canonical_plan, prepare_robot_client_mutation,
};
use cloud_sdk_hetzner::robot::{
    RobotServerListRequest, RobotServerName, RobotServerNumber, RobotServerUpdateRequest,
};
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PendingError;

impl DeliveryClassified for PendingError {
    fn delivery_phase(&self) -> DeliveryPhase {
        DeliveryPhase::PossiblySent
    }
}

struct PendingTransport {
    calls: AtomicUsize,
    endpoint: EndpointIdentity<'static>,
    binding: CredentialBinding,
}

impl PendingTransport {
    fn new() -> Self {
        Self {
            calls: AtomicUsize::new(0),
            endpoint: official_endpoint(),
            binding: CredentialBinding::new(
                [0x5a; cloud_sdk::authentication::CREDENTIAL_BINDING_BYTES],
            )
            .unwrap_or_else(|_| unreachable!("credential binding fixture failed")),
        }
    }
}

impl BoundTransport for PendingTransport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.endpoint)
    }
}

impl BoundCredentialTransport for PendingTransport {
    fn credential_binding(&self) -> CredentialBinding {
        self.binding
    }
}

impl AsyncAuthenticatedTransport for PendingTransport {
    type Error = PendingError;

    async fn send_authenticated<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        _request: AuthenticatedRequest<'request, 'policy>,
        _response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.calls.fetch_add(1, Ordering::AcqRel);
        core::future::pending().await
    }
}

#[test]
fn cancelling_after_async_dispatch_rejects_the_client_generation() {
    let client = RobotClient::official(PendingTransport::new())
        .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let request = RobotServerListRequest::new();
    let pool = ClientWorkspacePool::<1>::new()
        .unwrap_or_else(|_| unreachable!("workspace pool fixture failed"));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let mut response_body = [0_u8; 16_384];
    let mut response_headers = [0_u8; 8_192];
    {
        let workspace = ClientWorkspace::new(
            &mut target,
            &mut request_body,
            &mut response_body,
            &mut response_headers,
        );
        let lease = pool
            .try_acquire(workspace)
            .unwrap_or_else(|_| unreachable!("workspace lease fixture failed"));
        let mut future = core::pin::pin!(client.execute_async(&request, lease));
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
        assert_eq!(client.transport().calls.load(Ordering::Acquire), 1);
    }
    assert_eq!(
        client.credential_status().1,
        CredentialAttemptStatus::Rejected
    );
    assert_eq!(pool.active_leases(), 0);
}

struct ReentrantClock<'a, 'fixture> {
    client: &'a RobotClient<MockTransport<'fixture>>,
    saw_busy: Cell<bool>,
}

impl PermitClock for ReentrantClock<'_, '_> {
    fn now(&self) -> PermitTimestamp {
        let request = RobotServerListRequest::new();
        let pool = ClientWorkspacePool::<1>::new()
            .unwrap_or_else(|_| unreachable!("workspace pool fixture failed"));
        let mut target = [0_u8; 128];
        let mut request_body = [0_u8; 128];
        let mut response_body = [0_u8; 16_384];
        let mut response_headers = [0_u8; 8_192];
        let workspace = ClientWorkspace::new(
            &mut target,
            &mut request_body,
            &mut response_body,
            &mut response_headers,
        );
        let lease = pool
            .try_acquire(workspace)
            .unwrap_or_else(|_| unreachable!("workspace lease fixture failed"));
        let result = self.client.execute_blocking(&request, lease);
        self.saw_busy.set(matches!(
            result,
            Err(RobotClientExecutionError::Lifecycle(
                RobotClientLifecycleError::CredentialAttempt(CredentialAttemptError::DispatchBusy)
            ))
        ));
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn permit_clock_cannot_reenter_the_client_before_transport_dispatch() {
    let number = RobotServerNumber::new(321)
        .unwrap_or_else(|_| unreachable!("server number fixture failed"));
    let name = RobotServerName::new("renamed-1")
        .unwrap_or_else(|_| unreachable!("server name fixture failed"));
    let mutation = RobotServerUpdateRequest::rename(number, name);
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 128];
    let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
    let prepared = prepare_robot_client_mutation(&mutation, &mut storage)
        .unwrap_or_else(|_| unreachable!("mutation preparation fixture failed"));
    let wire = prepared.as_untyped().transport_request();
    let expected = ExpectedRequest::new(wire.method(), wire.target())
        .with_body(wire.body())
        .with_headers(wire.headers());
    let context = PermitContext::new(b"v0.94 Robot dispatch fixture")
        .unwrap_or_else(|_| unreachable!("permit context fixture failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity fixture failed"));
    let plan = cloud_sdk_hetzner::client::RobotMutationPlanConfirmation::new(
        prepared,
        official_endpoint(),
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget fixture failed")),
        PlanChange::ChangesState,
        None,
        None,
    );
    let mut fingerprint_storage = [0_u8; 4_096];
    let fingerprint = build_robot_mutation_canonical_plan(plan, &mut fingerprint_storage)
        .unwrap_or_else(|_| unreachable!("mutation fingerprint fixture failed"));
    let mut permit =
        RobotMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100))
            .unwrap_or_else(|_| unreachable!("mutation permit fixture failed"));
    let body = FixtureBody::new(b"")
        .unwrap_or_else(|_| unreachable!("authentication body fixture failed"));
    let unauthorized = cloud_sdk::transport::StatusCode::new(401)
        .unwrap_or_else(|| unreachable!("authentication status fixture failed"));
    let fixture = ResponseFixture::error(unauthorized, body)
        .unwrap_or_else(|_| unreachable!("authentication response fixture failed"));
    let exchanges = [MockExchange::new(expected, fixture)];
    let client =
        RobotClient::official(MockTransport::new(&exchanges).with_endpoint(official_endpoint()))
            .unwrap_or_else(|_| unreachable!("Robot client construction failed"));
    let clock = ReentrantClock {
        client: &client,
        saw_busy: Cell::new(false),
    };
    let client_attempt = client
        .begin_permit_attempt()
        .unwrap_or_else(|_| unreachable!("credential attempt fixture failed"));
    let permit_attempt = permit
        .begin(PermitTimestamp::from_seconds(101))
        .unwrap_or_else(|_| unreachable!("permit attempt fixture failed"));
    let mut response_body = [0_u8; 1_024];
    let mut response_headers = [0_u8; 8_192];
    let result = client_attempt.execute_mutation_blocking(
        permit_attempt,
        &clock,
        &mut response_body,
        &mut response_headers,
    );
    assert!(clock.saw_busy.get());
    assert!(matches!(
        result,
        Err(RobotMutationClientExecutionError::Permit(
            RobotPermitClientExecutionError::AuthenticationRejected(_)
        ))
    ));
    assert!(client.transport().is_complete());
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "robot-ws.your-server.de", 443, "/")
        .unwrap_or_else(|_| unreachable!("official Robot endpoint fixture failed"))
}
