use core::future::Future;
use core::task::{Context, Poll, Waker};

use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, ResponseBuffer, TransportRequest, drive_local};

use crate::{ExpectedRequest, FixtureBody, LocalMockTransport, MockExchange, ResponseFixture};

#[test]
fn local_async_mock_supports_cooperatively_outstanding_requests() {
    let Ok(target) = RequestTarget::new("/local") else {
        unreachable!("testkit security fixture construction failed");
    };
    let Ok(body) = FixtureBody::new(b"ok") else {
        unreachable!("testkit security fixture construction failed");
    };
    let exchanges = [
        MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            ResponseFixture::success(body),
        ),
        MockExchange::new(
            ExpectedRequest::new(Method::Get, target),
            ResponseFixture::success(body),
        ),
    ];
    let transport = LocalMockTransport::new(&exchanges);
    let mut first_body = [0_u8; 2];
    let mut first_headers = [0_u8; 64];
    let mut first_response = ResponseBuffer::new(&mut first_body, 2, &mut first_headers);
    let mut second_body = [0_u8; 2];
    let mut second_headers = [0_u8; 64];
    let mut second_response = ResponseBuffer::new(&mut second_body, 2, &mut second_headers);
    {
        let first = drive_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            first_response.writer(),
        );
        let second = drive_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            second_response.writer(),
        );
        let mut first = core::pin::pin!(first);
        let mut second = core::pin::pin!(second);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(first.as_mut(), &mut context),
            Poll::Ready(Ok(()))
        ));
        assert!(matches!(
            Future::poll(second.as_mut(), &mut context),
            Poll::Ready(Ok(()))
        ));
    }
    assert!(transport.is_complete());
    assert!(
        first_response
            .with_response(|response| response.body() == b"ok")
            .is_ok_and(core::convert::identity)
    );
    assert!(
        second_response
            .with_response(|response| response.body() == b"ok")
            .is_ok_and(core::convert::identity)
    );
}

#[test]
fn dropping_unpolled_local_mock_future_preserves_the_exchange() {
    let Ok(target) = RequestTarget::new("/local-drop") else {
        unreachable!("testkit security fixture construction failed");
    };
    let Ok(body) = FixtureBody::new(b"ok") else {
        unreachable!("testkit security fixture construction failed");
    };
    let exchanges = [MockExchange::new(
        ExpectedRequest::new(Method::Get, target),
        ResponseFixture::success(body),
    )];
    let transport = LocalMockTransport::new(&exchanges);
    let mut output = [0xa5_u8; 2];
    let mut headers = [0xa5_u8; 64];
    {
        let mut response = ResponseBuffer::new(&mut output, 2, &mut headers);
        let future = drive_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            response.writer(),
        );
        drop(future);
    }
    assert_eq!(output, [0_u8; 2]);
    assert_eq!(headers, [0_u8; 64]);
    assert_eq!(transport.remaining(), 1);
}
