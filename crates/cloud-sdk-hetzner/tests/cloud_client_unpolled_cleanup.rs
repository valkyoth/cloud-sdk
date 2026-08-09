//! Regression coverage for response cleanup before state-changing futures poll.

use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitState, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintScope, PreparationStorageGuard, ReplayPolicy,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::operations::PoweronServer;
use cloud_sdk_hetzner::association::{
    AssociatedMutationPermit, AssociatedOperation, AssociatedPlanConfirmation,
    build_associated_canonical_plan,
};
use cloud_sdk_hetzner::client::HetznerClient;
use cloud_sdk_hetzner::cloud::servers::ServerId;
use cloud_sdk_hetzner::cloud::servers::actions::{ServerActionEndpoint, ServerActionKind};
use cloud_sdk_testkit::{LocalMockTransport, MockTransport};

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

macro_rules! unpolled_named_cleanup_test {
    ($name:ident, $transport:ident, $method:ident) => {
        #[test]
        fn $name() {
            let endpoint = cloud_endpoint();
            let Some(server_id) = ServerId::new(42) else {
                unreachable!("server fixture ID failed");
            };
            let operation = AssociatedOperation::<PoweronServer, _>::endpoint(
                ServerActionEndpoint::Start(server_id, ServerActionKind::Poweron),
            );
            let Ok(operation) = operation else {
                unreachable!("power-on association failed");
            };
            let exchanges = [];
            let preparation_client =
                HetznerClient::cloud(MockTransport::new(&exchanges).with_endpoint(endpoint));
            let Ok(preparation_client) = preparation_client else {
                unreachable!("Cloud preparation client construction failed");
            };
            let mut target = [0_u8; 256];
            let mut request_body = [0_u8; 256];
            let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
            let prepared = preparation_client.prepare_poweron_server(&operation, &mut storage);
            let Ok(prepared) = prepared else {
                unreachable!("power-on preparation failed");
            };
            let plan = plan(prepared, endpoint);
            let mut fingerprint_storage = [0_u8; 4_096];
            let fingerprint = build_associated_canonical_plan(plan, &mut fingerprint_storage);
            let Ok(fingerprint) = fingerprint else {
                unreachable!("power-on fingerprint failed");
            };
            let permit = AssociatedMutationPermit::new(
                fingerprint.subject(),
                PermitTimestamp::from_seconds(100),
            );
            let Ok(mut permit) = permit else {
                unreachable!("power-on permit failed");
            };
            let attempt = match permit.begin(PermitTimestamp::from_seconds(101)) {
                Ok(attempt) => attempt,
                Err(_) => unreachable!("power-on attempt failed"),
            };
            let transport = $transport::new(&exchanges).with_endpoint(endpoint);
            let client = HetznerClient::cloud(transport);
            let Ok(client) = client else {
                unreachable!("Cloud execution client construction failed");
            };
            let mut body = [0xa5_u8; 512];
            let mut headers = [0x5a_u8; 8_192];

            let future = client.$method(attempt, &FixedClock, &mut body, &mut headers);
            drop(future);

            assert!(client.transport().is_complete());
            assert_eq!(body, [0_u8; 512]);
            assert_eq!(headers, [0_u8; 8_192]);
            assert_eq!(permit.state(), PermitState::PendingReconciliation);
        }
    };
}

unpolled_named_cleanup_test!(
    unpolled_named_send_async_clears_complete_response_storage,
    MockTransport,
    poweron_server_async
);
unpolled_named_cleanup_test!(
    unpolled_named_local_async_clears_complete_response_storage,
    LocalMockTransport,
    poweron_server_local_async
);

fn cloud_endpoint() -> EndpointIdentity<'static> {
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1");
    let Ok(endpoint) = endpoint else {
        unreachable!("official endpoint fixture failed");
    };
    endpoint
}

fn plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, PoweronServer>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, PoweronServer> {
    let context = PermitContext::new(b"v0.70 unpolled cleanup fixture");
    let Ok(context) = context else {
        unreachable!("permit context failed");
    };
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    );
    let Ok(validity) = validity else {
        unreachable!("permit validity failed");
    };
    let attempts = AttemptBudget::new(1);
    let Ok(attempts) = attempts else {
        unreachable!("attempt budget failed");
    };
    AssociatedPlanConfirmation::new(
        prepared,
        endpoint,
        PlanFingerprintScope::Value(b"account"),
        PlanFingerprintScope::Value(b"project"),
        context,
        validity,
        ReplayPolicy::SingleAttempt,
        attempts,
        PlanChange::ChangesState,
        None,
        None,
    )
}
