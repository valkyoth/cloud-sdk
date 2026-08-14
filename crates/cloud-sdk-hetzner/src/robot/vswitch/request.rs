use crate::robot::RobotCancellationSchedule;

use super::{
    RobotVSwitchId, RobotVSwitchName, RobotVSwitchServers, RobotVSwitchValueError, RobotVlanId,
};

/// Failure while validating or preparing a Robot vSwitch operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotVSwitchRequestError {
    /// A source value or membership snapshot was invalid.
    Value(RobotVSwitchValueError),
    /// Caller-owned path storage was too small or encoding failed.
    Path,
    /// Robot form validation or encoding failed.
    Form(crate::robot::RobotFormError),
    /// Temporary bounded form storage could not be allocated.
    Allocation,
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

impl_static_error!(RobotVSwitchRequestError,
    Self::Value(_) => "Robot vSwitch value is invalid",
    Self::Path => "Robot vSwitch path preparation failed",
    Self::Form(_) => "Robot vSwitch form preparation failed",
    Self::Allocation => "Robot vSwitch preparation allocation failed",
    Self::InvalidTarget(_) => "Robot vSwitch target is invalid",
    Self::InvalidHeaders(_) => "Robot vSwitch headers are invalid",
    Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
    Self::InvalidOperationId(_) => "Robot vSwitch operation identifier is invalid",
    Self::InvalidMetadata(_) => "Robot vSwitch metadata is invalid",
    Self::InvalidResponsePolicy(_) => "Robot vSwitch response policy is invalid",
    Self::InvalidRawPolicy(_) => "Robot vSwitch raw response policy is invalid",
    Self::InvalidPreparedPolicy(_) => "Robot vSwitch prepared policy is invalid",
);

/// Lists every vSwitch in the authenticated Robot account.
#[derive(Clone, Copy, Debug, Default)]
pub struct RobotVSwitchListRequest;

impl RobotVSwitchListRequest {
    /// Creates a vSwitch inventory request.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

/// Creates one vSwitch with an explicit name and VLAN.
#[derive(Debug)]
pub struct RobotVSwitchCreateRequest {
    pub(super) name: RobotVSwitchName,
    pub(super) vlan: RobotVlanId,
}

impl RobotVSwitchCreateRequest {
    /// Creates a complete vSwitch creation request.
    #[must_use]
    pub const fn new(name: RobotVSwitchName, vlan: RobotVlanId) -> Self {
        Self { name, vlan }
    }
}

/// Gets one vSwitch by provider identity.
#[derive(Clone, Copy, Debug)]
pub struct RobotVSwitchGetRequest {
    pub(super) id: RobotVSwitchId,
}

impl RobotVSwitchGetRequest {
    /// Creates one vSwitch detail request.
    #[must_use]
    pub const fn new(id: RobotVSwitchId) -> Self {
        Self { id }
    }
}

/// Non-empty vSwitch update intent.
#[derive(Debug)]
pub enum RobotVSwitchUpdateIntent {
    /// Changes only the name.
    Rename(RobotVSwitchName),
    /// Changes only the VLAN ID.
    ChangeVlan(RobotVlanId),
    /// Changes both mutable fields atomically in one provider request.
    RenameAndChangeVlan {
        /// Replacement name.
        name: RobotVSwitchName,
        /// Replacement VLAN ID.
        vlan: RobotVlanId,
    },
}

/// Updates one vSwitch's mutable configuration.
#[derive(Debug)]
pub struct RobotVSwitchUpdateRequest {
    pub(super) id: RobotVSwitchId,
    pub(super) intent: RobotVSwitchUpdateIntent,
}

impl RobotVSwitchUpdateRequest {
    /// Creates an update with an unambiguous non-empty intent.
    #[must_use]
    pub const fn new(id: RobotVSwitchId, intent: RobotVSwitchUpdateIntent) -> Self {
        Self { id, intent }
    }
}

/// Cancels one vSwitch now or on a validated date.
pub struct RobotVSwitchCancelRequest {
    pub(super) id: RobotVSwitchId,
    pub(super) schedule: RobotCancellationSchedule,
}

impl RobotVSwitchCancelRequest {
    /// Creates an explicitly destructive cancellation request.
    #[must_use]
    pub const fn new(id: RobotVSwitchId, schedule: RobotCancellationSchedule) -> Self {
        Self { id, schedule }
    }
}

impl core::fmt::Debug for RobotVSwitchCancelRequest {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotVSwitchCancelRequest([redacted])")
    }
}

macro_rules! membership_request {
    ($name:ident, $description:literal) => {
        #[doc = $description]
        #[derive(Clone, Copy, Debug)]
        pub struct $name<'a> {
            pub(super) id: RobotVSwitchId,
            pub(super) servers: RobotVSwitchServers<'a>,
        }

        impl<'a> $name<'a> {
            /// Creates a bounded, duplicate-free membership request.
            #[must_use]
            pub const fn new(id: RobotVSwitchId, servers: RobotVSwitchServers<'a>) -> Self {
                Self { id, servers }
            }
        }
    };
}

membership_request!(
    RobotVSwitchAddServersRequest,
    "Attaches one or more servers to a vSwitch."
);
membership_request!(
    RobotVSwitchRemoveServersRequest,
    "Detaches one or more servers from a vSwitch."
);
