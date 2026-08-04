use core::fmt;
use core::sync::atomic::{AtomicUsize, Ordering};

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Maximum number of atomic workspace leases in one pool.
pub const MAX_CLIENT_WORKSPACE_LEASES: usize = usize::BITS as usize;

/// Invalid compile-time workspace-pool capacity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspacePoolError {
    /// A pool must admit at least one workspace.
    ZeroCapacity,
    /// The requested capacity exceeds the atomic lease bitmap.
    CapacityTooLarge,
}

impl_static_error!(WorkspacePoolError,
    Self::ZeroCapacity => "client workspace pool capacity is zero",
    Self::CapacityTooLarge => "client workspace pool capacity exceeds the atomic bound",
);

/// Failure to admit one caller-owned workspace immediately.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorkspaceAcquireError {
    /// Every bounded pool slot is currently leased.
    Exhausted,
}

impl_static_error!(WorkspaceAcquireError,
    Self::Exhausted => "client workspace pool is exhausted",
);

/// Four independent caller-owned buffers for one complete client request.
///
/// The constructor borrows every region mutably, so aliasing request and
/// response storage is rejected by safe Rust:
///
/// ```compile_fail
/// use cloud_sdk::client::ClientWorkspace;
///
/// let mut shared = [0_u8; 128];
/// let mut body = [0_u8; 128];
/// let mut headers = [0_u8; 128];
/// let _ = ClientWorkspace::new(&mut shared, &mut body, &mut shared, &mut headers);
/// ```
pub struct ClientWorkspace<'storage> {
    target: SecretBuffer<'storage>,
    request_body: SecretBuffer<'storage>,
    response_body: SecretBuffer<'storage>,
    response_headers: SecretBuffer<'storage>,
}

impl<'storage> ClientWorkspace<'storage> {
    /// Admits and immediately clears four independent storage regions.
    #[must_use]
    pub fn new(
        target: &'storage mut [u8],
        request_body: &'storage mut [u8],
        response_body: &'storage mut [u8],
        response_headers: &'storage mut [u8],
    ) -> Self {
        sanitize_bytes(target);
        sanitize_bytes(request_body);
        sanitize_bytes(response_body);
        sanitize_bytes(response_headers);
        Self {
            target: SecretBuffer::new(target),
            request_body: SecretBuffer::new(request_body),
            response_body: SecretBuffer::new(response_body),
            response_headers: SecretBuffer::new(response_headers),
        }
    }

    /// Returns capacities without exposing stored bytes.
    #[must_use]
    pub fn capacities(&self) -> (usize, usize, usize, usize) {
        (
            self.target.as_slice().len(),
            self.request_body.as_slice().len(),
            self.response_body.as_slice().len(),
            self.response_headers.as_slice().len(),
        )
    }

    pub(crate) fn parts_mut(&mut self) -> ClientWorkspaceParts<'_> {
        ClientWorkspaceParts {
            target: self.target.as_mut_slice(),
            request_body: self.request_body.as_mut_slice(),
            response_body: self.response_body.as_mut_slice(),
            response_headers: self.response_headers.as_mut_slice(),
        }
    }
}

impl fmt::Debug for ClientWorkspace<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientWorkspace")
            .field("capacities", &self.capacities())
            .finish_non_exhaustive()
    }
}

pub(crate) struct ClientWorkspaceParts<'storage> {
    pub(crate) target: &'storage mut [u8],
    pub(crate) request_body: &'storage mut [u8],
    pub(crate) response_body: &'storage mut [u8],
    pub(crate) response_headers: &'storage mut [u8],
}

impl ClientWorkspaceParts<'_> {
    pub(crate) fn clear(&mut self) {
        sanitize_bytes(self.target);
        sanitize_bytes(self.request_body);
        sanitize_bytes(self.response_body);
        sanitize_bytes(self.response_headers);
    }
}

/// Fixed-capacity atomic admission for caller-supplied workspaces.
///
/// The pool stores no buffers and has no wait queue. Exhaustion is immediate.
pub struct ClientWorkspacePool<const N: usize> {
    leased: AtomicUsize,
}

impl<const N: usize> ClientWorkspacePool<N> {
    /// Creates a pool when `N` fits the platform atomic bitmap.
    pub const fn new() -> Result<Self, WorkspacePoolError> {
        if N == 0 {
            return Err(WorkspacePoolError::ZeroCapacity);
        }
        if N > MAX_CLIENT_WORKSPACE_LEASES {
            return Err(WorkspacePoolError::CapacityTooLarge);
        }
        Ok(Self {
            leased: AtomicUsize::new(0),
        })
    }

    /// Admits one workspace immediately or returns exhaustion without queuing.
    pub fn try_acquire<'pool, 'storage>(
        &'pool self,
        workspace: ClientWorkspace<'storage>,
    ) -> Result<ClientWorkspaceLease<'pool, 'storage, N>, WorkspaceAcquireError> {
        let valid = valid_mask::<N>();
        let mut observed = self.leased.load(Ordering::Acquire);
        loop {
            let available = !observed & valid;
            if available == 0 {
                return Err(WorkspaceAcquireError::Exhausted);
            }
            let index = available.trailing_zeros() as usize;
            let bit = 1_usize << index;
            match self.leased.compare_exchange_weak(
                observed,
                observed | bit,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    return Ok(ClientWorkspaceLease {
                        workspace,
                        _slot: LeaseSlot { pool: self, bit },
                    });
                }
                Err(current) => observed = current,
            }
        }
    }

    /// Returns the number of currently admitted workspaces.
    #[must_use]
    pub fn active_leases(&self) -> usize {
        (self.leased.load(Ordering::Acquire) & valid_mask::<N>()).count_ones() as usize
    }
}

impl<const N: usize> fmt::Debug for ClientWorkspacePool<N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientWorkspacePool")
            .field("capacity", &N)
            .field("active_leases", &self.active_leases())
            .finish()
    }
}

/// One admitted workspace whose buffers remain uniquely borrowed until drop.
///
/// The lease owns all mutable borrows when moved into an async execution
/// future, so caller code cannot touch any buffer while that request is live.
///
/// ```compile_fail
/// use cloud_sdk::client::{ClientWorkspace, ClientWorkspaceLease, ClientWorkspacePool};
///
/// async fn hold<const N: usize>(_: ClientWorkspaceLease<'_, '_, N>) {}
///
/// let pool = ClientWorkspacePool::<1>::new().unwrap();
/// let mut target = [0_u8; 16];
/// let mut request = [0_u8; 16];
/// let mut response = [0_u8; 16];
/// let mut headers = [0_u8; 16];
/// let workspace = ClientWorkspace::new(
///     &mut target,
///     &mut request,
///     &mut response,
///     &mut headers,
/// );
/// let lease = pool.try_acquire(workspace).unwrap();
/// let future = hold(lease);
/// response.fill(0xa5);
/// drop(future);
/// ```
pub struct ClientWorkspaceLease<'pool, 'storage, const N: usize> {
    workspace: ClientWorkspace<'storage>,
    _slot: LeaseSlot<'pool, N>,
}

impl<const N: usize> ClientWorkspaceLease<'_, '_, N> {
    /// Returns all workspace capacities without exposing stored bytes.
    #[must_use]
    pub fn capacities(&self) -> (usize, usize, usize, usize) {
        self.workspace.capacities()
    }

    pub(crate) fn parts_mut(&mut self) -> ClientWorkspaceParts<'_> {
        self.workspace.parts_mut()
    }
}

impl<const N: usize> fmt::Debug for ClientWorkspaceLease<'_, '_, N> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ClientWorkspaceLease")
            .field("capacities", &self.capacities())
            .field("slot", &"[leased]")
            .finish()
    }
}

struct LeaseSlot<'pool, const N: usize> {
    pool: &'pool ClientWorkspacePool<N>,
    bit: usize,
}

impl<const N: usize> Drop for LeaseSlot<'_, N> {
    fn drop(&mut self) {
        self.pool.leased.fetch_and(!self.bit, Ordering::Release);
    }
}

fn valid_mask<const N: usize>() -> usize {
    let shift = match u32::try_from(N) {
        Ok(value) => value,
        Err(_) => return usize::MAX,
    };
    match 1_usize.checked_shl(shift) {
        Some(upper_bit) => upper_bit.saturating_sub(1),
        None => usize::MAX,
    }
}
