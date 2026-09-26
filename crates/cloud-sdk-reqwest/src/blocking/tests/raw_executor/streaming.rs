use super::*;
use cloud_sdk::transport::{
    BlockingAuthorizedRawHttpExecutor, BlockingStreamSource, BoundTransport, EndpointIdentity,
    EndpointScheme, HeaderValue, RequestTarget, StreamRead,
};

fn request() -> TransportRequest<'static> {
    TransportRequest::new(
        Method::Get,
        RequestTarget::new("/artifact").unwrap_or_else(|_| unreachable!()),
    )
}
#[test]
fn blocking_stream_reads_partial_buffers_to_eof() {
    let server = spawn_raw_response(
        b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nabcdef",
    )
    .unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
    let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
    let mut source = client
        .open_stream(request(), 6)
        .unwrap_or_else(|e| unreachable!("open: {e}"));
    assert_eq!(source.content_length(), Some(6));
    let mut observed = std::vec::Vec::new();
    for _ in 0..10 {
        let mut bytes = [0; 2];
        match source
            .read_chunk(&mut bytes)
            .unwrap_or_else(|e| unreachable!("read: {e}"))
        {
            StreamRead::Chunk(length) => {
                observed.extend_from_slice(bytes.get(..length).unwrap_or_else(|| unreachable!()))
            }
            StreamRead::End => break,
            StreamRead::Wait => (),
        }
    }
    assert_eq!(observed, b"abcdef");
    assert_eq!(source.read_chunk(&mut [0; 2]), Ok(StreamRead::End));
}

#[test]
fn blocking_authorization_checks_actual_destination_before_sending() {
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
    let token = std::format!("fixture-{}", std::process::id());
    let authorization = HeaderValue::new(&token).unwrap_or_else(|_| unreachable!());
    let mut body = [0; 16];
    let mut headers = [0; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
    let policy = json_policy(&[]).unwrap_or_else(|| unreachable!());
    assert!(
        client
            .execute_authorized(other, authorization, request(), policy, response.writer())
            .is_err()
    );
    assert!(server.request.try_recv().is_err());
    assert!(
        client
            .execute_authorized(
                expected,
                authorization,
                request(),
                policy,
                response.writer()
            )
            .is_ok()
    );
    let request = server
        .request
        .recv_timeout(Duration::from_secs(2))
        .unwrap_or_else(|e| unreachable!("request: {e}"));
    let text = String::from_utf8(request.bytes).unwrap_or_else(|_| unreachable!());
    assert!(text.contains(&std::format!("authorization: {token}\r\n")));
    assert!(!text.contains("Bearer "));
}

#[cfg(feature = "async-rustls")]
#[test]
fn blocking_stream_can_be_dropped_in_an_async_runtime_without_panicking() {
    let server = spawn_raw_response(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nabc")
        .unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
    let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
    let source = client
        .open_stream(request(), 3)
        .unwrap_or_else(|_| unreachable!());
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|_| unreachable!());
    runtime.block_on(async {
        drop(source);
    });
}
