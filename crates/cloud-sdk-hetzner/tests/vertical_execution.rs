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
    AssociatedOperation, AuthenticationClass, HetznerOperation, Prepared, ReadOnlyOperation,
};
use cloud_sdk_hetzner::cloud::catalog::CatalogListEndpoint;
use cloud_sdk_hetzner::cloud::servers::ServerId;
use cloud_sdk_hetzner::cloud::servers::actions::{ServerActionEndpoint, ServerActionKind};
use cloud_sdk_hetzner::cloud::shared::CloudResourceId;
use cloud_sdk_hetzner::dns::zones::{ZoneEndpoint, ZoneReference};
use cloud_sdk_hetzner::security::certificates::{CertificateEndpoint, CertificateId};
use cloud_sdk_hetzner::serde::{HetznerSuccess, decode_associated_checked_response};
use cloud_sdk_hetzner::storage::storage_boxes::StorageBoxEndpoint;
use cloud_sdk_testkit::{
    ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, MockTransport, ResponseFixture,
};

const INVALID_JSON: &[u8] = br#"{"ok":true}"#;
const LOCATIONS: &[u8] = br#"{"locations":[{"id":42,"name":"fsn1","description":"Falkenstein DC Park 1","country":"DE","city":"Falkenstein","latitude":50.47612,"longitude":12.370071,"network_zone":"eu-central"}],"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#;
const CERTIFICATE: &[u8] = br#"{"certificate":{"id":897,"name":"website","labels":{"environment":"prod"},"type":"managed","certificate":"-----BEGIN CERTIFICATE-----\nfixture\n-----END CERTIFICATE-----","created":"2026-01-01T00:00:00Z","not_valid_before":"2026-01-01T00:00:00Z","not_valid_after":"2027-01-01T00:00:00Z","domain_names":["example.com"],"fingerprint":"03:c7:55","status":{"issuance":"completed","renewal":"scheduled","error":null},"used_by":[]}}"#;
const ZONEFILE: &[u8] = br#"{"zonefile":"$ORIGIN example.com.\n"}"#;
const STORAGE_BOXES: &[u8] = br#"{"storage_boxes":[{"id":42,"name":"backup","storage_box_type":{"id":7,"name":"bx11","description":"BX11","snapshot_limit":10,"automatic_snapshot_limit":10,"subaccounts_limit":200,"size":1073741824,"prices":[{"location":"fsn1","price_hourly":{"net":"1.0000","gross":"1.1900"},"price_monthly":{"net":"5.0000","gross":"5.9500"},"setup_fee":{"net":"0.0000","gross":"0.0000"}}],"deprecation":null},"location":{"id":1,"name":"fsn1","description":"Falkenstein DC Park 1","country":"DE","city":"Falkenstein","latitude":50.47612,"longitude":12.370071,"network_zone":"eu-central"},"access_settings":{"reachable_externally":false,"samba_enabled":true,"ssh_enabled":true,"webdav_enabled":false,"zfs_enabled":true},"snapshot_plan":null,"protection":{"delete":true},"labels":{},"status":"active","username":"u12345","server":"u12345.your-storagebox.de","system":"FSN1-BX355","stats":{"size":3,"size_data":2,"size_snapshots":1},"created":"2026-01-01T00:00:00Z"}],"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#;
const ACTION: &[u8] = br#"{"action":{"id":42,"command":"poweron","status":"running","progress":0,"started":"2026-01-01T00:00:00Z","finished":null,"resources":[{"id":42,"type":"server"}],"error":null}}"#;

struct FixedClock;

#[derive(Clone, Copy)]
enum ReadSuccess {
    Locations,
    Certificate,
    ZoneFile,
    StorageBoxes,
}

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
    exercise_read_modes(prepared, cloud, LOCATIONS, ReadSuccess::Locations);

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
    exercise_read_modes(prepared, cloud, CERTIFICATE, ReadSuccess::Certificate);

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
    exercise_read_modes(prepared, cloud, ZONEFILE, ReadSuccess::ZoneFile);

    let boxes = AssociatedOperation::<ListStorageBoxes, _>::endpoint(StorageBoxEndpoint::List);
    let Ok(boxes) = boxes else {
        unreachable!("Storage Box association failed")
    };
    let prepared = boxes.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("Storage Box preparation failed")
    };
    assert_eq!(
        <ListStorageBoxes as HetznerOperation>::DESCRIPTOR.authentication(),
        AuthenticationClass::Bearer,
    );
    exercise_read_modes(prepared, storage, STORAGE_BOXES, ReadSuccess::StorageBoxes);
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

fn exercise_read_modes<O>(
    prepared: Prepared<'_, O>,
    endpoint: EndpointIdentity<'static>,
    response_body: &'static [u8],
    expected_success: ReadSuccess,
) where
    O: Copy + HetznerOperation + ReadOnlyOperation,
{
    let request = prepared.as_untyped().transport_request();
    let expected = ExpectedRequest::new(request.method(), request.target())
        .with_body(request.body())
        .with_headers(request.headers());
    let fixture = || {
        let body = FixtureBody::new(response_body);
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

    let mut body = [0_u8; 8_192];
    let mut headers = [0_u8; 8_192];
    let blocking = prepared.execute_blocking(&mock, &mut body, &mut headers);
    let Ok(blocking) = blocking else {
        unreachable!("blocking vertical execution failed")
    };
    let decoded = decode_associated_checked_response(blocking);
    let Ok(decoded) = decoded else {
        unreachable!("blocking typed vertical decode failed")
    };
    assert_read_success(expected_success, decoded.success());

    let mut body = [0_u8; 8_192];
    let mut headers = [0_u8; 8_192];
    let future = prepared.execute_async(&mock, &mut body, &mut headers);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(asynchronous)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("Send-async vertical execution failed")
    };
    let decoded = decode_associated_checked_response(asynchronous);
    let Ok(decoded) = decoded else {
        unreachable!("Send-async typed vertical decode failed")
    };
    assert_read_success(expected_success, decoded.success());
    assert!(mock.is_complete());

    let exchanges = [MockExchange::new(expected, fixture())];
    let mock = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut body = [0_u8; 8_192];
    let mut headers = [0_u8; 8_192];
    let future = prepared.execute_local_async(&mock, &mut body, &mut headers);
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    let Poll::Ready(Ok(local)) = Future::poll(future.as_mut(), &mut context) else {
        unreachable!("local-async vertical execution failed")
    };
    let decoded = decode_associated_checked_response(local);
    let Ok(decoded) = decoded else {
        unreachable!("local-async typed vertical decode failed")
    };
    assert_read_success(expected_success, decoded.success());
    assert!(mock.is_complete());

    let invalid_body = FixtureBody::new(INVALID_JSON);
    let Ok(invalid_body) = invalid_body else {
        unreachable!("invalid response fixture body failed")
    };
    let exchanges = [MockExchange::new(
        expected,
        ResponseFixture::success(invalid_body).with_content_type("application/json"),
    )];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut body = [0_u8; 64];
    let mut headers = [0_u8; 8_192];
    let invalid = prepared.execute_blocking(&mock, &mut body, &mut headers);
    let Ok(invalid) = invalid else {
        unreachable!("invalid vertical execution failed before provider decoding")
    };
    assert!(decode_associated_checked_response(invalid).is_err());
}

fn assert_read_success(expected: ReadSuccess, success: &HetznerSuccess) {
    let matches = matches!(
        (expected, success),
        (ReadSuccess::Locations, HetznerSuccess::Locations(_))
            | (
                ReadSuccess::Certificate,
                HetznerSuccess::SecurityResource(
                    cloud_sdk_hetzner::serde::SecurityResource::Certificate(_),
                ),
            )
            | (ReadSuccess::ZoneFile, HetznerSuccess::ZoneFile(_))
            | (ReadSuccess::StorageBoxes, HetznerSuccess::StorageBoxes(_))
    );
    assert!(matches);
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
