//! Provider-generic typed client execution over caller-owned workspaces.

mod error;
mod execution;
mod profile;
mod response;
mod workspace;

pub use error::ClientExecutionError;
#[cfg(feature = "alloc")]
pub use profile::OwnedClientWorkspace;
pub use profile::{ClientCapacityError, ClientCapacityProfile};
pub use response::{CheckedDecodeError, ClientResponse, ClientResponseKind};
pub use workspace::{
    ClientWorkspace, ClientWorkspaceLease, ClientWorkspacePool, MAX_CLIENT_WORKSPACE_LEASES,
    WorkspaceAcquireError, WorkspacePoolError,
};

use core::fmt;

use crate::operation::PrepareOperation;

type ClientResult<R, P, T, D> = Result<R, ClientExecutionError<P, T, D>>;

/// Typed provider operation executable by [`ClientKernel`].
///
/// Preparation binds method, target, authentication, endpoint, response, and
/// safety policy. Decoding must consume [`ClientResponse`] through its checked
/// success or error path and return an owned value.
pub trait ClientOperation: PrepareOperation {
    /// Owned operation result.
    type Output;
    /// Provider-specific checked-decoding failure.
    type DecodeError;

    /// Decodes one bounded, authenticated, send-once response.
    fn decode_response(
        &self,
        response: ClientResponse<'_, '_>,
    ) -> Result<Self::Output, Self::DecodeError>;
}

/// Reusable provider-neutral client policy kernel.
///
/// The kernel owns only its transport. It owns no executor, queue, clock,
/// retry policy, or request storage. Every in-flight request must consume a
/// caller-owned [`ClientWorkspaceLease`].
pub struct ClientKernel<T> {
    transport: T,
}

impl<T> ClientKernel<T> {
    /// Creates a kernel around one endpoint-bound authenticated transport.
    #[must_use]
    pub const fn new(transport: T) -> Self {
        Self { transport }
    }

    /// Returns the underlying transport.
    #[must_use]
    pub const fn transport(&self) -> &T {
        &self.transport
    }

    /// Consumes the kernel and returns its transport.
    #[must_use]
    pub fn into_transport(self) -> T {
        self.transport
    }
}

impl<T> fmt::Debug for ClientKernel<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientKernel")
            .field("transport", &"[bound]")
            .finish()
    }
}

#[cfg(test)]
mod tests;
