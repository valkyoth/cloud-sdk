use super::*;
use crate::asynchronous::RawUpload;
use cloud_sdk::transport::{
    AsyncRawUploadExecutor, AsyncStreamSource, AuthorizedUpload, BoundTransport, HeaderValue,
    LocalRawUploadExecutor, RequestHeader, RequestHeaders, RequestTarget, StreamFraming,
    StreamKind, StreamLimits, StreamPolicy, StreamRead, StreamReplayability, StreamSinkMode,
};

mod failures;
mod live;
mod policy;

struct Source {
    bytes: &'static [u8],
    reads: usize,
    pending: bool,
}
impl AsyncStreamSource for Source {
    type Error = ();
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, ()> {
        self.reads = self.reads.checked_add(1).ok_or(())?;
        if self.pending {
            core::future::pending::<()>().await;
        }
        if self.bytes.is_empty() {
            return Ok(StreamRead::End);
        }
        let n = output.len().min(self.bytes.len());
        output
            .get_mut(..n)
            .ok_or(())?
            .copy_from_slice(self.bytes.get(..n).ok_or(())?);
        self.bytes = self.bytes.get(n..).ok_or(())?;
        Ok(StreamRead::Chunk(n))
    }
}
fn stream_policy(length: u64) -> StreamPolicy {
    StreamPolicy::new(
        StreamKind::FiniteUpload,
        StreamFraming::Declared(length),
        StreamSinkMode::Direct,
        StreamLimits::new(64, 2, 64, 256, 2).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!())
}
fn request<'a>(headers: &'a [RequestHeader<'a>]) -> TransportRequest<'a> {
    TransportRequest::new(
        Method::Put,
        RequestTarget::new("/upload").unwrap_or_else(|_| unreachable!()),
    )
    .with_headers(RequestHeaders::new(headers).unwrap_or_else(|_| unreachable!()))
}
fn send<F: Future + Send>(future: F) -> F {
    future
}

#[test]
fn uploads_exact_bytes_with_explicit_authorization_in_both_async_modes() {
    run_async_test(async {
        for local in [false, true] {
            let server = spawn_raw_response(
                b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
            )
            .unwrap_or_else(|e| unreachable!("server: {e}"));
            let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
            let value = std::format!("fixture-{}", std::process::id());
            let mut source = Source {
                bytes: b"abcde",
                reads: 0,
                pending: false,
            };
            let mut scratch = [0xa5; 2];
            let mut body = [0xa5; 16];
            let mut headers = [0xa5; 128];
            let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
            let request_headers = [
                RequestHeader::new("content-type", "application/octet-stream")
                    .unwrap_or_else(|_| unreachable!()),
            ];
            let upload = AuthorizedUpload {
                expected: client
                    .endpoint_identity()
                    .unwrap_or_else(|_| unreachable!()),
                authorization: HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
                source: &mut source,
                policy: stream_policy(5),
                scratch: &mut scratch,
            };
            let result = if local {
                client
                    .upload_local(
                        request(&request_headers),
                        policy(2).unwrap_or_else(|| unreachable!()),
                        upload,
                        response.writer(),
                    )
                    .await
            } else {
                send(client.upload(
                    request(&request_headers),
                    policy(2).unwrap_or_else(|| unreachable!()),
                    upload,
                    response.writer(),
                ))
                .await
            };
            assert_eq!(result, Ok(()));
            assert_eq!(source.reads, 4);
            assert_eq!(scratch, [0; 2]);
            let wire = server
                .request
                .recv_timeout(Duration::from_secs(2))
                .unwrap_or_else(|e| unreachable!("request: {e}"));
            let wire =
                std::string::String::from_utf8(wire.bytes).unwrap_or_else(|_| unreachable!());
            assert!(wire.starts_with("PUT /v1/upload HTTP/1.1\r\n"));
            assert!(wire.contains("content-length: 5\r\n"));
            assert!(wire.contains(&std::format!("authorization: {value}\r\n")));
            assert!(!wire.contains("Bearer "));
            assert!(!wire.contains("transfer-encoding:"));
            assert!(wire.ends_with("\r\n\r\nabcde"));
        }
    });
}
