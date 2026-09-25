use super::*;
use crate::discovery::tests::Fixture as _;
#[cfg(feature = "std")]
use crate::std as test_std;
use alloc::{format, vec, vec::Vec};
use cloud_sdk::transport::{
    BlockingStreamSink, BlockingStreamSource, StreamLimits, StreamPartialState, StreamRead,
    StreamReplayability,
};
#[cfg(feature = "blocking")]
mod execution;
mod metadata;
const META: &[u8] = br#"{"name":"serde","vers":"1.2.3+build.2","deps":[],"features":{},"authors":[],"description":"example","documentation":null,"homepage":null,"readme":null,"readme_file":null,"keywords":[],"categories":[],"license":"MIT OR Apache-2.0","license_file":null,"repository":null,"badges":{},"links":null,"rust_version":"1.92"}"#;
fn limits() -> StreamLimits {
    StreamLimits::new(1_000_000_000, 4096, 1_000_000, 4_000_000, 2).fixture("limits")
}
fn request(length: u64) -> PublishRequest<'static> {
    PublishRequest::new(
        PublishMetadata::from_json(META).fixture("metadata"),
        length,
        limits(),
    )
    .fixture("request")
}
#[derive(Default)]
struct Sink {
    bytes: Vec<u8>,
    committed: bool,
    aborted: bool,
    fail: bool,
    fail_commit: bool,
}
impl BlockingStreamSink for Sink {
    type Error = ();
    fn write_chunk(&mut self, bytes: &[u8]) -> Result<usize, ()> {
        if self.fail {
            return Err(());
        }
        let n = bytes.len().min(3);
        self.bytes
            .extend_from_slice(bytes.get(..n).fixture("prefix"));
        Ok(n)
    }
    fn commit(&mut self) -> Result<(), ()> {
        if self.fail_commit {
            return Err(());
        }
        self.committed = true;
        Ok(())
    }
    fn abort(&mut self, _: StreamPartialState) {
        self.aborted = true;
    }
}
#[test]
fn framing_is_exact_cargo_little_endian_across_every_small_chunk() {
    for chunk in 1..16 {
        let request = request(5);
        let mut source = SlicePackage::new(b"crate");
        let mut upload = PublishUpload::new(&request, &mut source).fixture("upload");
        let mut scratch = vec![0xa5; chunk];
        let mut sink = Sink::default();
        upload
            .transfer_to(&mut sink, &mut scratch)
            .fixture("transfer");
        let mut expected = Vec::new();
        expected.extend_from_slice(&u32::try_from(META.len()).fixture("length").to_le_bytes());
        expected.extend_from_slice(META);
        expected.extend_from_slice(&5u32.to_le_bytes());
        expected.extend_from_slice(b"crate");
        assert_eq!(sink.bytes, expected);
        assert_eq!(
            upload.content_length(),
            u64::try_from(expected.len()).fixture("length")
        );
        assert!(sink.committed && !sink.aborted && upload.complete);
        assert!(scratch.iter().all(|b| *b == 0));
        assert!(
            upload
                .transfer_to(&mut Sink::default(), &mut scratch)
                .is_err()
        );
        assert!(!upload.complete);
    }
}
#[test]
fn lengths_are_checked_without_allocating_archive_and_sources_cannot_lie() {
    for (size, valid) in [
        (0, false),
        (1, true),
        (MAX_PUBLISH_ARCHIVE_BYTES, true),
        (MAX_PUBLISH_ARCHIVE_BYTES.saturating_add(1), false),
        (u64::from(u32::MAX), false),
        (u64::MAX, false),
    ] {
        assert_eq!(
            PublishRequest::new(
                PublishMetadata::from_json(META).fixture("metadata"),
                size,
                limits()
            )
            .is_ok(),
            valid
        );
    }
    for bytes in [b"short".as_slice(), b"too-long"] {
        let request = request(6);
        let mut source = SlicePackage::new(bytes);
        let mut upload = PublishUpload::new(&request, &mut source).fixture("upload");
        let mut sink = Sink::default();
        let mut scratch = [0xa5; 64];
        assert!(upload.transfer_to(&mut sink, &mut scratch).is_err());
        assert!(sink.aborted && !sink.committed && !upload.complete);
        assert_eq!(scratch, [0; 64]);
    }
}
struct Fault(u8);
impl BlockingStreamSource for Fault {
    type Error = ();
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, ()> {
        match self.0 {
            0 => Err(()),
            1 => Ok(StreamRead::Wait),
            _ => Ok(StreamRead::Chunk(output.len().saturating_add(1))),
        }
    }
}
#[test]
fn source_failure_wait_overclaim_and_sink_cancellation_abort_and_erase() {
    for fault in 0..3 {
        let request = request(6);
        let mut source = Fault(fault);
        let mut upload = PublishUpload::new(&request, &mut source).fixture("upload");
        let mut sink = Sink::default();
        let mut scratch = [0xa5; 64];
        assert!(upload.transfer_to(&mut sink, &mut scratch).is_err());
        assert!(sink.aborted && !sink.committed && !upload.complete);
        assert_eq!(scratch, [0; 64]);
    }
    let request = request(5);
    let mut source = SlicePackage::new(b"crate");
    let mut upload = PublishUpload::new(&request, &mut source).fixture("upload");
    let mut sink = Sink {
        fail: true,
        ..Sink::default()
    };
    let mut scratch = [0xa5; 64];
    assert!(upload.transfer_to(&mut sink, &mut scratch).is_err());
    assert!(sink.aborted && !upload.complete);
    assert_eq!(scratch, [0; 64]);
}

#[test]
fn empty_scratch_and_commit_failure_never_complete_an_upload() {
    for size in [0, 64] {
        let request = request(5);
        let mut source = SlicePackage::new(b"crate");
        let mut upload = PublishUpload::new(&request, &mut source).fixture("upload");
        let mut sink = Sink {
            fail_commit: true,
            ..Sink::default()
        };
        let mut scratch = vec![0xa5; size];
        assert!(upload.transfer_to(&mut sink, &mut scratch).is_err());
        assert!(sink.aborted && !sink.committed && !upload.complete);
        assert!(scratch.iter().all(|b| *b == 0));
    }
}

#[test]
#[cfg(feature = "std")]
fn unwinding_source_aborts_direct_sink_and_erases_upload_scratch() {
    struct Interrupted;
    impl BlockingStreamSource for Interrupted {
        type Error = ();
        fn replayability(&self) -> StreamReplayability<'_> {
            StreamReplayability::NotReplayable
        }
        fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, ()> {
            output.fill(0xa5);
            unreachable!("injected source unwind");
        }
    }
    let request = request(5);
    let mut source = Interrupted;
    let mut upload = PublishUpload::new(&request, &mut source).fixture("upload");
    let mut sink = Sink::default();
    let mut scratch = [0xa5; 64];
    let outcome = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let _ = upload.transfer_to(&mut sink, &mut scratch);
    }));
    assert!(outcome.is_err());
    assert!(sink.aborted && !sink.committed && !upload.complete);
    assert_eq!(scratch, [0; 64]);
}
