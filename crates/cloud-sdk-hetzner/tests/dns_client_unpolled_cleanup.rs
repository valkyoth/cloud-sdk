//! Regression coverage for DNS response cleanup before mutation futures poll.

use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitState, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintScope, PreparationStorageGuard, ReplayPolicy,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::operations::ChangeZoneTtl;
use cloud_sdk_hetzner::association::{
    AssociatedMutationPermit, AssociatedOperation, AssociatedPlanConfirmation,
    build_associated_canonical_plan,
};
use cloud_sdk_hetzner::client::HetznerClient;
use cloud_sdk_hetzner::cloud::shared::CloudResourceId;
use cloud_sdk_hetzner::dns::zones::{ZoneActionEndpoint, ZoneReference, ZoneTtl, ZoneTtlRequest};
use cloud_sdk_testkit::{LocalMockTransport, MockTransport};

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

macro_rules! unpolled_dns_cleanup_test {
    ($name:ident, $transport:ident, $method:ident) => {
        #[test]
        fn $name() {
            let endpoint = official_endpoint();
            let zone_id =
                CloudResourceId::new(42).unwrap_or_else(|| unreachable!("zone fixture ID failed"));
            let zone = ZoneReference::Id(zone_id);
            let ttl = ZoneTtl::new(300).unwrap_or_else(|_| unreachable!("zone TTL fixture failed"));
            let request = ZoneTtlRequest::new(zone, ttl);
            let operation = AssociatedOperation::<ChangeZoneTtl, _, _, _>::json(
                ZoneActionEndpoint::ChangeTtl(zone),
                request,
            )
            .unwrap_or_else(|_| unreachable!("change-zone-TTL association failed"));
            let exchanges = [];
            let preparation_client =
                HetznerClient::dns(MockTransport::new(&exchanges).with_endpoint(endpoint))
                    .unwrap_or_else(|_| unreachable!("DNS preparation client construction failed"));
            let mut target = [0_u8; 256];
            let mut request_body = [0_u8; 256];
            let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
            let prepared = preparation_client
                .prepare_change_zone_ttl(&operation, &mut storage)
                .unwrap_or_else(|_| unreachable!("change-zone-TTL preparation failed"));
            let plan = plan(prepared, endpoint);
            let mut fingerprint_storage = [0_u8; 4_096];
            let fingerprint = build_associated_canonical_plan(plan, &mut fingerprint_storage)
                .unwrap_or_else(|_| unreachable!("change-zone-TTL fingerprint failed"));
            let mut permit = AssociatedMutationPermit::new(
                fingerprint.subject(),
                PermitTimestamp::from_seconds(100),
            )
            .unwrap_or_else(|_| unreachable!("change-zone-TTL permit failed"));
            let attempt = permit
                .begin(PermitTimestamp::from_seconds(101))
                .unwrap_or_else(|_| unreachable!("change-zone-TTL attempt failed"));
            let transport = $transport::new(&exchanges).with_endpoint(endpoint);
            let client = HetznerClient::dns(transport)
                .unwrap_or_else(|_| unreachable!("DNS execution client construction failed"));
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

unpolled_dns_cleanup_test!(
    unpolled_named_dns_send_async_clears_complete_response_storage,
    MockTransport,
    change_zone_ttl_async
);
unpolled_dns_cleanup_test!(
    unpolled_named_dns_local_async_clears_complete_response_storage,
    LocalMockTransport,
    change_zone_ttl_local_async
);

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("official DNS endpoint fixture failed"))
}

fn plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, ChangeZoneTtl>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, ChangeZoneTtl> {
    let context = PermitContext::new(b"v0.71 DNS unpolled cleanup fixture")
        .unwrap_or_else(|_| unreachable!("permit context failed"));
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    )
    .unwrap_or_else(|_| unreachable!("permit validity failed"));
    let attempts = AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed"));
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
