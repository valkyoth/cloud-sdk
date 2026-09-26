use super::PublishError as Error;
#[cfg(any(feature = "blocking", feature = "async-rustls", test))]
use super::PublishRequest;
use cloud_sdk::transport::{
    BlockingStreamSink, BlockingStreamSource, StreamCompletion, StreamOutcome, StreamPolicy,
    StreamRead, StreamReplayability, drive_blocking_stream,
};

/// Borrowed immutable package source. Source storage remains caller-owned and
/// is not erased; no second archive allocation is made.
pub struct SlicePackage<'a> {
    bytes: &'a [u8],
}
impl<'a> SlicePackage<'a> {
    /// An already packaged compressed Cargo `.crate` artifact.
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes }
    }
}
impl BlockingStreamSource for SlicePackage<'_> {
    type Error = Error;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Error> {
        if self.bytes.is_empty() {
            return Ok(StreamRead::End);
        }
        if output.is_empty() {
            return Err(Error::Limit);
        }
        let n = self.bytes.len().min(output.len());
        output
            .get_mut(..n)
            .ok_or(Error::Limit)?
            .copy_from_slice(self.bytes.get(..n).ok_or(Error::Limit)?);
        self.bytes = self.bytes.get(n..).ok_or(Error::Limit)?;
        Ok(StreamRead::Chunk(n))
    }
}
/// One-shot framed upload lent only to a trusted transport callback. The
/// callback must stream it into the same authenticated HTTP exchange and use
/// direct, not transactional, sink semantics. Never buffer the whole archive.
pub struct PublishUpload<'a, S> {
    body: Option<Framed<'a, S>>,
    policy: StreamPolicy,
    pub(super) complete: bool,
}
impl<'a, S: BlockingStreamSource> PublishUpload<'a, S> {
    #[cfg(any(feature = "blocking", test))]
    pub(super) fn new(request: &'a PublishRequest<'_>, source: &'a mut S) -> Result<Self, Error> {
        Ok(Self {
            body: Some(Framed::new(request, source)?),
            policy: request.policy,
            complete: false,
        })
    }
    /// Exact bytes the HTTP adapter must declare, including the two length words.
    pub fn content_length(&self) -> u64 {
        match self.policy.framing() {
            cloud_sdk::transport::StreamFraming::Declared(n) => n,
            _ => 0,
        }
    }
    #[cfg(feature = "blocking-rustls")]
    pub(super) fn take_framed(&mut self) -> Result<(Framed<'a, S>, StreamPolicy), Error> {
        self.complete = false;
        Ok((self.body.take().ok_or(Error::Binding)?, self.policy))
    }
    /// Stream once with bounded progress and direct sink abort semantics. Any
    /// failure, cancellation signalled by the source/sink, or unwinding aborts
    /// the sink and clears scratch. Partial remote publication cannot roll back.
    pub fn transfer_to<D: BlockingStreamSink>(
        &mut self,
        sink: &mut D,
        scratch: &mut [u8],
    ) -> Result<StreamCompletion, Error> {
        self.complete = false;
        let mut scratch = cloud_sdk_sanitization::SecretBuffer::new(scratch);
        let Some(mut body) = self.body.take() else {
            sink.abort(cloud_sdk::transport::StreamPartialState::Clean);
            return Err(Error::Binding);
        };
        if scratch.as_slice().is_empty() {
            sink.abort(cloud_sdk::transport::StreamPartialState::Clean);
            return Err(Error::Limit);
        }
        let result = drive_blocking_stream(
            self.policy,
            &mut body,
            sink,
            scratch.as_mut_slice(),
            &mut StreamOutcome::new(),
        )
        .map_err(|_| Error::Value)?;
        self.complete = true;
        Ok(result)
    }
}
pub(super) struct Framed<'a, S> {
    metadata: &'a [u8],
    source: &'a mut S,
    metadata_length: [u8; 4],
    archive_length: [u8; 4],
    stage: u8,
    offset: usize,
    remaining: u64,
}
impl<S: BlockingStreamSource> BlockingStreamSource for Framed<'_, S> {
    type Error = Error;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, Error> {
        if let Some(read) = self.prefix(output)? {
            return Ok(read);
        }
        let read = self.source.read_chunk(output).map_err(|_| Error::Value)?;
        self.finish_read(read, output.len())
    }
}

impl<'a, S> Framed<'a, S> {
    #[cfg(any(feature = "blocking", feature = "async-rustls", test))]
    pub(super) fn new(request: &'a PublishRequest<'_>, source: &'a mut S) -> Result<Self, Error> {
        Ok(Self {
            metadata: request.metadata.bytes,
            source,
            metadata_length: u32::try_from(request.metadata.bytes.len())
                .map_err(|_| Error::Limit)?
                .to_le_bytes(),
            archive_length: u32::try_from(request.archive)
                .map_err(|_| Error::Limit)?
                .to_le_bytes(),
            stage: 0,
            offset: 0,
            remaining: request.archive,
        })
    }
    fn prefix(&mut self, output: &mut [u8]) -> Result<Option<StreamRead>, Error> {
        if output.is_empty() {
            return Err(Error::Limit);
        }
        while self.stage < 3 {
            let bytes = match self.stage {
                0 => self.metadata_length.as_slice(),
                1 => self.metadata,
                _ => self.archive_length.as_slice(),
            };
            let rest = bytes.get(self.offset..).ok_or(Error::Limit)?;
            if rest.is_empty() {
                self.stage = self.stage.checked_add(1).ok_or(Error::Limit)?;
                self.offset = 0;
                continue;
            }
            let n = rest.len().min(output.len());
            output
                .get_mut(..n)
                .ok_or(Error::Limit)?
                .copy_from_slice(rest.get(..n).ok_or(Error::Limit)?);
            self.offset = self.offset.checked_add(n).ok_or(Error::Limit)?;
            return Ok(Some(StreamRead::Chunk(n)));
        }
        Ok(None)
    }
    fn finish_read(&mut self, read: StreamRead, capacity: usize) -> Result<StreamRead, Error> {
        match read {
            StreamRead::Chunk(n) => {
                if n > capacity {
                    return Err(Error::Limit);
                }
                self.remaining = self
                    .remaining
                    .checked_sub(u64::try_from(n).map_err(|_| Error::Limit)?)
                    .ok_or(Error::Limit)?;
                Ok(StreamRead::Chunk(n))
            }
            StreamRead::End if self.remaining != 0 => Err(Error::Binding),
            other => Ok(other),
        }
    }
}

#[cfg(feature = "async")]
pub(super) mod asynchronous;
