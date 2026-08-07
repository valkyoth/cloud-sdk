//! Cross-executor execution evidence for v0.62 read-only vertical slices.

use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{
    AttemptBudget, DestructivePermit, MutationPermit, PermitClock, PermitContext, PermitTimestamp,
    PermitValidity, PlanChange, PlanConfirmation, PlanFingerprintScope, PreparationStorage,
    PreparedRequest, ReplayPolicy, build_canonical_plan,
};
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme, StatusCode};
use cloud_sdk_hetzner::association::operations::{
    DeleteCertificate, GetCertificate, GetZoneZonefile, ListLocations, ListStorageBoxes,
    PoweronServer,
};
use cloud_sdk_hetzner::association::{
    AssociatedOperation, HetznerOperation, Prepared, ReadOnlyOperation,
};
use cloud_sdk_hetzner::cloud::catalog::CatalogListEndpoint;
use cloud_sdk_hetzner::cloud::servers::ServerId;
use cloud_sdk_hetzner::cloud::servers::actions::{ServerActionEndpoint, ServerActionKind};
use cloud_sdk_hetzner::cloud::shared::CloudResourceId;
use cloud_sdk_hetzner::dns::zones::{ZoneEndpoint, ZoneReference};
use cloud_sdk_hetzner::security::certificates::{CertificateEndpoint, CertificateId};
use cloud_sdk_hetzner::storage::storage_boxes::StorageBoxEndpoint;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport, ResponseFixture,
};

const JSON: &[u8] = br#"{"ok":true}"#;
const ACTION: &[u8] = br#"{"action":{"id":42,"command":"poweron","status":"running","progress":0,"started":"2026-01-01T00:00:00Z","finished":null,"resources":[{"id":42,"type":"server"}],"error":null}}"#;

struct FixedClock;

impl PermitClock for FixedClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(102)
    }
}

#[test]
fn source_complete_read_slices_cross_every_executor_and_testkit_path() {
    let cloud = endpoint("api.hetzner.cloud");
    let storage = endpoint("api.hetzner.com");

    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 256];
    let locations =
        AssociatedOperation::<ListLocations, _>::endpoint(CatalogListEndpoint::Locations);
    let Ok(locations) = locations else {
        unreachable!("location association failed")
    };
    let prepared = locations.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("location preparation failed")
    };
    exercise_read_modes(prepared, cloud);

    let certificate_id = CertificateId::new(42);
    let Some(certificate_id) = certificate_id else {
        unreachable!("certificate fixture ID failed")
    };
    let certificate = AssociatedOperation::<GetCertificate, _>::endpoint(CertificateEndpoint::Get(
        certificate_id,
    ));
    let Ok(certificate) = certificate else {
        unreachable!("certificate association failed")
    };
    let prepared =
        certificate.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("certificate preparation failed")
    };
    exercise_read_modes(prepared, cloud);

    let zone_id = CloudResourceId::new(42);
    let Some(zone_id) = zone_id else {
        unreachable!("zone fixture ID failed")
    };
    let zone = AssociatedOperation::<GetZoneZonefile, _>::endpoint(ZoneEndpoint::ExportZoneFile(
        ZoneReference::Id(zone_id),
    ));
    let Ok(zone) = zone else {
        unreachable!("zonefile association failed")
    };
    let prepared = zone.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("zonefile preparation failed")
    };
    exercise_read_modes(prepared, cloud);

    let boxes = AssociatedOperation::<ListStorageBoxes, _>::endpoint(StorageBoxEndpoint::List);
    let Ok(boxes) = boxes else {
        unreachable!("Storage Box association failed")
    };
    let prepared = boxes.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("Storage Box preparation failed")
    };
    exercise_read_modes(prepared, storage);
}

#[test]
fn action_and_no_content_slices_cross_permit_and_executor_paths() {
    let cloud = endpoint("api.hetzner.cloud");
    let server_id = ServerId::new(42);
    let Some(server_id) = server_id else {
        unreachable!("server fixture ID failed")
    };
    let certificate_id = CertificateId::new(42);
    let Some(certificate_id) = certificate_id else {
        unreachable!("certificate fixture ID failed")
    };
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 256];

    let operation = AssociatedOperation::<PoweronServer, _>::endpoint(ServerActionEndpoint::Start(
        server_id,
        ServerActionKind::Poweron,
    ));
    let Ok(operation) = operation else {
        unreachable!("poweron association failed")
    };
    let prepared = operation.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("poweron preparation failed")
    };
    exercise_mutation_modes(prepared.into_untyped(), cloud);

    let operation = AssociatedOperation::<DeleteCertificate, _>::endpoint(
        CertificateEndpoint::Delete(certificate_id),
    );
    let Ok(operation) = operation else {
        unreachable!("delete certificate association failed")
    };
    let prepared = operation.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("delete certificate preparation failed")
    };
    exercise_destructive_modes(prepared.into_untyped(), cloud);
}

fn exercise_read_modes<O>(prepared: Prepared<'_, O>, endpoint: EndpointIdentity<'static>)
where
    O: Copy + HetznerOperation + ReadOnlyOperation,
{
    let request = prepared.as_untyped().transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let fixture = || {
        let body = FixtureBody::new(JSON);
        let Ok(body) = body else {
            unreachable!("response fixture body failed")
        };
        ResponseFixture::success(body).with_content_type("application/json")
    };
    let exchanges = [
        MockExchange::new(expected, fixture()),
        MockExchange::new(expected, fixture()),
    ];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);

    let mut body = [0_u8; 64];
    let mut headers = [0_u8; 8_192];
    let blocking = prepared.execute_blocking(&mock, &mut body, &mut headers);
    let Ok(blocking) = blocking else {
        unreachable!("blocking vertical execution failed")
    };
    assert_eq!(blocking.status(), StatusCode::OK);
    drop(blocking);

    let mut body = [0_u8; 64];
    let mut headers = [0_u8; 8_192];
    let future = prepared.execute_async(&mock, &mut body, &mut headers);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(asynchronous)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("Send-async vertical execution failed")
    };
    assert_eq!(asynchronous.status(), StatusCode::OK);
    drop(asynchronous);
    assert!(mock.is_complete());

    let exchanges = [MockExchange::new(expected, fixture())];
    let mock = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut body = [0_u8; 64];
    let mut headers = [0_u8; 8_192];
    let future = prepared.execute_local_async(&mock, &mut body, &mut headers);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(local)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("local-async vertical execution failed")
    };
    assert_eq!(local.status(), StatusCode::OK);
    drop(local);
    assert!(mock.is_complete());
}

fn endpoint(host: &'static str) -> EndpointIdentity<'static> {
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, host, 443, "/v1");
    let Ok(endpoint) = endpoint else {
        unreachable!("official endpoint fixture failed")
    };
    endpoint
}

fn exercise_mutation_modes(prepared: PreparedRequest<'_>, endpoint: EndpointIdentity<'static>) {
    let mut scratch = [0_u8; 4_096];
    let plan = plan(prepared, endpoint);
    let fingerprint = build_canonical_plan(plan, &mut scratch);
    let Ok(fingerprint) = fingerprint else {
        unreachable!("mutation plan fingerprint failed")
    };
    let permit = MutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100));
    let Ok(mut permit) = permit else {
        unreachable!("mutation permit failed")
    };
    let attempt = permit.begin(PermitTimestamp::from_seconds(101));
    let Ok(attempt) = attempt else {
        unreachable!("mutation attempt failed")
    };
    let exchange = exchange(prepared, StatusCode::CREATED, ACTION, true);
    let exchanges = [exchange];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut body = [0_u8; 512];
    let mut headers = [0_u8; 8_192];
    let result = attempt.execute_blocking(&FixedClock, &mock, &mut body, &mut headers);
    assert!(result.is_ok());
    drop(result);

    execute_mutation_async(prepared, endpoint, false);
    execute_mutation_async(prepared, endpoint, true);
}

fn execute_mutation_async(
    prepared: PreparedRequest<'_>,
    endpoint: EndpointIdentity<'static>,
    local: bool,
) {
    let mut scratch = [0_u8; 4_096];
    let fingerprint = build_canonical_plan(plan(prepared, endpoint), &mut scratch);
    let Ok(fingerprint) = fingerprint else {
        unreachable!("async mutation plan fingerprint failed")
    };
    let permit = MutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100));
    let Ok(mut permit) = permit else {
        unreachable!("async mutation permit failed")
    };
    let attempt = permit.begin(PermitTimestamp::from_seconds(101));
    let Ok(attempt) = attempt else {
        unreachable!("async mutation attempt failed")
    };
    let exchanges = [exchange(prepared, StatusCode::CREATED, ACTION, true)];
    let mut body = [0_u8; 512];
    let mut headers = [0_u8; 8_192];
    let mut context = Context::from_waker(Waker::noop());
    if local {
        let mock = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
        let future = attempt.execute_local_async(&FixedClock, &mock, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    } else {
        let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
        let future = attempt.execute_async(&FixedClock, &mock, &mut body, &mut headers);
        let mut future = core::pin::pin!(future);
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Ready(Ok(_))
        ));
    }
}

fn exercise_destructive_modes(prepared: PreparedRequest<'_>, endpoint: EndpointIdentity<'static>) {
    for mode in 0..3 {
        let mut scratch = [0_u8; 4_096];
        let fingerprint = build_canonical_plan(plan(prepared, endpoint), &mut scratch);
        let Ok(fingerprint) = fingerprint else {
            unreachable!("destructive plan fingerprint failed")
        };
        let permit =
            DestructivePermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100));
        let Ok(mut permit) = permit else {
            unreachable!("destructive permit failed")
        };
        let attempt = permit.begin(PermitTimestamp::from_seconds(101));
        let Ok(attempt) = attempt else {
            unreachable!("destructive attempt failed")
        };
        let exchanges = [exchange(prepared, StatusCode::NO_CONTENT, b"", false)];
        let mut body = [0_u8; 1];
        let mut headers = [0_u8; 8_192];
        let mut context = Context::from_waker(Waker::noop());
        match mode {
            0 => {
                let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
                assert!(
                    attempt
                        .execute_blocking(&FixedClock, &mock, &mut body, &mut headers)
                        .is_ok()
                );
            }
            1 => {
                let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
                let future = attempt.execute_async(&FixedClock, &mock, &mut body, &mut headers);
                let mut future = core::pin::pin!(future);
                assert!(matches!(
                    Future::poll(future.as_mut(), &mut context),
                    Poll::Ready(Ok(_))
                ));
            }
            _ => {
                let mock = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
                let future =
                    attempt.execute_local_async(&FixedClock, &mock, &mut body, &mut headers);
                let mut future = core::pin::pin!(future);
                assert!(matches!(
                    Future::poll(future.as_mut(), &mut context),
                    Poll::Ready(Ok(_))
                ));
            }
        }
    }
}

fn plan<'a>(
    prepared: PreparedRequest<'a>,
    endpoint: EndpointIdentity<'static>,
) -> PlanConfirmation<'static, 'a> {
    let context = PermitContext::new(b"v0.62 vertical fixture");
    let Ok(context) = context else {
        unreachable!("permit context failed")
    };
    let validity = PermitValidity::new(
        PermitTimestamp::from_seconds(100),
        PermitTimestamp::from_seconds(200),
    );
    let Ok(validity) = validity else {
        unreachable!("permit validity failed")
    };
    let attempts = AttemptBudget::new(1);
    let Ok(attempts) = attempts else {
        unreachable!("attempt budget failed")
    };
    PlanConfirmation::new(
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

fn exchange<'a>(
    prepared: PreparedRequest<'a>,
    status: StatusCode,
    body: &'static [u8],
    json: bool,
) -> MockExchange<'a> {
    let request = prepared.transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let fixture_body = FixtureBody::new(body);
    let Ok(fixture_body) = fixture_body else {
        unreachable!("permit response body failed")
    };
    let fixture = ResponseFixture::success_at(status, fixture_body);
    let Ok(mut fixture) = fixture else {
        unreachable!("permit success status failed")
    };
    if json {
        fixture = fixture.with_content_type("application/json");
    }
    MockExchange::new(expected, fixture)
}
