use super::{ResponseMetadata, ResponseWriter, ResponseWriterError};
use crate::transport::{ResponseHeaders, StatusCode};

/// Completion metadata returned by an asynchronous transport.
///
/// This value does not commit a response. The SDK validates and commits it
/// only after the transport future returns successfully.
pub struct ResponseCompletion {
    status: StatusCode,
    initialized_len: usize,
    metadata: ResponseMetadata,
}

impl ResponseCompletion {
    /// Describes one completely initialized response.
    #[must_use]
    pub const fn new(
        status: StatusCode,
        initialized_len: usize,
        metadata: ResponseMetadata,
    ) -> Self {
        Self {
            status,
            initialized_len,
            metadata,
        }
    }
}

/// Non-committing response view supplied to an asynchronous transport.
///
/// Implementations may initialize body and header storage, but this type
/// deliberately exposes no commit operation. Cancellation drops the
/// SDK-owned outer attempt and clears all partial state.
///
/// ```compile_fail
/// use cloud_sdk::transport::{AsyncResponseStaging, ResponseMetadata, StatusCode};
/// fn cannot_commit(mut staging: AsyncResponseStaging<'_, '_>) {
///     staging.commit(StatusCode::OK, 0, ResponseMetadata::EMPTY);
/// }
/// ```
pub struct AsyncResponseStaging<'staging, 'buffer> {
    writer: &'staging mut ResponseWriter<'buffer>,
}

impl<'buffer> AsyncResponseStaging<'_, 'buffer> {
    /// Returns the admitted response-body capacity.
    #[must_use]
    pub const fn body_capacity(&self) -> usize {
        self.writer.body_capacity()
    }

    /// Returns exclusive access to the admitted response-body prefix.
    pub fn body_mut(&mut self) -> Result<&mut [u8], ResponseWriterError> {
        self.writer.body_mut()
    }

    /// Returns mutable caller-owned response-header storage.
    pub fn headers_mut(&mut self) -> Result<&mut ResponseHeaders<'buffer>, ResponseWriterError> {
        self.writer.headers_mut()
    }

    /// Returns response headers captured so far.
    #[must_use]
    pub const fn headers(&self) -> &ResponseHeaders<'buffer> {
        self.writer.headers()
    }
}

/// Cleanup-owning transaction around one response write attempt.
pub struct ResponseAttempt<'writer, 'buffer> {
    pub(super) writer: &'writer mut ResponseWriter<'buffer>,
    pub(super) completed: bool,
}

impl<'buffer> ResponseAttempt<'_, 'buffer> {
    /// Returns the admitted response-body capacity.
    #[must_use]
    pub const fn body_capacity(&self) -> usize {
        self.writer.body_capacity()
    }

    /// Returns exclusive access to the admitted response-body prefix.
    pub fn body_mut(&mut self) -> Result<&mut [u8], ResponseWriterError> {
        self.writer.body_mut()
    }

    /// Returns mutable caller-owned response-header storage.
    pub fn headers_mut(&mut self) -> Result<&mut ResponseHeaders<'buffer>, ResponseWriterError> {
        self.writer.headers_mut()
    }

    /// Returns response headers captured by this attempt.
    #[must_use]
    pub const fn headers(&self) -> &ResponseHeaders<'buffer> {
        self.writer.headers()
    }

    /// Commits this synchronous attempt exactly once.
    pub fn commit(
        &mut self,
        status: StatusCode,
        initialized_len: usize,
        metadata: ResponseMetadata,
    ) -> Result<(), ResponseWriterError> {
        self.writer.commit(status, initialized_len, metadata)?;
        self.completed = true;
        Ok(())
    }

    pub(crate) fn staging(&mut self) -> AsyncResponseStaging<'_, 'buffer> {
        AsyncResponseStaging {
            writer: self.writer,
        }
    }

    /// Commits completion metadata after an asynchronous stage is ready.
    pub fn commit_completion(
        &mut self,
        completion: ResponseCompletion,
    ) -> Result<(), ResponseWriterError> {
        self.commit(
            completion.status,
            completion.initialized_len,
            completion.metadata,
        )
    }
}

impl Drop for ResponseAttempt<'_, '_> {
    fn drop(&mut self) {
        if !self.completed {
            self.writer.rollback_attempt();
        }
    }
}
