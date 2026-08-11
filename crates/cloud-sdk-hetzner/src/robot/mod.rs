//! Hetzner Robot Webservice request primitives.
//!
//! Robot uses HTTP Basic authentication and form bodies rather than the
//! bearer-token JSON protocol used by Hetzner Cloud APIs. The module provides
//! bounded forms plus an allocation-gated protected credential and lockout
//! policy. Server operations are source-locked in `v0.78.0`, and server, IP,
//! and subnet cancellation operations are source-locked in `v0.79.0`.

#[cfg(feature = "alloc")]
mod cancellation;
#[cfg(feature = "alloc")]
mod credentials;
mod form;
#[cfg(feature = "serde")]
mod protocol;
#[cfg(feature = "alloc")]
mod server;

/// Maximum Robot error-body bytes admitted by request and response policies.
pub const MAX_ROBOT_ERROR_BODY_BYTES: usize = 65_536;

#[cfg(feature = "alloc")]
pub use cancellation::{
    MAX_ROBOT_CANCELLATION_REASON_INPUT_BYTES, RobotCancellationDate, RobotCancellationReason,
    RobotCancellationRequestError, RobotCancellationSchedule, RobotCancellationValueError,
    RobotIpAddress, RobotIpCancellationCreateRequest, RobotIpCancellationDeleteRequest,
    RobotIpCancellationGetRequest, RobotLocationReservationIntent,
    RobotServerCancellationCreateRequest, RobotServerCancellationDeleteRequest,
    RobotServerCancellationGetRequest, RobotSubnetAddress, RobotSubnetCancellationCreateRequest,
    RobotSubnetCancellationDeleteRequest, RobotSubnetCancellationGetRequest,
};

#[cfg(feature = "serde")]
pub use cancellation::{
    CancellationCanonicalPlanFingerprint, CancellationDestructivePermit, CancellationPermitAttempt,
    CancellationPlanConfirmation, CancellationPlanFingerprintDigest, CancellationPlanSubject,
    CancellationSharedDestructivePermit, CheckedCancellation, MAX_ROBOT_CANCELLATION_REASON_BYTES,
    MAX_ROBOT_CANCELLATION_REASONS, PreparedCancellation, RobotCancellationDecodeError,
    RobotIpCancellation, RobotServerCancellation, RobotServerCancellationReason,
    RobotSubnetCancellation, build_cancellation_canonical_plan, build_cancellation_plan_digest,
    decode_robot_ip_cancellation, decode_robot_server_cancellation,
    decode_robot_subnet_cancellation,
};

#[cfg(feature = "alloc")]
pub use credentials::{
    MAX_ROBOT_PASSWORD_BYTES, MAX_ROBOT_USERNAME_BYTES, RobotCredentialAttempt,
    RobotCredentialError, RobotCredentialRotationError, RobotCredentialScope,
    RobotCredentialStateError, RobotCredentials,
};

pub use form::{
    EncodedRobotForm, MAX_ROBOT_FORM_BODY_BYTES, MAX_ROBOT_FORM_FIELDS, MAX_ROBOT_FORM_NAME_BYTES,
    MAX_ROBOT_FORM_VALUE_BYTES, RobotForm, RobotFormError, RobotFormField, RobotFormSensitivity,
};

#[cfg(feature = "serde")]
pub use protocol::{
    MAX_ROBOT_INPUT_FIELDS, RobotDecodeError, RobotFailure, RobotFailureCategory,
    RobotInvalidInput, RobotProviderError, RobotProviderErrorCode, RobotQuota,
    RobotRetryDisposition, RobotTransientTransport, decode_robot_failure,
};

#[cfg(feature = "alloc")]
pub use server::{
    MAX_ROBOT_SERVER_NAME_BYTES, RobotServerGetRequest, RobotServerListRequest, RobotServerName,
    RobotServerNumber, RobotServerNumberError, RobotServerRequestError, RobotServerUpdateIntent,
    RobotServerUpdateRequest,
};

#[cfg(feature = "serde")]
pub use server::{
    MAX_ROBOT_SERVER_ADDRESSES, MAX_ROBOT_SERVER_LIST_ITEMS, ProtectedIpAddr, RobotServer,
    RobotServerCapabilities, RobotServerDate, RobotServerDecodeError, RobotServerList,
    RobotServerStatus, RobotServerSubnet, RobotServerSummary, RobotStorageBoxNumber,
    decode_robot_server, decode_robot_server_list,
};
