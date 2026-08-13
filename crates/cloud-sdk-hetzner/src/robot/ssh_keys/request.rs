use super::{RobotSshKeyData, RobotSshKeyFingerprint, RobotSshKeyName};

/// Failure while validating or preparing a Robot SSH-key operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotSshKeyRequestError {
    /// Caller-owned path storage was too small or encoding failed.
    Path,
    /// Robot form validation or encoding failed.
    Form(crate::robot::RobotFormError),
    /// The constructed request target was rejected.
    InvalidTarget(cloud_sdk::transport::RequestTargetError),
    /// Source-locked request headers were rejected.
    InvalidHeaders(cloud_sdk::transport::HeaderError),
    /// The official Robot endpoint policy was invalid.
    InvalidEndpoint(crate::endpoint::OfficialEndpointError),
    /// A source-locked operation identifier was invalid.
    InvalidOperationId(cloud_sdk::operation::OperationIdError),
    /// Operation safety metadata was internally inconsistent.
    InvalidMetadata(cloud_sdk::operation::OperationMetadataError),
    /// The success-response policy was internally inconsistent.
    InvalidResponsePolicy(cloud_sdk::operation::ResponsePolicyValidationError),
    /// The raw response-wire policy was internally inconsistent.
    InvalidRawPolicy(cloud_sdk::transport::RawResponsePolicyError),
    /// Cross-policy prepared-request validation failed.
    InvalidPreparedPolicy(cloud_sdk::operation::PreparedRequestPolicyError),
}

impl_static_error!(RobotSshKeyRequestError,
    Self::Path => "Robot SSH-key path preparation failed",
    Self::Form(_) => "Robot SSH-key form preparation failed",
    Self::InvalidTarget(_) => "Robot SSH-key target is invalid",
    Self::InvalidHeaders(_) => "Robot SSH-key headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot SSH-key operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot SSH-key metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot SSH-key response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot SSH-key raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot SSH-key prepared policy is invalid",
);

/// Lists every SSH key in the Robot account.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotSshKeyListRequest;

impl RobotSshKeyListRequest {
    /// Creates an account-wide list request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Adds one SSH public key.
pub struct RobotSshKeyCreateRequest<'a> {
    pub(super) name: RobotSshKeyName,
    pub(super) data: RobotSshKeyData<'a>,
}

impl<'a> RobotSshKeyCreateRequest<'a> {
    /// Creates a request with required validated fields.
    #[must_use]
    pub const fn new(name: RobotSshKeyName, data: RobotSshKeyData<'a>) -> Self {
        Self { name, data }
    }

    /// Returns the protected requested name.
    #[must_use]
    pub const fn name(&self) -> &RobotSshKeyName {
        &self.name
    }

    /// Returns the redacted public-key request value.
    #[must_use]
    pub const fn data(&self) -> &RobotSshKeyData<'a> {
        &self.data
    }
}

impl core::fmt::Debug for RobotSshKeyCreateRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyCreateRequest([redacted])")
    }
}

macro_rules! fingerprint_request {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        pub struct $name {
            pub(super) fingerprint: RobotSshKeyFingerprint,
        }

        impl $name {
            /// Creates a request for one canonical key fingerprint.
            #[must_use]
            pub const fn new(fingerprint: RobotSshKeyFingerprint) -> Self {
                Self { fingerprint }
            }

            /// Returns the exact protected fingerprint.
            #[must_use]
            pub const fn fingerprint(&self) -> &RobotSshKeyFingerprint {
                &self.fingerprint
            }
        }

        impl core::fmt::Debug for $name {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($name), "([redacted])"))
            }
        }
    };
}

fingerprint_request!(RobotSshKeyGetRequest, "Gets one SSH key.");
fingerprint_request!(RobotSshKeyDeleteRequest, "Deletes one SSH key.");

/// Changes only the name of one fingerprint-selected SSH key.
pub struct RobotSshKeyUpdateRequest {
    pub(super) fingerprint: RobotSshKeyFingerprint,
    pub(super) name: RobotSshKeyName,
}

impl RobotSshKeyUpdateRequest {
    /// Creates a name update for one canonical fingerprint.
    #[must_use]
    pub const fn new(fingerprint: RobotSshKeyFingerprint, name: RobotSshKeyName) -> Self {
        Self { fingerprint, name }
    }

    /// Returns the exact protected fingerprint.
    #[must_use]
    pub const fn fingerprint(&self) -> &RobotSshKeyFingerprint {
        &self.fingerprint
    }

    /// Returns the exact protected replacement name.
    #[must_use]
    pub const fn name(&self) -> &RobotSshKeyName {
        &self.name
    }
}

impl core::fmt::Debug for RobotSshKeyUpdateRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotSshKeyUpdateRequest([redacted])")
    }
}
