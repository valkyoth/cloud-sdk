use super::*;
use cloud_sdk::transport::{
    BoundTransport, EndpointIdentity, EndpointScheme, HeaderValue, RequestTarget,
    drive_async_authorized_raw,
};

#[test]
fn explicit_authorization_is_destination_bound_and_not_prefixed() {
    run_async_test(async {
        let server = spawn_raw_response(
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
        )
        .unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
        let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
        let expected = client
            .endpoint_identity()
            .unwrap_or_else(|_| unreachable!());
        let other = EndpointIdentity::new(EndpointScheme::Https, "wrong.invalid", 443, "/")
            .unwrap_or_else(|_| unreachable!());
        let value = std::format!("fixture-{}", std::process::id());
        let authorization = HeaderValue::new(&value).unwrap_or_else(|_| unreachable!());
        let request = TransportRequest::new(
            Method::Get,
            RequestTarget::new("/test").unwrap_or_else(|_| unreachable!()),
        );
        let mut body = [0xa5; 16];
        let mut headers = [0xa5; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
        let policy = policy(2).unwrap_or_else(|| unreachable!());
        assert!(
            drive_async_authorized_raw(
                &client,
                other,
                authorization,
                request,
                policy,
                response.writer()
            )
            .await
            .is_err()
        );
        assert!(server.request.try_recv().is_err());
        assert!(
            drive_async_authorized_raw(
                &client,
                expected,
                authorization,
                request,
                policy,
                response.writer()
            )
            .await
            .is_ok()
        );
        let wire = server
            .request
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or_else(|e| unreachable!("request: {e}"));
        let wire = std::string::String::from_utf8(wire.bytes).unwrap_or_else(|_| unreachable!());
        assert!(wire.contains(&std::format!("authorization: {value}\r\n")));
        assert!(!wire.contains("Bearer "));
        drop(response);
        assert!(body.iter().all(|v| *v == 0));
        assert!(headers.iter().all(|v| *v == 0));
    });
}
