use super::*;
use cloud_sdk::transport::{AsyncStreamSink, drive_async_stream};
use core::{
    future::Future,
    task::{Context, Poll, Waker},
};

impl AsyncStreamSink for Sink {
    type Error = ();
    async fn write_chunk<'a>(&'a mut self, bytes: &'a [u8]) -> Result<usize, ()> {
        BlockingStreamSink::write_chunk(self, bytes)
    }
    async fn commit(&mut self) -> Result<(), ()> {
        BlockingStreamSink::commit(self)
    }
    fn abort(&mut self, state: StreamPartialState) {
        BlockingStreamSink::abort(self, state);
    }
}
pub(super) fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    for _ in 0..4096 {
        if let Poll::Ready(result) = future
            .as_mut()
            .poll(&mut Context::from_waker(Waker::noop()))
        {
            return result;
        }
    }
    unreachable!("fixture did not finish")
}
fn send<F: Future + Send>(future: F) -> F {
    future
}

#[test]
fn async_publish_framing_matches_independent_cargo_bytes_and_rejects_bad_lengths() {
    for chunk in 1..16 {
        for bytes in [b"crate".as_slice(), b"crat", b"crates"] {
            let request = request(5);
            let mut source = SlicePackage::new(bytes);
            let mut framed =
                super::super::stream::Framed::new(&request, &mut source).fixture("framing");
            let mut scratch = vec![0xa5; chunk];
            let mut sink = Sink::default();
            let mut outcome = cloud_sdk::transport::StreamOutcome::new();
            let result = ready(send(drive_async_stream(
                request.policy,
                &mut framed,
                &mut sink,
                &mut scratch,
                &mut outcome,
            )));
            assert_eq!(result.is_ok(), bytes == b"crate");
            assert_eq!(sink.committed, bytes == b"crate");
            assert_eq!(sink.aborted, bytes != b"crate");
            assert!(scratch.iter().all(|b| *b == 0));
            if bytes == b"crate" {
                let mut expected = Vec::new();
                expected
                    .extend_from_slice(&u32::try_from(META.len()).fixture("length").to_le_bytes());
                expected.extend_from_slice(META);
                expected.extend_from_slice(&5_u32.to_le_bytes());
                expected.extend_from_slice(b"crate");
                assert_eq!(sink.bytes, expected);
            }
        }
    }
}

#[cfg(feature = "async-rustls")]
#[test]
fn local_publish_framing_accepts_non_send_source() {
    use cloud_sdk::transport::{LocalAsyncStreamSource, drive_local_stream};
    struct Local<'a>(SlicePackage<'a>, alloc::rc::Rc<()>);
    impl LocalAsyncStreamSource for Local<'_> {
        type Error = PublishError;
        fn replayability(&self) -> StreamReplayability<'_> {
            StreamReplayability::NotReplayable
        }
        async fn read_chunk_local<'a>(
            &'a mut self,
            output: &'a mut [u8],
        ) -> Result<StreamRead, PublishError> {
            assert_eq!(alloc::rc::Rc::strong_count(&self.1), 1);
            BlockingStreamSource::read_chunk(&mut self.0, output)
        }
    }
    let request = request(5);
    let mut source = Local(SlicePackage::new(b"crate"), alloc::rc::Rc::new(()));
    let mut framed = super::super::stream::asynchronous::LocalFramed::new(&request, &mut source)
        .fixture("framing");
    let mut sink = Sink::default();
    let mut scratch = [0xa5; 7];
    ready(drive_local_stream(
        request.policy,
        &mut framed,
        &mut sink,
        &mut scratch,
        &mut cloud_sdk::transport::StreamOutcome::new(),
    ))
    .fixture("stream");
    let mut expected = Vec::new();
    expected.extend_from_slice(&u32::try_from(META.len()).fixture("length").to_le_bytes());
    expected.extend_from_slice(META);
    expected.extend_from_slice(&5_u32.to_le_bytes());
    expected.extend_from_slice(b"crate");
    assert_eq!(sink.bytes, expected);
    assert!(sink.committed);
    assert_eq!(scratch, [0; 7]);
}
