//! Trusted adapter contracts for one finite authenticated streaming request.
use super::{
    AsyncAuthorizedRawHttpExecutor, AsyncStreamSource, BlockingAuthorizedRawHttpExecutor,
    BlockingStreamSource, EndpointIdentity, HeaderValue, LocalAsyncStreamSource,
    LocalAuthorizedRawHttpExecutor, RawResponsePolicy, ResponseWriter, StreamPolicy,
    TransportRequest,
};
use core::future::Future;

/// Borrowed upload inputs. Source and authorization storage remain caller-owned.
/// Adapters must clear scratch on every exit, including unpolled cancellation.
pub struct AuthorizedUpload<'a, S> {
    /// Expected bound destination; compare before credentials or source reads.
    pub expected: EndpointIdentity<'a>,
    /// Complete sensitive header value, without an adapter-added prefix.
    pub authorization: HeaderValue<'a>,
    /// One-shot source. Never seek or replay implicitly.
    pub source: &'a mut S,
    /// Declared finite upload framing and bounded progress policy.
    pub policy: StreamPolicy,
    /// Bounded transfer scratch; never whole-body buffering.
    pub scratch: &'a mut [u8],
}
impl<S> core::fmt::Debug for AuthorizedUpload<'_, S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("AuthorizedUpload([redacted])")
    }
}

/// Trusted blocking upload adapter. Enforce destination binding, explicit
/// authorization, empty request-body storage, finite declared framing, progress
/// limits and response policy. No redirects, cookies, retries or buffering of
/// the complete upload. Commit the response only after source EOF and complete
/// response validation. A failure may follow a committed remote mutation.
/// Synchronous source code must cooperate with the caller's deadline policy.
pub trait BlockingRawUploadExecutor: BlockingAuthorizedRawHttpExecutor {
    /// Executes exactly one upload. Clear scratch and tentative response on
    /// every error; preserve delivery classification in the adapter's error.
    fn upload<S: BlockingStreamSource>(
        &self,
        request: TransportRequest<'_>,
        policy: RawResponsePolicy<'_>,
        upload: AuthorizedUpload<'_, S>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>;
}

/// Send upload counterpart of [`BlockingRawUploadExecutor`], with identical
/// wire and commitment requirements. No blocking bridge is permitted. Guards
/// must be installed before returning the future; cancellation cannot authorize
/// replay or establish whether the upstream mutation completed.
pub trait AsyncRawUploadExecutor: AsyncAuthorizedRawHttpExecutor + Sync {
    /// Executes once with a bounded live source and clears scratch on drop.
    fn upload<'a, 'b: 'a, S: AsyncStreamSource + Send + 'a>(
        &'a self,
        request: TransportRequest<'a>,
        policy: RawResponsePolicy<'a>,
        upload: AuthorizedUpload<'a, S>,
        response: &'a mut ResponseWriter<'b>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'a;
}

/// Executor-local upload counterpart with the same security contract and no
/// Send requirement on the source or future. Implementations remain trusted.
pub trait LocalRawUploadExecutor: LocalAuthorizedRawHttpExecutor {
    /// Executes once; arm cleanup before returning, even when never polled.
    fn upload_local<'a, 'b: 'a, S: LocalAsyncStreamSource + 'a>(
        &'a self,
        request: TransportRequest<'a>,
        policy: RawResponsePolicy<'a>,
        upload: AuthorizedUpload<'a, S>,
        response: &'a mut ResponseWriter<'b>,
    ) -> impl Future<Output = Result<(), Self::Error>> + 'a;
}
