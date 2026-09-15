use crate::{
    endpoint::{OfficialCratesIoEndpoint, StaticDownloadTarget},
    identifiers::{CrateName, Version},
    wire::IdentifyingUserAgent,
};
use cloud_sdk::{
    Method,
    buffer::{SnapshotEncoder, encode_snapshot_bounded},
    transport::{
        BlockingStreamSink, BlockingStreamSource, BoundTransport, BoundUserAgent, StatusCode,
        StreamCompletion, StreamFraming, StreamKind, StreamLimits, StreamOutcome,
        StreamPartialState, StreamPolicy, StreamSinkMode, TransportRequest, drive_blocking_stream,
    },
};
use core::fmt;

mod asynchronous;
pub use asynchronous::{AsyncArtifactTransport, LocalArtifactTransport};
#[cfg(all(test, feature = "alloc"))]
mod tests;

/// Payload-free artifact preparation, header, transfer or checksum failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactError {
    /// Invalid limits or expected length.
    Policy,
    /// Caller target buffer is too small or its path is invalid.
    Target,
    /// Transport origin or identifying user agent differs from the request.
    Binding,
    /// Non-200 status, transformed body, or inconsistent content length.
    Headers,
    /// Trusted anonymous transport failed to open the response.
    Transport,
    /// Stream source, sink or progress policy failed; partial output is aborted.
    Stream,
    /// Caller checksum implementation failed or the archive digest differs.
    Checksum,
}
impl fmt::Display for ArtifactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("crate artifact operation failed")
    }
}
impl core::error::Error for ArtifactError {}

/// Incremental SHA-256 hook. Supply a reviewed implementation initialized fresh
/// for this archive. Never substitute a non-cryptographic hash. Expected digests
/// must come from trusted registry/index metadata, not the downloaded body.
pub trait ArtifactChecksum {
    /// Feed exactly the bytes accepted by the transactional sink.
    fn update(&mut self, bytes: &[u8]) -> Result<(), ArtifactError>;
    /// Finalize once; errors and mismatches prevent sink commit.
    fn finish(&mut self) -> Result<[u8; 32], ArtifactError>;
}

/// A finite raw response opened by a trusted credential-free transport.
/// Construction validates normalized response headers, not network provenance.
/// The adapter must reject duplicate/framing-conflicting headers, transformations,
/// automatic redirects/retries, credentials, and a source from another request.
pub struct OpenedArtifact<S> {
    source: S,
    length: Option<u64>,
}
impl<S> OpenedArtifact<S> {
    /// Accept only 200 and identity HTTP content coding. The `.crate` archive
    /// itself remains compressed; no archive extraction is performed here.
    pub fn new(
        source: S,
        status: StatusCode,
        length: Option<u64>,
        content_encoding: Option<&str>,
    ) -> Result<Self, ArtifactError> {
        if status != StatusCode::OK || content_encoding.is_some_and(|v| v != "identity") {
            return Err(ArtifactError::Headers);
        }
        Ok(Self { source, length })
    }
}
impl<S> fmt::Debug for OpenedArtifact<S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("OpenedArtifact([redacted])")
    }
}
/// Trusted anonymous HTTP streaming adapter, not an authenticated client.
/// Bind immutable origin/user-agent to the same connection used by `open`.
/// Stream actual bytes into the supplied source buffers; do not preload the
/// complete archive. Enforce connection/read deadlines and TLS verification.
pub trait BlockingArtifactTransport: BoundTransport + BoundUserAgent {
    /// Live response-body source, tied to this transport borrow.
    type Source<'a>: BlockingStreamSource
    where
        Self: 'a;
    /// Open exactly one SDK-owned GET without retaining the borrowed request.
    fn open<'a>(
        &'a self,
        request: TransportRequest<'_>,
    ) -> Result<OpenedArtifact<Self::Source<'a>>, ArtifactError>;
}

/// Explicit immutable archive identity and complete per-attempt limits.
/// Uses static CDN directly; API JSON locations are not fetch capabilities.
#[derive(Clone, Copy)]
pub struct ArtifactDownload<'a> {
    name: CrateName<'a>,
    version: Version<'a>,
    expected_bytes: u64,
    sha256: [u8; 32],
    policy: StreamPolicy,
}
impl fmt::Debug for ArtifactDownload<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("ArtifactDownload([redacted])")
    }
}
impl<'a> ArtifactDownload<'a> {
    /// Length and SHA-256 must be obtained from trusted metadata. The sink must
    /// hide tentative bytes until commit and erase/rollback them on abort.
    pub fn new(
        name: CrateName<'a>,
        version: Version<'a>,
        expected_bytes: u64,
        sha256: [u8; 32],
        limits: StreamLimits,
    ) -> Result<Self, ArtifactError> {
        if expected_bytes == 0 {
            return Err(ArtifactError::Policy);
        }
        let policy = StreamPolicy::new(
            StreamKind::FiniteDownload,
            StreamFraming::Declared(expected_bytes),
            StreamSinkMode::Transactional,
            limits,
        )
        .map_err(|_| ArtifactError::Policy)?;
        Ok(Self {
            name,
            version,
            expected_bytes,
            sha256,
            policy,
        })
    }
    /// Canonical static path encoded atomically into caller storage.
    pub fn write_target(
        self,
        output: &mut [u8],
    ) -> Result<StaticDownloadTarget<'_>, ArtifactError> {
        let len = encode_snapshot_bounded(
            self,
            output,
            crate::query::MAX_TARGET_BYTES,
            ArtifactError::Target,
            |s, e: &mut SnapshotEncoder<'_, ArtifactError>| {
                e.string("/crates/")?;
                e.percent_encoded(s.name.as_str())?;
                e.byte(b'/')?;
                e.percent_encoded(s.name.as_str())?;
                e.byte(b'-')?;
                e.percent_encoded(s.version.as_str())?;
                e.string(".crate")
            },
        )?;
        StaticDownloadTarget::new(
            core::str::from_utf8(output.get(..len).ok_or(ArtifactError::Target)?)
                .map_err(|_| ArtifactError::Target)?,
        )
        .map_err(|_| ArtifactError::Target)
    }
    /// Stream one credential-free static GET with bounded caller scratch. Any
    /// error aborts the transactional sink; only length and checksum success
    /// commits. No redirects, retry, sleep, API crawling or decompression.
    pub fn execute<T: BlockingArtifactTransport, D: BlockingStreamSink, C: ArtifactChecksum>(
        self,
        transport: &T,
        identity: IdentifyingUserAgent<'_>,
        sink: &mut D,
        checksum: C,
        scratch: &mut [u8],
    ) -> Result<StreamCompletion, ArtifactError> {
        let mut sink = CheckedSink {
            sink,
            checksum,
            expected: self.sha256,
            armed: true,
        };
        cloud_sdk::buffer::sanitize_bytes(scratch);
        let scratch = Scratch(scratch);
        if scratch.0.is_empty() {
            return Err(ArtifactError::Policy);
        }
        OfficialCratesIoEndpoint::static_downloads()
            .verify_transport(transport)
            .map_err(|_| ArtifactError::Binding)?;
        if transport.configured_user_agent() != identity.as_str().as_bytes() {
            return Err(ArtifactError::Binding);
        }
        let mut target = [0; crate::query::MAX_TARGET_BYTES];
        let target = self.write_target(&mut target)?;
        let mut opened = transport.open(TransportRequest::new(
            Method::Get,
            target.as_request_target(),
        ))?;
        if opened.length.is_some_and(|len| len != self.expected_bytes) {
            return Err(ArtifactError::Headers);
        }
        drive_blocking_stream(
            self.policy,
            &mut opened.source,
            &mut sink,
            scratch.0,
            &mut StreamOutcome::new(),
        )
        .map_err(stream_error)
    }
}
struct Scratch<'a>(&'a mut [u8]);
fn stream_error<S>(
    error: cloud_sdk::transport::StreamExecutionError<S, ArtifactError>,
) -> ArtifactError {
    match error {
        cloud_sdk::transport::StreamExecutionError::Sink(error) => error,
        _ => ArtifactError::Stream,
    }
}
impl Drop for Scratch<'_> {
    fn drop(&mut self) {
        cloud_sdk::buffer::sanitize_bytes(self.0);
    }
}
struct CheckedSink<'a, D: BlockingStreamSink, C> {
    sink: &'a mut D,
    checksum: C,
    expected: [u8; 32],
    armed: bool,
}
impl<D: BlockingStreamSink, C: ArtifactChecksum> BlockingStreamSink for CheckedSink<'_, D, C> {
    type Error = ArtifactError;
    fn write_chunk(&mut self, input: &[u8]) -> Result<usize, Self::Error> {
        let len = self
            .sink
            .write_chunk(input)
            .map_err(|_| ArtifactError::Stream)?;
        self.checksum
            .update(input.get(..len).ok_or(ArtifactError::Stream)?)?;
        Ok(len)
    }
    fn commit(&mut self) -> Result<(), Self::Error> {
        if self.checksum.finish()? != self.expected {
            return Err(ArtifactError::Checksum);
        }
        self.sink.commit().map_err(|_| ArtifactError::Stream)?;
        self.armed = false;
        Ok(())
    }
    fn abort(&mut self, partial: StreamPartialState) {
        if self.armed {
            self.sink.abort(partial);
            self.armed = false;
        }
    }
}
impl<D: BlockingStreamSink, C> Drop for CheckedSink<'_, D, C> {
    fn drop(&mut self) {
        if self.armed {
            self.sink.abort(StreamPartialState::RollbackRequired);
        }
    }
}
