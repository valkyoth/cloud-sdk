//! Request preparation capacity profiles and cleanup guards.

use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

use super::{PreparationStorage, PrepareOperation, PreparedRequest};
use crate::transport::MAX_REQUEST_TARGET_BYTES;

/// Embedded request-body capacity in bytes.
pub const EMBEDDED_BODY_BYTES: usize = 16 * 1024;
/// Default request-body capacity in bytes.
pub const DEFAULT_BODY_BYTES: usize = 1024 * 1024;
/// Large request-body capacity in bytes.
pub const LARGE_BODY_BYTES: usize = 8 * 1024 * 1024;

/// Named, bounded storage capacities for request preparation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PreparationCapacityProfile {
    target_bytes: usize,
    body_bytes: usize,
}

impl PreparationCapacityProfile {
    /// Small profile for constrained devices and ordinary JSON mutations.
    pub const EMBEDDED: Self = Self::new(1024, EMBEDDED_BODY_BYTES);
    /// General profile supporting the complete request-target limit.
    pub const DEFAULT: Self = Self::new(MAX_REQUEST_TARGET_BYTES, DEFAULT_BODY_BYTES);
    /// Large profile for explicitly admitted bulk request bodies.
    pub const LARGE: Self = Self::new(MAX_REQUEST_TARGET_BYTES, LARGE_BODY_BYTES);

    const fn new(target_bytes: usize, body_bytes: usize) -> Self {
        Self {
            target_bytes,
            body_bytes,
        }
    }

    /// Returns the required request-target capacity.
    #[must_use]
    pub const fn target_bytes(self) -> usize {
        self.target_bytes
    }

    /// Returns the required request-body capacity.
    #[must_use]
    pub const fn body_bytes(self) -> usize {
        self.body_bytes
    }

    /// Checks whether two buffers satisfy this profile.
    pub const fn validate(
        self,
        target_bytes: usize,
        body_bytes: usize,
    ) -> Result<(), PreparationCapacityError> {
        if target_bytes < self.target_bytes {
            return Err(PreparationCapacityError::TargetTooSmall);
        }
        if body_bytes < self.body_bytes {
            return Err(PreparationCapacityError::BodyTooSmall);
        }
        Ok(())
    }
}

/// Failure while selecting or allocating preparation storage.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PreparationCapacityError {
    /// Request-target storage does not satisfy the selected profile.
    TargetTooSmall,
    /// Request-body storage does not satisfy the selected profile.
    BodyTooSmall,
    /// The allocator rejected the requested bounded profile.
    AllocationFailed,
}

impl_static_error!(PreparationCapacityError,
    Self::TargetTooSmall => "preparation target storage is too small",
    Self::BodyTooSmall => "preparation body storage is too small",
    Self::AllocationFailed => "preparation storage allocation failed",
);

/// Caller-owned preparation buffers that are cleared together on drop.
///
/// A prepared request borrows this guard through [`Self::prepare`], so safe
/// Rust prevents the guard from being dropped before transport use completes.
/// Each preparation attempt first volatile-clears both complete borrowed
/// buffers, including residue from a previous request. Dropping the guard
/// performs the same complete cleanup.
pub struct PreparationStorageGuard<'storage> {
    target: SecretBuffer<'storage>,
    body: SecretBuffer<'storage>,
}

impl<'storage> PreparationStorageGuard<'storage> {
    /// Guards two independent caller-owned buffers.
    #[must_use]
    pub const fn new(target: &'storage mut [u8], body: &'storage mut [u8]) -> Self {
        Self {
            target: SecretBuffer::new(target),
            body: SecretBuffer::new(body),
        }
    }

    /// Guards buffers after validating one named capacity profile.
    pub fn for_profile(
        target: &'storage mut [u8],
        body: &'storage mut [u8],
        profile: PreparationCapacityProfile,
    ) -> Result<Self, PreparationCapacityError> {
        let guard = Self::new(target, body);
        let (target_bytes, body_bytes) = guard.capacities();
        profile.validate(target_bytes, body_bytes)?;
        Ok(guard)
    }

    /// Prepares one operation while retaining cleanup ownership.
    ///
    /// Both complete buffers are volatile-cleared before each attempt. Reusing
    /// one guard therefore does not retain bytes from an earlier request in an
    /// unused tail.
    pub fn prepare<'guard, O>(
        &'guard mut self,
        operation: &O,
    ) -> Result<PreparedRequest<'guard>, O::Error>
    where
        O: PrepareOperation,
    {
        sanitize_bytes(self.target.as_mut_slice());
        sanitize_bytes(self.body.as_mut_slice());
        operation.prepare(PreparationStorage::new(
            self.target.as_mut_slice(),
            self.body.as_mut_slice(),
        ))
    }

    /// Runs provider-specific preparation while retaining cleanup ownership.
    ///
    /// This is the typed counterpart to [`Self::prepare`]. The closure may
    /// return a provider-specific prepared wrapper that borrows this guard.
    /// Both complete buffers are cleared before the closure is entered.
    pub fn prepare_with<'guard, T, E, F>(&'guard mut self, prepare: F) -> Result<T, E>
    where
        F: FnOnce(PreparationStorage<'guard>) -> Result<T, E>,
    {
        sanitize_bytes(self.target.as_mut_slice());
        sanitize_bytes(self.body.as_mut_slice());
        prepare(PreparationStorage::new(
            self.target.as_mut_slice(),
            self.body.as_mut_slice(),
        ))
    }

    /// Returns capacities without exposing stored request bytes.
    #[must_use]
    pub fn capacities(&self) -> (usize, usize) {
        (self.target.as_slice().len(), self.body.as_slice().len())
    }
}

impl fmt::Debug for PreparationStorageGuard<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (target_capacity, body_capacity) = self.capacities();
        formatter
            .debug_struct("PreparationStorageGuard")
            .field("target_capacity", &target_capacity)
            .field("body_capacity", &body_capacity)
            .finish_non_exhaustive()
    }
}

/// Fallibly allocated preparation buffers cleared in full on drop.
#[cfg(feature = "alloc")]
pub struct OwnedPreparationStorage {
    target: alloc::boxed::Box<[u8]>,
    body: alloc::boxed::Box<[u8]>,
}

#[cfg(feature = "alloc")]
impl OwnedPreparationStorage {
    /// Allocates exactly one named profile without panicking on allocation
    /// failure.
    pub fn try_for_profile(
        profile: PreparationCapacityProfile,
    ) -> Result<Self, PreparationCapacityError> {
        let target = allocate_zeroed(profile.target_bytes)?;
        let body = allocate_zeroed(profile.body_bytes)?;
        Ok(Self { target, body })
    }

    /// Borrows both owned buffers behind a cleanup guard.
    pub fn guard(&mut self) -> PreparationStorageGuard<'_> {
        PreparationStorageGuard::new(&mut self.target, &mut self.body)
    }

    /// Returns capacities without exposing stored request bytes.
    #[must_use]
    pub fn capacities(&self) -> (usize, usize) {
        (self.target.len(), self.body.len())
    }
}

#[cfg(feature = "alloc")]
impl Drop for OwnedPreparationStorage {
    fn drop(&mut self) {
        cloud_sdk_sanitization::sanitize_bytes(&mut self.target);
        cloud_sdk_sanitization::sanitize_bytes(&mut self.body);
    }
}

#[cfg(feature = "alloc")]
impl fmt::Debug for OwnedPreparationStorage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (target_capacity, body_capacity) = self.capacities();
        formatter
            .debug_struct("OwnedPreparationStorage")
            .field("target_capacity", &target_capacity)
            .field("body_capacity", &body_capacity)
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "alloc")]
fn allocate_zeroed(len: usize) -> Result<alloc::boxed::Box<[u8]>, PreparationCapacityError> {
    let mut bytes = alloc::vec::Vec::new();
    bytes
        .try_reserve_exact(len)
        .map_err(|_| PreparationCapacityError::AllocationFailed)?;
    bytes.resize(len, 0);
    Ok(bytes.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use crate::operation::{PreparationStorage, PrepareOperation, PreparedRequest};

    use super::{PreparationCapacityError, PreparationCapacityProfile, PreparationStorageGuard};

    struct ContaminateStorage;

    impl PrepareOperation for ContaminateStorage {
        type Error = ();

        fn prepare<'storage>(
            &self,
            storage: PreparationStorage<'storage>,
        ) -> Result<PreparedRequest<'storage>, Self::Error> {
            let (target, body) = storage.into_parts();
            target.fill(0xA5);
            body.fill(0x5A);
            Err(())
        }
    }

    struct AssertCleared;

    impl PrepareOperation for AssertCleared {
        type Error = ();

        fn prepare<'storage>(
            &self,
            storage: PreparationStorage<'storage>,
        ) -> Result<PreparedRequest<'storage>, Self::Error> {
            let (target, body) = storage.into_parts();
            assert!(target.iter().all(|byte| *byte == 0));
            assert!(body.iter().all(|byte| *byte == 0));
            Err(())
        }
    }

    #[test]
    fn profiles_are_bounded_and_validate_both_regions() {
        assert_eq!(
            PreparationCapacityProfile::DEFAULT.target_bytes(),
            crate::transport::MAX_REQUEST_TARGET_BYTES
        );
        assert_eq!(
            PreparationCapacityProfile::DEFAULT.validate(8191, usize::MAX),
            Err(PreparationCapacityError::TargetTooSmall)
        );
        assert_eq!(
            PreparationCapacityProfile::DEFAULT.validate(8192, 1024),
            Err(PreparationCapacityError::BodyTooSmall)
        );
    }

    #[test]
    fn borrowed_guard_clears_both_complete_buffers() {
        let mut target = [0xA5_u8; 8];
        let mut body = [0x5A_u8; 16];
        {
            let guard = PreparationStorageGuard::new(&mut target, &mut body);
            assert_eq!(guard.capacities(), (8, 16));
        }
        assert_eq!(target, [0; 8]);
        assert_eq!(body, [0; 16]);
    }

    #[test]
    fn every_preparation_attempt_clears_complete_reused_storage_first() {
        let mut target = [0_u8; 8];
        let mut body = [0_u8; 16];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut body);

        assert!(matches!(guard.prepare(&ContaminateStorage), Err(())));
        assert!(matches!(guard.prepare(&AssertCleared), Err(())));
    }

    #[test]
    fn provider_specific_preparation_retains_cleanup_ownership() {
        let mut target = [0xA5_u8; 8];
        let mut body = [0x5A_u8; 16];
        {
            let mut guard = PreparationStorageGuard::new(&mut target, &mut body);
            let result: Result<(), ()> = guard.prepare_with(|storage| {
                let (target, body) = storage.into_parts();
                assert!(target.iter().all(|byte| *byte == 0));
                assert!(body.iter().all(|byte| *byte == 0));
                target.fill(0x11);
                body.fill(0x22);
                Err(())
            });
            assert_eq!(result, Err(()));
        }
        assert_eq!(target, [0; 8]);
        assert_eq!(body, [0; 16]);
    }

    #[test]
    fn profile_validation_failures_clear_both_complete_buffers() {
        let mut short_target = [0xA5_u8; 8];
        let mut target_failure_body = [0x5A_u8; 16];
        assert!(matches!(
            PreparationStorageGuard::for_profile(
                &mut short_target,
                &mut target_failure_body,
                PreparationCapacityProfile::DEFAULT,
            ),
            Err(PreparationCapacityError::TargetTooSmall)
        ));
        assert_eq!(short_target, [0; 8]);
        assert_eq!(target_failure_body, [0; 16]);

        let mut target = [0xA5_u8; crate::transport::MAX_REQUEST_TARGET_BYTES];
        let mut short_body = [0x5A_u8; 16];
        assert!(matches!(
            PreparationStorageGuard::for_profile(
                &mut target,
                &mut short_body,
                PreparationCapacityProfile::DEFAULT,
            ),
            Err(PreparationCapacityError::BodyTooSmall)
        ));
        assert!(target.iter().all(|byte| *byte == 0));
        assert_eq!(short_body, [0; 16]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn owned_profiles_allocate_exact_bounded_regions() {
        let storage =
            super::OwnedPreparationStorage::try_for_profile(PreparationCapacityProfile::EMBEDDED);
        assert!(storage.is_ok());
        assert_eq!(
            storage.map(|storage| storage.capacities()),
            Ok((1024, super::EMBEDDED_BODY_BYTES))
        );
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn impossible_allocation_fails_without_panicking() {
        assert_eq!(
            super::allocate_zeroed(usize::MAX),
            Err(PreparationCapacityError::AllocationFailed)
        );
    }
}
