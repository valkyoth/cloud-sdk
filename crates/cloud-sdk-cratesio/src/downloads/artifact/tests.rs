use super::*;
use crate::discovery::tests::Fixture as _;
use alloc::vec::Vec;
use cloud_sdk::transport::*;
use core::{
    future::Future,
    task::{Context, Poll, Waker},
};

const UA: &str = "artifact-test/1 (test@example.org)";
// A test hook checks exact accepted input; this is not a SHA-256 implementation.
struct Checksum(Vec<u8>);
impl ArtifactChecksum for Checksum {
    fn update(&mut self, bytes: &[u8]) -> Result<(), ArtifactError> {
        self.0.extend_from_slice(bytes);
        Ok(())
    }
    fn finish(&mut self) -> Result<[u8; 32], ArtifactError> {
        if self.0 != b"abc" {
            return Err(ArtifactError::Checksum);
        }
        Ok([7; 32])
    }
}
struct Source {
    bytes: &'static [u8],
    offset: usize,
    pending: bool,
    fail: bool,
}
impl BlockingStreamSource for Source {
    type Error = ArtifactError;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Self::Error> {
        if self.fail {
            return Err(ArtifactError::Transport);
        }
        let rest = self.bytes.get(self.offset..).ok_or(ArtifactError::Stream)?;
        if rest.is_empty() {
            return Ok(StreamRead::End);
        }
        let len = rest.len().min(output.len());
        output
            .get_mut(..len)
            .ok_or(ArtifactError::Stream)?
            .copy_from_slice(rest.get(..len).ok_or(ArtifactError::Stream)?);
        self.offset = self.offset.checked_add(len).fixture("offset");
        Ok(StreamRead::Chunk(len))
    }
}
impl AsyncStreamSource for Source {
    type Error = ArtifactError;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, Self::Error> {
        if self.pending && self.offset > 0 {
            core::future::pending::<()>().await;
        }
        BlockingStreamSource::read_chunk(self, output)
    }
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    committed: bool,
    aborts: usize,
    fail: bool,
}
impl BlockingStreamSink for Sink {
    type Error = ArtifactError;
    fn write_chunk(&mut self, input: &[u8]) -> Result<usize, Self::Error> {
        if self.fail {
            return Err(ArtifactError::Stream);
        }
        // Deliberately partial acceptance exercises correct checksum accounting.
        self.bytes
            .extend_from_slice(input.get(..1).ok_or(ArtifactError::Stream)?);
        Ok(1)
    }
    fn commit(&mut self) -> Result<(), Self::Error> {
        self.committed = true;
        Ok(())
    }
    fn abort(&mut self, _: StreamPartialState) {
        cloud_sdk::buffer::sanitize_bytes(&mut self.bytes);
        self.bytes.clear();
        self.aborts = self.aborts.checked_add(1).fixture("aborts");
    }
}
impl AsyncStreamSink for Sink {
    type Error = ArtifactError;
    async fn write_chunk<'a>(&'a mut self, input: &'a [u8]) -> Result<usize, Self::Error> {
        BlockingStreamSink::write_chunk(self, input)
    }
    async fn commit(&mut self) -> Result<(), Self::Error> {
        BlockingStreamSink::commit(self)
    }
    fn abort(&mut self, state: StreamPartialState) {
        BlockingStreamSink::abort(self, state);
    }
}
struct Transport {
    bytes: &'static [u8],
    length: Option<u64>,
    pending: bool,
    official: bool,
    fail: bool,
}
impl BoundTransport for Transport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        if self.official {
            Ok(OfficialCratesIoEndpoint::static_downloads()
                .identity()
                .fixture("static endpoint"))
        } else {
            Ok(OfficialCratesIoEndpoint::production_api()
                .identity()
                .fixture("API endpoint"))
        }
    }
}
impl BoundUserAgent for Transport {
    fn configured_user_agent(&self) -> &[u8] {
        UA.as_bytes()
    }
}
impl BlockingArtifactTransport for Transport {
    type Source<'a> = Source;
    fn open(&self, request: TransportRequest<'_>) -> Result<OpenedArtifact<Source>, ArtifactError> {
        assert_eq!(request.method(), Method::Get);
        assert_eq!(request.target().as_str(), "/crates/serde/serde-1.0.0.crate");
        assert!(request.headers().get("authorization").is_none());
        assert!(request.headers().get("cookie").is_none());
        assert!(request.headers().as_slice().is_empty());
        assert!(request.body().is_empty());
        OpenedArtifact::new(
            Source {
                bytes: self.bytes,
                offset: 0,
                pending: self.pending,
                fail: self.fail,
            },
            StatusCode::OK,
            self.length,
            None,
        )
    }
}
impl AsyncArtifactTransport for Transport {
    type Source<'a> = Source;
    async fn open_async<'a, 'r>(
        &'a self,
        request: TransportRequest<'r>,
    ) -> Result<OpenedArtifact<Source>, ArtifactError>
    where
        'a: 'r,
    {
        BlockingArtifactTransport::open(self, request)
    }
}
impl LocalArtifactTransport for Transport {
    type Source<'a> = Source;
    async fn open_local<'a, 'r>(
        &'a self,
        request: TransportRequest<'r>,
    ) -> Result<OpenedArtifact<Source>, ArtifactError>
    where
        'a: 'r,
    {
        BlockingArtifactTransport::open(self, request)
    }
}
fn request() -> ArtifactDownload<'static> {
    ArtifactDownload::new(
        CrateName::new("serde").fixture("name"),
        Version::new("1.0.0").fixture("version"),
        3,
        [7; 32],
        StreamLimits::new(3, 2, 8, 32, 2).fixture("limits"),
    )
    .fixture("request")
}
fn transport() -> Transport {
    Transport {
        bytes: b"abc",
        length: Some(3),
        pending: false,
        official: true,
        fail: false,
    }
}
fn identity() -> IdentifyingUserAgent<'static> {
    IdentifyingUserAgent::new(UA).fixture("identity")
}
fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    match future
        .as_mut()
        .poll(&mut Context::from_waker(Waker::noop()))
    {
        Poll::Ready(result) => result,
        Poll::Pending => unreachable!("fixture did not finish"),
    }
}
fn send<F: Future + Send>(future: F) -> F {
    future
}
#[test]
fn all_modes_stream_partial_writes_and_verify_before_commit() {
    for mode in 0..3 {
        let mut sink = Sink::default();
        let mut scratch = [0xa5; 2];
        let transport = transport();
        let checksum = Checksum(Vec::new());
        match mode {
            0 => request().execute(&transport, identity(), &mut sink, checksum, &mut scratch),
            1 => ready(send(request().execute_async(
                &transport,
                identity(),
                &mut sink,
                checksum,
                &mut scratch,
            ))),
            _ => ready(request().execute_local(
                &transport,
                identity(),
                &mut sink,
                checksum,
                &mut scratch,
            )),
        }
        .fixture("stream");
        assert_eq!(sink.bytes, b"abc");
        assert!(sink.committed);
        assert_eq!(sink.aborts, 0);
        assert_eq!(scratch, [0; 2]);
    }
}
#[test]
fn errors_never_commit_and_clear_scratch() {
    for mode in 0..3 {
        for case in 0..7 {
            let mut transport = transport();
            let mut sink = Sink::default();
            match case {
                0 => transport.length = Some(4),
                1 => transport.bytes = b"ab",
                2 => transport.bytes = b"abcd",
                3 => transport.bytes = b"abd",
                4 => transport.official = false,
                5 => transport.fail = true,
                _ => sink.fail = true,
            }
            let mut scratch = [0xa5; 2];
            let checksum = Checksum(Vec::new());
            let result = match mode {
                0 => request().execute(&transport, identity(), &mut sink, checksum, &mut scratch),
                1 => ready(send(request().execute_async(
                    &transport,
                    identity(),
                    &mut sink,
                    checksum,
                    &mut scratch,
                ))),
                _ => ready(request().execute_local(
                    &transport,
                    identity(),
                    &mut sink,
                    checksum,
                    &mut scratch,
                )),
            };
            assert!(result.is_err());
            assert!(!sink.committed);
            assert_eq!(sink.aborts, 1);
            assert!(sink.bytes.is_empty());
            assert_eq!(scratch, [0; 2]);
        }
    }
}
#[test]
fn cancelled_and_unpolled_streams_abort() {
    for polled in [false, true] {
        let transport = Transport {
            pending: true,
            ..transport()
        };
        let mut sink = Sink::default();
        let mut scratch = [0xa5; 2];
        {
            let future = request().execute_async(
                &transport,
                identity(),
                &mut sink,
                Checksum(Vec::new()),
                &mut scratch,
            );
            if polled {
                let mut future = core::pin::pin!(future);
                assert!(
                    future
                        .as_mut()
                        .poll(&mut Context::from_waker(Waker::noop()))
                        .is_pending()
                );
            } else {
                drop(future);
            }
        }
        assert!(!sink.committed);
        assert_eq!(sink.aborts, 1);
        assert!(sink.bytes.is_empty());
        assert_eq!(scratch, [0; 2]);
    }
}
#[test]
fn head_and_empty_buffer_checks_fail_closed() {
    for status in [201, 206, 302, 404, 500] {
        assert!(
            OpenedArtifact::new((), StatusCode::new(status).fixture("status"), Some(3), None)
                .is_err()
        );
    }
    assert!(OpenedArtifact::new((), StatusCode::OK, None, Some("gzip")).is_err());
    let mut sink = Sink::default();
    assert!(
        request()
            .execute(
                &transport(),
                identity(),
                &mut sink,
                Checksum(Vec::new()),
                &mut []
            )
            .is_err()
    );
    assert_eq!(sink.aborts, 1);
    let mut output = [0xa5; 2];
    assert!(request().write_target(&mut output).is_err());
    assert_eq!(output, [0xa5; 2]);
}

#[test]
fn local_cancellation_clears_unpolled_and_partial_transfers() {
    for polled in [false, true] {
        let transport = Transport {
            pending: true,
            ..transport()
        };
        let mut sink = Sink::default();
        let mut scratch = [0xa5; 2];
        {
            let future = request().execute_local(
                &transport,
                identity(),
                &mut sink,
                Checksum(Vec::new()),
                &mut scratch,
            );
            if polled {
                let mut future = core::pin::pin!(future);
                assert!(
                    future
                        .as_mut()
                        .poll(&mut Context::from_waker(Waker::noop()))
                        .is_pending()
                );
            } else {
                drop(future);
            }
        }
        assert!(!sink.committed);
        assert_eq!(sink.aborts, 1);
        assert!(sink.bytes.is_empty());
        assert_eq!(scratch, [0; 2]);
    }
}
