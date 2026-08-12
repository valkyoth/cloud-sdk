use core::{cell::Cell, fmt};

use cloud_sdk::authentication::{
    AuthenticatedRequest, BoundCredentialTransport, CredentialBinding,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::transport::{
    AsyncResponseStaging, BoundTransport, EndpointIdentity, EndpointIdentityError,
    LocalAsyncTransport, ResponseCompletion, TransportRequest,
};

use super::{MockError, MockExchange, MockTransport};

/// Ordered mock transport whose futures are intentionally local-only.
///
/// The `Cell` marker makes this type `!Sync`, so futures borrowing it cannot be
/// sent between threads. It exercises browser, embedded, and single-threaded
/// executor integrations without adding an allocator or runtime dependency.
///
/// ```compile_fail
/// use cloud_sdk_testkit::LocalMockTransport;
/// fn require_sync<T: Sync>() {}
/// require_sync::<LocalMockTransport<'static>>();
/// ```
pub struct LocalMockTransport<'a> {
    inner: MockTransport<'a>,
    local_marker: Cell<()>,
}

impl<'a> LocalMockTransport<'a> {
    /// Creates a local-only mock over an ordered exchange slice.
    #[must_use]
    pub const fn new(exchanges: &'a [MockExchange<'a>]) -> Self {
        Self {
            inner: MockTransport::new(exchanges),
            local_marker: Cell::new(()),
        }
    }

    /// Binds the mock permanently to one normalized endpoint identity.
    #[must_use]
    pub const fn with_endpoint(mut self, endpoint: EndpointIdentity<'a>) -> Self {
        self.inner = self.inner.with_endpoint(endpoint);
        self
    }

    /// Selects a deterministic credential lineage for association tests.
    #[must_use]
    pub const fn with_credential_binding(mut self, binding: CredentialBinding) -> Self {
        self.inner = self.inner.with_credential_binding(binding);
        self
    }

    /// Returns the number of exchanges not yet consumed.
    #[must_use]
    pub fn remaining(&self) -> usize {
        self.inner.remaining()
    }

    /// Reports whether every expected exchange was consumed.
    #[must_use]
    pub fn is_complete(&self) -> bool {
        self.inner.is_complete()
    }
}

impl LocalAsyncTransport for LocalMockTransport<'_> {
    type Error = MockError;

    async fn send_local<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        self.local_marker.get();
        self.inner.stage_inner(request, &mut response)
    }
}

impl LocalAsyncAuthenticatedTransport for LocalMockTransport<'_> {
    type Error = MockError;

    async fn send_authenticated_local<'transport, 'request, 'policy, 'writer, 'buffer>(
        &'transport self,
        request: AuthenticatedRequest<'request, 'policy>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'policy: 'writer,
        'buffer: 'writer,
    {
        self.local_marker.get();
        self.inner
            .stage_inner(request.transport_request(), &mut response)
    }
}

impl BoundTransport for LocalMockTransport<'_> {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        self.inner.endpoint_identity()
    }
}

impl BoundCredentialTransport for LocalMockTransport<'_> {
    fn credential_binding(&self) -> CredentialBinding {
        self.inner.credential_binding()
    }
}

impl fmt::Debug for LocalMockTransport<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LocalMockTransport")
            .field("remaining", &self.remaining())
            .finish_non_exhaustive()
    }
}
