//! Named capacities for complete client request workspaces.

#[cfg(feature = "alloc")]
use core::fmt;

use super::ClientWorkspace;
use crate::operation::{DEFAULT_BODY_BYTES, EMBEDDED_BODY_BYTES, LARGE_BODY_BYTES};
use crate::transport::{MAX_REQUEST_TARGET_BYTES, MAX_RESPONSE_HEADER_BYTES};

/// Embedded response-body capacity in bytes.
pub const EMBEDDED_RESPONSE_BYTES: usize = 64 * 1024;
/// Default response-body capacity in bytes.
pub const DEFAULT_RESPONSE_BYTES: usize = 1024 * 1024;
/// Large response-body capacity in bytes.
pub const LARGE_RESPONSE_BYTES: usize = 8 * 1024 * 1024;

/// Named capacities for all storage used by one complete client execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ClientCapacityProfile {
    target_bytes: usize,
    request_body_bytes: usize,
    response_body_bytes: usize,
    response_header_bytes: usize,
}

impl ClientCapacityProfile {
    /// Small profile for constrained clients and ordinary JSON requests.
    pub const EMBEDDED: Self = Self::new(
        1024,
        EMBEDDED_BODY_BYTES,
        EMBEDDED_RESPONSE_BYTES,
        MAX_RESPONSE_HEADER_BYTES,
    );
    /// General profile supporting the complete request-target limit.
    pub const DEFAULT: Self = Self::new(
        MAX_REQUEST_TARGET_BYTES,
        DEFAULT_BODY_BYTES,
        DEFAULT_RESPONSE_BYTES,
        MAX_RESPONSE_HEADER_BYTES,
    );
    /// Explicit large-payload profile bounded at eight MiB per body.
    pub const LARGE: Self = Self::new(
        MAX_REQUEST_TARGET_BYTES,
        LARGE_BODY_BYTES,
        LARGE_RESPONSE_BYTES,
        MAX_RESPONSE_HEADER_BYTES,
    );

    const fn new(
        target_bytes: usize,
        request_body_bytes: usize,
        response_body_bytes: usize,
        response_header_bytes: usize,
    ) -> Self {
        Self {
            target_bytes,
            request_body_bytes,
            response_body_bytes,
            response_header_bytes,
        }
    }

    /// Returns the required request-target capacity.
    #[must_use]
    pub const fn target_bytes(self) -> usize {
        self.target_bytes
    }

    /// Returns the required request-body capacity.
    #[must_use]
    pub const fn request_body_bytes(self) -> usize {
        self.request_body_bytes
    }

    /// Returns the required response-body capacity.
    #[must_use]
    pub const fn response_body_bytes(self) -> usize {
        self.response_body_bytes
    }

    /// Returns the required response-header capacity.
    #[must_use]
    pub const fn response_header_bytes(self) -> usize {
        self.response_header_bytes
    }

    /// Checks whether all four independent regions satisfy this profile.
    pub const fn validate(
        self,
        target_bytes: usize,
        request_body_bytes: usize,
        response_body_bytes: usize,
        response_header_bytes: usize,
    ) -> Result<(), ClientCapacityError> {
        if target_bytes < self.target_bytes {
            return Err(ClientCapacityError::TargetTooSmall);
        }
        if request_body_bytes < self.request_body_bytes {
            return Err(ClientCapacityError::RequestBodyTooSmall);
        }
        if response_body_bytes < self.response_body_bytes {
            return Err(ClientCapacityError::ResponseBodyTooSmall);
        }
        if response_header_bytes < self.response_header_bytes {
            return Err(ClientCapacityError::ResponseHeadersTooSmall);
        }
        Ok(())
    }
}

/// Failure while admitting or allocating a complete client workspace.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ClientCapacityError {
    /// Request-target storage does not satisfy the selected profile.
    TargetTooSmall,
    /// Request-body storage does not satisfy the selected profile.
    RequestBodyTooSmall,
    /// Response-body storage does not satisfy the selected profile.
    ResponseBodyTooSmall,
    /// Response-header storage does not satisfy the selected profile.
    ResponseHeadersTooSmall,
    /// The allocator rejected the requested bounded profile.
    AllocationFailed,
}

impl_static_error!(ClientCapacityError,
    Self::TargetTooSmall => "client target storage is too small",
    Self::RequestBodyTooSmall => "client request-body storage is too small",
    Self::ResponseBodyTooSmall => "client response-body storage is too small",
    Self::ResponseHeadersTooSmall => "client response-header storage is too small",
    Self::AllocationFailed => "client workspace allocation failed",
);

impl<'storage> ClientWorkspace<'storage> {
    /// Clears and admits four caller-owned regions under one named profile.
    pub fn for_profile(
        target: &'storage mut [u8],
        request_body: &'storage mut [u8],
        response_body: &'storage mut [u8],
        response_headers: &'storage mut [u8],
        profile: ClientCapacityProfile,
    ) -> Result<Self, ClientCapacityError> {
        let workspace = Self::new(target, request_body, response_body, response_headers);
        let (target, request, response, headers) = workspace.capacities();
        profile.validate(target, request, response, headers)?;
        Ok(workspace)
    }
}

/// Fallibly allocated complete workspace cleared in full on drop.
#[cfg(feature = "alloc")]
pub struct OwnedClientWorkspace {
    target: alloc::boxed::Box<[u8]>,
    request_body: alloc::boxed::Box<[u8]>,
    response_body: alloc::boxed::Box<[u8]>,
    response_headers: alloc::boxed::Box<[u8]>,
}

#[cfg(feature = "alloc")]
impl OwnedClientWorkspace {
    /// Allocates exactly one named profile without panicking on allocation failure.
    pub fn try_for_profile(profile: ClientCapacityProfile) -> Result<Self, ClientCapacityError> {
        Ok(Self {
            target: allocate_zeroed(profile.target_bytes)?,
            request_body: allocate_zeroed(profile.request_body_bytes)?,
            response_body: allocate_zeroed(profile.response_body_bytes)?,
            response_headers: allocate_zeroed(profile.response_header_bytes)?,
        })
    }

    /// Borrows all four allocations as one cleanup-owning workspace.
    pub fn workspace(&mut self) -> ClientWorkspace<'_> {
        ClientWorkspace::new(
            &mut self.target,
            &mut self.request_body,
            &mut self.response_body,
            &mut self.response_headers,
        )
    }

    /// Returns capacities without exposing stored bytes.
    #[must_use]
    pub fn capacities(&self) -> (usize, usize, usize, usize) {
        (
            self.target.len(),
            self.request_body.len(),
            self.response_body.len(),
            self.response_headers.len(),
        )
    }
}

#[cfg(feature = "alloc")]
impl Drop for OwnedClientWorkspace {
    fn drop(&mut self) {
        cloud_sdk_sanitization::sanitize_bytes(&mut self.target);
        cloud_sdk_sanitization::sanitize_bytes(&mut self.request_body);
        cloud_sdk_sanitization::sanitize_bytes(&mut self.response_body);
        cloud_sdk_sanitization::sanitize_bytes(&mut self.response_headers);
    }
}

#[cfg(feature = "alloc")]
impl fmt::Debug for OwnedClientWorkspace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OwnedClientWorkspace")
            .field("capacities", &self.capacities())
            .finish_non_exhaustive()
    }
}

#[cfg(feature = "alloc")]
fn allocate_zeroed(len: usize) -> Result<alloc::boxed::Box<[u8]>, ClientCapacityError> {
    let mut bytes = alloc::vec::Vec::new();
    bytes
        .try_reserve_exact(len)
        .map_err(|_| ClientCapacityError::AllocationFailed)?;
    bytes.resize(len, 0);
    Ok(bytes.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::{ClientCapacityError, ClientCapacityProfile};
    use crate::client::ClientWorkspace;

    #[test]
    fn profiles_validate_every_region_at_exact_bounds() {
        let profile = ClientCapacityProfile::DEFAULT;
        assert_eq!(
            profile.validate(
                profile.target_bytes(),
                profile.request_body_bytes(),
                profile.response_body_bytes(),
                profile.response_header_bytes(),
            ),
            Ok(())
        );
        assert_eq!(
            profile.validate(
                profile.target_bytes(),
                profile.request_body_bytes(),
                profile.response_body_bytes() - 1,
                usize::MAX,
            ),
            Err(ClientCapacityError::ResponseBodyTooSmall)
        );
        assert_eq!(
            profile.validate(usize::MAX, usize::MAX, usize::MAX, 0),
            Err(ClientCapacityError::ResponseHeadersTooSmall)
        );
    }

    #[test]
    fn rejected_profile_clears_all_borrowed_regions() {
        let mut target = [0x11; 8];
        let mut request = [0x22; 8];
        let mut response = [0x33; 8];
        let mut headers = [0x44; 8];
        assert!(matches!(
            ClientWorkspace::for_profile(
                &mut target,
                &mut request,
                &mut response,
                &mut headers,
                ClientCapacityProfile::DEFAULT,
            ),
            Err(ClientCapacityError::TargetTooSmall)
        ));
        assert_eq!(target, [0; 8]);
        assert_eq!(request, [0; 8]);
        assert_eq!(response, [0; 8]);
        assert_eq!(headers, [0; 8]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn owned_workspace_allocates_the_exact_selected_profile() {
        let profile = ClientCapacityProfile::EMBEDDED;
        let workspace = super::OwnedClientWorkspace::try_for_profile(profile);
        assert!(workspace.is_ok());
        let Ok(workspace) = workspace else {
            unreachable!("bounded client workspace allocation failed")
        };
        assert_eq!(
            workspace.capacities(),
            (
                profile.target_bytes(),
                profile.request_body_bytes(),
                profile.response_body_bytes(),
                profile.response_header_bytes(),
            )
        );
    }
}
