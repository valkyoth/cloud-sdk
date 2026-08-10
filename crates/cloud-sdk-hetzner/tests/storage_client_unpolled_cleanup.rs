//! Regression coverage for Storage cleanup before password-reset futures poll.

use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitState, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintScope, PreparationStorageGuard, ReplayPolicy,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::operations::ResetStorageBoxPassword;
use cloud_sdk_hetzner::association::{
    AssociatedDestructivePermit, AssociatedOperation, AssociatedPlanConfirmation, Sha256PlanHasher,
    build_associated_plan_digest,
};
use cloud_sdk_hetzner::client::HetznerClient;
use cloud_sdk_hetzner::storage::storage_boxes::{
    StorageBoxActionEndpoint, StorageBoxId, StorageBoxPassword, StorageBoxResetPasswordRequest,
};
use cloud_sdk_testkit::{LocalMockTransport, MockTransport};

const PASSWORD: &str = "correct-horse-battery-staple";

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

macro_rules! unpolled_storage_cleanup_test {
    ($name:ident, $transport:ident, $method:ident) => {
        #[test]
        fn $name() {
            let endpoint = official_endpoint();
            let operation = reset_operation();
            let exchanges = [];
            let preparation_client =
                HetznerClient::storage(MockTransport::new(&exchanges).with_endpoint(endpoint))
                    .unwrap_or_else(|_| unreachable!("Storage preparation client failed"));
            let mut target = [0xa5_u8; 128];
            let mut request_body = [0x5a_u8; 256];
            {
                let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
                let prepared = preparation_client
                    .prepare_reset_storage_box_password(&operation, &mut storage)
                    .unwrap_or_else(|_| unreachable!("password reset preparation failed"));
                let mut fingerprint_scratch = [0xa5_u8; 4_096];
                let mut digest_storage = [0xa5_u8; 32];
                let fingerprint = build_associated_plan_digest(
                    plan(prepared, endpoint),
                    &mut fingerprint_scratch,
                    &mut digest_storage,
                    &Sha256PlanHasher,
                )
                .unwrap_or_else(|_| unreachable!("password reset fingerprint failed"));
                assert_eq!(fingerprint_scratch, [0_u8; 4_096]);
                let mut permit = AssociatedDestructivePermit::new(
                    fingerprint.subject(),
                    PermitTimestamp::from_seconds(100),
                )
                .unwrap_or_else(|_| unreachable!("password reset permit failed"));
                let attempt = permit
                    .begin(PermitTimestamp::from_seconds(101))
                    .unwrap_or_else(|_| unreachable!("password reset attempt failed"));
                let client =
                    HetznerClient::storage($transport::new(&exchanges).with_endpoint(endpoint))
                        .unwrap_or_else(|_| unreachable!("Storage execution client failed"));
                let mut body = [0xa5_u8; 1_024];
                let mut headers = [0x5a_u8; 8_192];

                let future = client.$method(attempt, &FixedClock, &mut body, &mut headers);
                drop(future);

                assert!(client.transport().is_complete());
                assert_eq!(body, [0_u8; 1_024]);
                assert_eq!(headers, [0_u8; 8_192]);
                assert_eq!(permit.state(), PermitState::PendingReconciliation);
            }
            assert_eq!(target, [0_u8; 128]);
            assert_eq!(request_body, [0_u8; 256]);
        }
    };
}

unpolled_storage_cleanup_test!(
    unpolled_named_storage_send_async_clears_secret_request_and_response_storage,
    MockTransport,
    reset_storage_box_password_async
);
unpolled_storage_cleanup_test!(
    unpolled_named_storage_local_async_clears_secret_request_and_response_storage,
    LocalMockTransport,
    reset_storage_box_password_local_async
);

fn reset_operation() -> AssociatedOperation<
    ResetStorageBoxPassword,
    StorageBoxActionEndpoint,
    cloud_sdk_hetzner::prepared::NoQuery,
    StorageBoxResetPasswordRequest<'static>,
> {
    let id = StorageBoxId::new(42).unwrap_or_else(|| unreachable!("Storage Box ID failed"));
    let password = StorageBoxPassword::new(PASSWORD)
        .unwrap_or_else(|_| unreachable!("Storage password fixture failed"));
    AssociatedOperation::<ResetStorageBoxPassword, _, _, _>::json(
        StorageBoxActionEndpoint::ResetPassword(id),
        StorageBoxResetPasswordRequest::new(password),
    )
    .unwrap_or_else(|_| unreachable!("password reset association failed"))
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.com", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("official Storage endpoint fixture failed"))
}

fn plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, ResetStorageBoxPassword>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, ResetStorageBoxPassword> {
    let context = PermitContext::new(b"v0.73 Storage unpolled cleanup fixture")
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
