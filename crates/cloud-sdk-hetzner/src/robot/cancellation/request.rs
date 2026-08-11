use super::{RobotCancellationDate, RobotIpAddress, RobotSubnetAddress};
use crate::robot::server::RobotServerNumber;

/// Maximum caller-supplied cancellation reason bytes.
pub const MAX_ROBOT_CANCELLATION_REASON_INPUT_BYTES: usize = 4_096;

/// Failure while validating or preparing a Robot cancellation operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotCancellationRequestError {
    /// A cancellation reason is empty, too long, or unsafe to display.
    InvalidReason,
    /// Caller-owned path storage was too small or path encoding failed.
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

impl core::fmt::Display for RobotCancellationRequestError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidReason => "Robot cancellation reason is invalid",
            Self::Path => "Robot cancellation path preparation failed",
            Self::Form(_) => "Robot cancellation form preparation failed",
            Self::InvalidTarget(_) => "Robot cancellation target is invalid",
            Self::InvalidHeaders(_) => "Robot cancellation headers are invalid",
            Self::InvalidEndpoint(_) => "official Robot endpoint is invalid",
            Self::InvalidOperationId(_) => "Robot cancellation operation identifier is invalid",
            Self::InvalidMetadata(_) => "Robot cancellation metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Robot cancellation response policy is invalid",
            Self::InvalidRawPolicy(_) => "Robot cancellation raw response policy is invalid",
            Self::InvalidPreparedPolicy(_) => "Robot cancellation prepared policy is invalid",
        })
    }
}

impl core::error::Error for RobotCancellationRequestError {}

/// Explicit cancellation schedule accepted by Robot.
pub enum RobotCancellationSchedule {
    /// Cancel at the provider's earliest admitted instant (`now`).
    Immediate,
    /// Cancel on an exact calendar date.
    On(RobotCancellationDate),
}

impl core::fmt::Debug for RobotCancellationSchedule {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotCancellationSchedule([redacted])")
    }
}

/// Explicit location-reservation field policy for server cancellation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotLocationReservationIntent {
    /// Omit the field when the preceding GET reports reservation unavailable.
    Omit,
    /// Request reservation when the preceding GET reports it available.
    Reserve,
    /// Explicitly decline reservation when the preceding GET requires a choice.
    DoNotReserve,
}

/// Bounded borrowed cancellation reason. Diagnostics are always redacted.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotCancellationReason<'a>(&'a str);

impl<'a> RobotCancellationReason<'a> {
    /// Validates bounded text without controls or directional formatting.
    pub fn new(value: &'a str) -> Result<Self, RobotCancellationRequestError> {
        if value.is_empty()
            || value.len() > MAX_ROBOT_CANCELLATION_REASON_INPUT_BYTES
            || value
                .chars()
                .any(crate::display::is_unsafe_display_character)
        {
            return Err(RobotCancellationRequestError::InvalidReason);
        }
        Ok(Self(value))
    }

    pub(super) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl core::fmt::Debug for RobotCancellationReason<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotCancellationReason([redacted])")
    }
}

/// Gets one server cancellation.
pub struct RobotServerCancellationGetRequest {
    pub(super) number: RobotServerNumber,
}
impl RobotServerCancellationGetRequest {
    /// Creates a bodyless get request.
    #[must_use]
    pub const fn new(number: RobotServerNumber) -> Self {
        Self { number }
    }
}

/// Creates or replaces one server cancellation.
pub struct RobotServerCancellationCreateRequest<'a> {
    pub(super) number: RobotServerNumber,
    pub(super) schedule: RobotCancellationSchedule,
    pub(super) reason: Option<RobotCancellationReason<'a>>,
    pub(super) reservation: RobotLocationReservationIntent,
}
impl<'a> RobotServerCancellationCreateRequest<'a> {
    /// Creates a destructive request with explicit schedule and reservation intent.
    #[must_use]
    pub const fn new(
        number: RobotServerNumber,
        schedule: RobotCancellationSchedule,
        reason: Option<RobotCancellationReason<'a>>,
        reservation: RobotLocationReservationIntent,
    ) -> Self {
        Self {
            number,
            schedule,
            reason,
            reservation,
        }
    }
}

/// Revokes one server cancellation.
pub struct RobotServerCancellationDeleteRequest {
    pub(super) number: RobotServerNumber,
}
impl RobotServerCancellationDeleteRequest {
    /// Creates a destructive revoke request.
    #[must_use]
    pub const fn new(number: RobotServerNumber) -> Self {
        Self { number }
    }
}

macro_rules! address_requests {
    ($get:ident, $create:ident, $delete:ident, $identity:ty, $field:ident) => {
        /// Gets one source-locked cancellation resource.
        pub struct $get {
            pub(super) $field: $identity,
        }
        impl $get {
            /// Creates a bodyless get request.
            #[must_use]
            pub const fn new($field: $identity) -> Self {
                Self { $field }
            }
        }

        /// Creates or replaces one source-locked cancellation resource.
        pub struct $create {
            pub(super) $field: $identity,
            pub(super) schedule: RobotCancellationSchedule,
        }
        impl $create {
            /// Creates a destructive request with an explicit schedule.
            #[must_use]
            pub const fn new($field: $identity, schedule: RobotCancellationSchedule) -> Self {
                Self { $field, schedule }
            }
        }

        /// Revokes one source-locked cancellation resource.
        pub struct $delete {
            pub(super) $field: $identity,
        }
        impl $delete {
            /// Creates a destructive revoke request.
            #[must_use]
            pub const fn new($field: $identity) -> Self {
                Self { $field }
            }
        }
    };
}

address_requests!(
    RobotIpCancellationGetRequest,
    RobotIpCancellationCreateRequest,
    RobotIpCancellationDeleteRequest,
    RobotIpAddress,
    ip
);
address_requests!(
    RobotSubnetCancellationGetRequest,
    RobotSubnetCancellationCreateRequest,
    RobotSubnetCancellationDeleteRequest,
    RobotSubnetAddress,
    subnet
);

macro_rules! redacted_debug {
    ($($type:ident),+ $(,)?) => {$ (
        impl core::fmt::Debug for $type {
            fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str(concat!(stringify!($type), "([redacted])"))
            }
        }
    )+ };
}

redacted_debug!(
    RobotServerCancellationGetRequest,
    RobotServerCancellationDeleteRequest,
    RobotIpCancellationGetRequest,
    RobotIpCancellationCreateRequest,
    RobotIpCancellationDeleteRequest,
    RobotSubnetCancellationGetRequest,
    RobotSubnetCancellationCreateRequest,
    RobotSubnetCancellationDeleteRequest,
);

impl core::fmt::Debug for RobotServerCancellationCreateRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotServerCancellationCreateRequest([redacted])")
    }
}
