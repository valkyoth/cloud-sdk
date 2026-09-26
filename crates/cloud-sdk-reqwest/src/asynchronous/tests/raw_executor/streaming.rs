use super::*;
use cloud_sdk::transport::{AsyncStreamSource, RequestTarget, StreamRead};

fn request() -> TransportRequest<'static> {
    TransportRequest::new(
        Method::Get,
        RequestTarget::new("/artifact").unwrap_or_else(|_| unreachable!()),
    )
}
#[test]
fn streaming_returns_before_the_complete_body_and_cancellation_poison_source() {
    run_async_test(async {
        let server = spawn_raw_split(
            b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nabc",
            b"def",
            Duration::from_secs(2),
        )
        .unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
        let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
        let mut source =
            tokio::time::timeout(Duration::from_secs(1), client.open_stream(request(), 6))
                .await
                .unwrap_or_else(|_| unreachable!("body was preloaded"))
                .unwrap_or_else(|e| unreachable!("open: {e}"));
        assert_eq!(source.content_length(), Some(6));
        let mut output = [0; 3];
        assert_eq!(
            source.read_chunk(&mut output).await,
            Ok(StreamRead::Chunk(3))
        );
        assert_eq!(&output, b"abc");
        assert!(
            tokio::time::timeout(Duration::from_millis(30), source.read_chunk(&mut output))
                .await
                .is_err()
        );
        assert!(source.read_chunk(&mut output).await.is_err());
        let wire = server
            .request
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or_else(|e| unreachable!("request: {e}"));
        let text = std::string::String::from_utf8(wire.bytes)
            .unwrap_or_else(|_| unreachable!())
            .to_ascii_lowercase();
        assert!(text.contains("accept-encoding: identity"));
        assert!(!text.contains("authorization:"));
        assert!(!text.contains("cookie:"));
    });
}

#[test]
fn streaming_reads_chunked_bodies_incrementally_and_rejects_bad_heads() {
    run_async_test(async {
        let server = spawn_raw_response(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n3\r\ndef\r\n0\r\n\r\n")
            .unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
        let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
        let mut source = client
            .open_stream(request(), 6)
            .await
            .unwrap_or_else(|e| unreachable!("open: {e}"));
        assert_eq!(source.content_length(), None);
        let mut result = std::vec::Vec::new();
        for _ in 0..12 {
            let mut output = [0; 2];
            match source
                .read_chunk(&mut output)
                .await
                .unwrap_or_else(|e| unreachable!("read: {e}"))
            {
                StreamRead::Chunk(len) => {
                    result.extend_from_slice(output.get(..len).unwrap_or_else(|| unreachable!()))
                }
                StreamRead::End => break,
                StreamRead::Wait => (),
            }
        }
        assert_eq!(result, b"abcdef");
        assert_eq!(source.read_chunk(&mut [0; 1]).await, Ok(StreamRead::End));
        for wire in [
            b"HTTP/1.1 302 Found\r\nLocation: https://attacker.invalid/\r\nContent-Length: 0\r\n\r\n".as_slice(),
            b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\n\r\nabcdefg",
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nContent-Encoding: gzip\r\n\r\nabc",
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nTrailer: x-test\r\n\r\nabc",
            b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\nX-Test: a\r\nX-Test: b\r\n\r\nabc",
            b"HTTP/1.1 206 Partial Content\r\nContent-Length: 3\r\n\r\nabc",
        ] {
            let server = spawn_raw_response(wire).unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
            let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
            assert!(client.open_stream(request(), 6).await.is_err());
        }
    });
}

#[test]
fn streaming_rejects_truncation_overflow_trailers_and_expired_body_deadline() {
    run_async_test(async {
        for wire in [
            b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\nConnection: close\r\n\r\nabc".as_slice(),
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n7\r\nabcdefg\r\n0\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n3\r\nabc\r\n0\r\nx-secret: value\r\n\r\n",
        ] {
            let server = spawn_raw_response(wire).unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
            let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
            let mut source = client.open_stream(request(), 6).await.unwrap_or_else(|e| unreachable!("open: {e}"));
            let mut failed = false;
            for _ in 0..10 {
                if source.read_chunk(&mut [0; 4]).await.is_err() { failed = true; break; }
            }
            assert!(failed);
            assert!(source.read_chunk(&mut [0; 4]).await.is_err());
        }
        let server = spawn_raw_split(
            b"HTTP/1.1 200 OK\r\nContent-Length: 6\r\n\r\nabc",
            b"def",
            Duration::from_secs(2),
        )
        .unwrap_or_else(|e| unreachable!("loopback fixture: {e}"));
        let endpoint = crate::asynchronous::HttpsEndpoint::local_http(&server.endpoint)
            .unwrap_or_else(|_| unreachable!());
        let timeouts = crate::asynchronous::RequestTimeouts::new(
            Duration::from_millis(200),
            Duration::from_millis(100),
        )
        .unwrap_or_else(|_| unreachable!());
        let client = RawAsyncClientBuilder::new(
            endpoint,
            UserAgent::new("stream-test/1").unwrap_or_else(|_| unreachable!()),
            timeouts,
        )
        .build_for_loopback()
        .unwrap_or_else(|_| unreachable!());
        let mut source = client
            .open_stream(request(), 6)
            .await
            .unwrap_or_else(|e| unreachable!("open: {e}"));
        assert!(source.read_chunk(&mut [0; 3]).await.is_ok());
        let failure = source.read_chunk(&mut [0; 3]).await;
        assert_eq!(
            failure,
            Err(cloud_sdk::transport::TransportFailure::response_started(
                RawHttpError::TimedOut
            ))
        );
    });
}
