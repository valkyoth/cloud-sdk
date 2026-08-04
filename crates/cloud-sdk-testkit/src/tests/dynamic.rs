use alloc::format;
use core::sync::atomic::{AtomicUsize, Ordering};

use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, RequestTarget, ResponseBuffer, StatusCode, TransportRequest, drive_async,
};

use crate::{
    DynamicMockConfigError, DynamicMockError, DynamicMockTransport, DynamicRequest,
    DynamicResponder, FixtureBody, MAX_DYNAMIC_RECORDS, RecordedMethod, RequestRecordSlot,
    ResponseFixture,
};

#[derive(Clone, Copy, Eq, PartialEq)]
struct Rejected;

fn fixture(body: &'static [u8]) -> ResponseFixture<'static> {
    let Ok(body) = FixtureBody::new(body) else {
        unreachable!()
    };
    ResponseFixture::success(body)
}

#[test]
fn rejection_and_small_response_storage_do_not_consume_or_record() {
    let fixture = fixture(b"hello");
    let Ok(expected) = RequestTarget::new("/expected") else {
        unreachable!("testkit security fixture construction failed");
    };
    let responder = DynamicResponder::new(|request: DynamicRequest<'_>| {
        if request.target() == expected {
            Ok(&fixture)
        } else {
            Err(Rejected)
        }
    });
    let slots = [const { RequestRecordSlot::new() }; 1];
    let Ok(transport) = DynamicMockTransport::new(responder, &slots) else {
        unreachable!("testkit security fixture construction failed");
    };

    let Ok(wrong) = RequestTarget::new("/wrong") else {
        unreachable!("testkit security fixture construction failed");
    };
    let mut body = [0xa5_u8; 5];
    let mut headers = [0xa5_u8; 64];
    let mut response = ResponseBuffer::new(&mut body, 5, &mut headers);
    assert!(matches!(
        transport.send(TransportRequest::new(Method::Get, wrong), response.writer()),
        Err(DynamicMockError::Builder(Rejected))
    ));
    assert_eq!(transport.recorded(), 0);
    assert_eq!(transport.record(0), None);

    let mut small_body = [0xa5_u8; 4];
    let mut small_headers = [0xa5_u8; 64];
    let mut small_response = ResponseBuffer::new(&mut small_body, 4, &mut small_headers);
    assert!(matches!(
        transport.send(
            TransportRequest::new(Method::Get, expected),
            small_response.writer()
        ),
        Err(DynamicMockError::Fixture(_))
    ));
    assert_eq!(transport.recorded(), 0);

    assert!(
        transport
            .send(
                TransportRequest::new(Method::Get, expected),
                response.writer()
            )
            .is_ok()
    );
    let Some(record) = transport.record(0) else {
        unreachable!("testkit security fixture construction failed");
    };
    assert_eq!(record.sequence(), 0);
    assert_eq!(record.method(), RecordedMethod::Get);
    assert_eq!(record.target_len(), expected.len());
    assert_eq!(record.body_len(), 0);
    assert_eq!(record.header_count(), 0);
    assert_eq!(record.status(), StatusCode::OK);
    assert!(!format!("{record:?}").contains("expected"));
}

#[test]
fn capacity_exhaustion_is_fail_closed() {
    let fixture = fixture(b"x");
    let responder = DynamicResponder::new(|_: DynamicRequest<'_>| Ok::<_, Rejected>(&fixture));
    let slots = [const { RequestRecordSlot::new() }; 1];
    let Ok(transport) = DynamicMockTransport::new(responder, &slots) else {
        unreachable!("testkit security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/bounded") else {
        unreachable!("testkit security fixture construction failed");
    };
    for expected_success in [true, false] {
        let mut body = [0_u8; 1];
        let mut headers = [0_u8; 32];
        let mut response = ResponseBuffer::new(&mut body, 1, &mut headers);
        let result = transport.send(
            TransportRequest::new(Method::Post, target),
            response.writer(),
        );
        assert_eq!(result.is_ok(), expected_success);
        if !expected_success {
            assert!(matches!(result, Err(DynamicMockError::Exhausted)));
        }
    }
    assert_eq!(transport.recorded(), 1);
}

#[test]
fn configuration_and_generic_error_diagnostics_are_payload_free() {
    let responder =
        DynamicResponder::new(|_: DynamicRequest<'_>| Err::<&ResponseFixture<'_>, _>(Rejected));
    assert!(matches!(
        DynamicMockTransport::new(responder, &[]),
        Err(DynamicMockConfigError::NoRecordSlots)
    ));
    let error = DynamicMockError::Builder("credential-shaped-payload");
    assert_eq!(
        format!("{error:?}"),
        "DynamicMockError::Builder([redacted])"
    );
    assert!(!format!("{error}").contains("credential"));
}

#[test]
fn exact_recording_cap_is_admitted_and_plus_one_is_rejected() {
    let maximum = [const { RequestRecordSlot::new() }; MAX_DYNAMIC_RECORDS];
    let responder =
        DynamicResponder::new(|_: DynamicRequest<'_>| Err::<&ResponseFixture<'_>, _>(Rejected));
    assert!(DynamicMockTransport::new(responder, &maximum).is_ok());

    let oversized = [const { RequestRecordSlot::new() }; MAX_DYNAMIC_RECORDS + 1];
    let responder =
        DynamicResponder::new(|_: DynamicRequest<'_>| Err::<&ResponseFixture<'_>, _>(Rejected));
    assert!(matches!(
        DynamicMockTransport::new(responder, &oversized),
        Err(DynamicMockConfigError::TooManyRecordSlots)
    ));
}

#[test]
fn committed_slots_cannot_be_silently_reused() {
    let fixture = fixture(b"x");
    let slots = [const { RequestRecordSlot::new() }; 1];
    {
        let responder = DynamicResponder::new(|_: DynamicRequest<'_>| Ok::<_, Rejected>(&fixture));
        let Ok(transport) = DynamicMockTransport::new(responder, &slots) else {
            unreachable!("testkit security fixture construction failed");
        };
        let Ok(target) = RequestTarget::new("/once") else {
            unreachable!("testkit security fixture construction failed");
        };
        let mut body = [0_u8; 1];
        let mut headers = [0_u8; 32];
        let mut response = ResponseBuffer::new(&mut body, 1, &mut headers);
        assert!(
            transport
                .send(
                    TransportRequest::new(Method::Get, target),
                    response.writer()
                )
                .is_ok()
        );
    }
    let responder =
        DynamicResponder::new(|_: DynamicRequest<'_>| Err::<&ResponseFixture<'_>, _>(Rejected));
    assert!(matches!(
        DynamicMockTransport::new(responder, &slots),
        Err(DynamicMockConfigError::DirtyRecordSlot)
    ));
}

#[test]
fn dropping_an_unpolled_future_does_not_invoke_or_record() {
    let fixture = fixture(b"x");
    let calls = AtomicUsize::new(0);
    let responder = DynamicResponder::new(|_: DynamicRequest<'_>| {
        calls.fetch_add(1, Ordering::Relaxed);
        Ok::<_, Rejected>(&fixture)
    });
    let slots = [const { RequestRecordSlot::new() }; 1];
    let Ok(transport) = DynamicMockTransport::new(responder, &slots) else {
        unreachable!("testkit security fixture construction failed");
    };
    let Ok(target) = RequestTarget::new("/cancel") else {
        unreachable!("testkit security fixture construction failed");
    };
    let mut body = [0xa5_u8; 1];
    let mut headers = [0xa5_u8; 32];
    {
        let mut response = ResponseBuffer::new(&mut body, 1, &mut headers);
        let future = drive_async(
            &transport,
            TransportRequest::new(Method::Get, target),
            response.writer(),
        );
        drop(future);
    }
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(transport.recorded(), 0);
    assert_eq!(body, [0_u8; 1]);
    assert_eq!(headers, [0_u8; 32]);
}
