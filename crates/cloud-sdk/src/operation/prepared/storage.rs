use core::fmt;

use super::PreparedRequest;

/// Caller-owned target and request-body storage supplied to preparation.
pub struct PreparationStorage<'storage> {
    target: &'storage mut [u8],
    body: &'storage mut [u8],
}

impl<'storage> PreparationStorage<'storage> {
    /// Creates complete caller-owned storage for one preparation attempt.
    ///
    /// # Security
    ///
    /// Preparation may write credentials or other secrets into `body`. A
    /// successful [`PreparedRequest`] must retain those bytes until transport
    /// use, so this wrapper cannot clear them on success. For secret-bearing
    /// operations, guard `body` with a volatile-clearing type such as
    /// `cloud_sdk_sanitization::SecretBuffer` and drop the guard immediately
    /// after transport use. A plain mutable slice is not cleared when the
    /// prepared request is dropped.
    #[must_use]
    pub const fn new(target: &'storage mut [u8], body: &'storage mut [u8]) -> Self {
        Self { target, body }
    }

    /// Consumes the storage wrapper and returns both independent buffers.
    #[must_use]
    pub fn into_parts(self) -> (&'storage mut [u8], &'storage mut [u8]) {
        (self.target, self.body)
    }
}

impl fmt::Debug for PreparationStorage<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparationStorage")
            .field("target_capacity", &self.target.len())
            .field("body_capacity", &self.body.len())
            .finish()
    }
}

/// Typed provider operation that can prepare one complete request.
///
/// ```compile_fail
/// use cloud_sdk::operation::PrepareOperation;
///
/// fn prepare_without_storage<O: PrepareOperation>(operation: &O) {
///     let _ = operation.prepare();
/// }
/// ```
pub trait PrepareOperation {
    /// Preparation-specific failure.
    type Error;

    /// Writes into caller storage and returns an executable prepared request.
    fn prepare<'storage>(
        &self,
        storage: PreparationStorage<'storage>,
    ) -> Result<PreparedRequest<'storage>, Self::Error>;
}
