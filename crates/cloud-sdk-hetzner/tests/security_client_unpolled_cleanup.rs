//! Regression coverage for Security response cleanup before mutation futures poll.

use cloud_sdk::operation::{
    AttemptBudget, PermitClock, PermitContext, PermitState, PermitTimestamp, PermitValidity,
    PlanChange, PlanFingerprintScope, PreparationStorageGuard, ReplayPolicy,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};
use cloud_sdk_hetzner::association::operations::CreateCertificate;
use cloud_sdk_hetzner::association::{
    AssociatedMutationPermit, AssociatedOperation, AssociatedPlanConfirmation, Sha256PlanHasher,
    build_associated_plan_digest,
};
use cloud_sdk_hetzner::client::HetznerClient;
use cloud_sdk_hetzner::security::certificates::{
    CertificateCreateMode, CertificateCreateRequest, CertificateName, certificate_pem,
    private_key_pem,
};
use cloud_sdk_testkit::{LocalMockTransport, MockTransport};

const CERTIFICATE: &str =
    "-----BEGIN CERTIFICATE-----\nY2xvdWQtc2RrLXRlc3Q=\n-----END CERTIFICATE-----";
const PRIVATE_KEY: &str =
    "-----BEGIN PRIVATE KEY-----\nY2xvdWQtc2RrLXNlY3JldA==\n-----END PRIVATE KEY-----";

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

macro_rules! unpolled_security_cleanup_test {
    ($name:ident, $transport:ident, $method:ident) => {
        #[test]
        fn $name() {
            let endpoint = official_endpoint();
            let operation = create_operation();
            let exchanges = [];
            let preparation_client =
                HetznerClient::security(MockTransport::new(&exchanges).with_endpoint(endpoint))
                    .unwrap_or_else(|_| unreachable!("Security preparation client failed"));
            let mut target = [0xa5_u8; 128];
            let mut request_body = [0x5a_u8; 512];
            {
                let mut storage = PreparationStorageGuard::new(&mut target, &mut request_body);
                let prepared = preparation_client
                    .prepare_create_certificate(&operation, &mut storage)
                    .unwrap_or_else(|_| unreachable!("certificate preparation failed"));
                let mut fingerprint_scratch = [0xa5_u8; 4_096];
                let mut digest_storage = [0xa5_u8; 32];
                let fingerprint = build_associated_plan_digest(
                    plan(prepared, endpoint),
                    &mut fingerprint_scratch,
                    &mut digest_storage,
                    &Sha256PlanHasher,
                )
                .unwrap_or_else(|_| unreachable!("certificate fingerprint failed"));
                assert_eq!(fingerprint_scratch, [0_u8; 4_096]);
                let mut permit = AssociatedMutationPermit::new(
                    fingerprint.subject(),
                    PermitTimestamp::from_seconds(100),
                )
                .unwrap_or_else(|_| unreachable!("certificate permit failed"));
                let attempt = permit
                    .begin(PermitTimestamp::from_seconds(101))
                    .unwrap_or_else(|_| unreachable!("certificate attempt failed"));
                let client =
                    HetznerClient::security($transport::new(&exchanges).with_endpoint(endpoint))
                        .unwrap_or_else(|_| unreachable!("Security execution client failed"));
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
            assert_eq!(request_body, [0_u8; 512]);
        }
    };
}

unpolled_security_cleanup_test!(
    unpolled_named_security_send_async_clears_secret_request_and_response_storage,
    MockTransport,
    create_certificate_async
);
unpolled_security_cleanup_test!(
    unpolled_named_security_local_async_clears_secret_request_and_response_storage,
    LocalMockTransport,
    create_certificate_local_async
);

fn create_operation() -> AssociatedOperation<
    CreateCertificate,
    cloud_sdk_hetzner::security::certificates::CertificateEndpoint,
    cloud_sdk_hetzner::prepared::NoQuery,
    CertificateCreateRequest<'static>,
> {
    let name = CertificateName::new("website")
        .unwrap_or_else(|_| unreachable!("certificate name fixture failed"));
    let certificate = certificate_pem(CERTIFICATE)
        .unwrap_or_else(|_| unreachable!("certificate PEM fixture failed"));
    let private_key =
        private_key_pem(PRIVATE_KEY).unwrap_or_else(|_| unreachable!("private-key fixture failed"));
    let request = CertificateCreateRequest::new(
        name,
        CertificateCreateMode::uploaded(certificate, private_key),
    );
    AssociatedOperation::<CreateCertificate, _, _, _>::json(request.endpoint(), request)
        .unwrap_or_else(|_| unreachable!("create-certificate association failed"))
}

fn official_endpoint() -> EndpointIdentity<'static> {
    EndpointIdentity::new(EndpointScheme::Https, "api.hetzner.cloud", 443, "/v1")
        .unwrap_or_else(|_| unreachable!("official Security endpoint fixture failed"))
}

fn plan<'request>(
    prepared: cloud_sdk_hetzner::association::Prepared<'request, CreateCertificate>,
    endpoint: EndpointIdentity<'static>,
) -> AssociatedPlanConfirmation<'static, 'request, CreateCertificate> {
    let context = PermitContext::new(b"v0.72 Security unpolled cleanup fixture")
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
