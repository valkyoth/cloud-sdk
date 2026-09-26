use super::*;
use cloud_sdk::transport::AsyncStreamSource;
use cloud_sdk::transport::LocalAsyncStreamSource;

impl AsyncStreamSource for SlicePackage<'_> {
    type Error = Error;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, Error> {
        BlockingStreamSource::read_chunk(self, output)
    }
}

impl<S: AsyncStreamSource + Send> AsyncStreamSource for Framed<'_, S> {
    type Error = Error;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, Error> {
        if let Some(read) = self.prefix(output)? {
            return Ok(read);
        }
        let read = self
            .source
            .read_chunk(output)
            .await
            .map_err(|_| Error::Value)?;
        self.finish_read(read, output.len())
    }
}

// A distinct wrapper avoids overlapping the blanket Send-to-local source impl.
pub(crate) struct LocalFramed<'a, S>(pub(super) Framed<'a, S>);
impl<'a, S> LocalFramed<'a, S> {
    pub(crate) fn new(request: &'a PublishRequest<'_>, source: &'a mut S) -> Result<Self, Error> {
        Framed::new(request, source).map(Self)
    }
}
impl<S: LocalAsyncStreamSource> LocalAsyncStreamSource for LocalFramed<'_, S> {
    type Error = Error;
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk_local<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, Error> {
        if let Some(read) = self.0.prefix(output)? {
            return Ok(read);
        }
        let read = self
            .0
            .source
            .read_chunk_local(output)
            .await
            .map_err(|_| Error::Value)?;
        self.0.finish_read(read, output.len())
    }
}
