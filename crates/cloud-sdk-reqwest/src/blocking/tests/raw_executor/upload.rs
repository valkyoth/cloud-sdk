use super::*;
use cloud_sdk::transport::{
    AuthorizedUpload, BlockingRawUploadExecutor, BlockingStreamSource, BoundTransport, HeaderValue,
    RequestHeader, RequestHeaders, RequestTarget, StreamFraming, StreamKind, StreamLimits,
    StreamPolicy, StreamRead, StreamReplayability, StreamSinkMode,
};

struct Source {
    bytes: &'static [u8],
    _local: std::rc::Rc<()>,
}
impl BlockingStreamSource for Source {
    type Error = ();
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, ()> {
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
#[test]
fn blocking_upload_accepts_non_send_sources_and_exact_framing() {
    let server = spawn_raw_response(
        b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\n\r\n{}",
    )
    .unwrap_or_else(|e| unreachable!("server: {e}"));
    let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
    let mut source = Source {
        bytes: b"abcde",
        _local: std::rc::Rc::new(()),
    };
    let mut scratch = [0xa5; 2];
    let mut body = [0xa5; 16];
    let mut headers = [0xa5; 128];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
    let stream_policy = StreamPolicy::new(
        StreamKind::FiniteUpload,
        StreamFraming::Declared(5),
        StreamSinkMode::Direct,
        StreamLimits::new(5, 2, 8, 32, 2).unwrap_or_else(|_| unreachable!()),
    )
    .unwrap_or_else(|_| unreachable!());
    let value = std::format!("fixture-{}", std::process::id());
    let upload = AuthorizedUpload {
        expected: client
            .endpoint_identity()
            .unwrap_or_else(|_| unreachable!()),
        authorization: HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
        source: &mut source,
        policy: stream_policy,
        scratch: &mut scratch,
    };
    let request_headers = [
        RequestHeader::new("content-type", "application/octet-stream")
            .unwrap_or_else(|_| unreachable!()),
    ];
    let request = TransportRequest::new(
        Method::Put,
        RequestTarget::new("/upload").unwrap_or_else(|_| unreachable!()),
    )
    .with_headers(RequestHeaders::new(&request_headers).unwrap_or_else(|_| unreachable!()));
    assert_eq!(
        client.upload(
            request,
            json_policy(&[]).unwrap_or_else(|| unreachable!()),
            upload,
            response.writer()
        ),
        Ok(())
    );
    assert_eq!(scratch, [0; 2]);
    assert!(source.bytes.is_empty());
    let wire = server
        .request
        .recv_timeout(Duration::from_secs(2))
        .unwrap_or_else(|e| unreachable!("request: {e}"));
    assert!(wire.bytes.ends_with(b"\r\n\r\nabcde"));
}
