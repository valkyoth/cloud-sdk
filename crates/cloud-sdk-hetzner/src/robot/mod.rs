//! Hetzner Robot Webservice request primitives.
//!
//! Robot uses HTTP Basic authentication and form bodies rather than the
//! bearer-token JSON protocol used by Hetzner Cloud APIs. The module provides
//! bounded forms plus an allocation-gated protected credential and lockout
//! policy. Server operations are source-locked in `v0.78.0`, and server, IP,
//! and subnet cancellation operations are source-locked in `v0.79.0`. Active
//! single-IP and separate-MAC operations are source-locked in `v0.80.0`, and
//! active subnet and subnet-MAC operations are source-locked in `v0.81.0`.

#[cfg(feature = "alloc")]
mod cancellation;
#[cfg(feature = "alloc")]
mod credentials;
#[cfg(feature = "serde")]
mod duplicates;
mod form;
#[cfg(feature = "alloc")]
mod ip;
#[cfg(feature = "serde")]
mod protocol;
#[cfg(feature = "alloc")]
mod server;
#[cfg(feature = "alloc")]
mod subnet;

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

#[cfg(feature = "alloc")]
pub use ip::{
    RobotIpGetRequest, RobotIpListRequest, RobotIpMacDeleteRequest, RobotIpMacGetRequest,
    RobotIpMacSetRequest, RobotIpRequestError, RobotIpTrafficUpdate, RobotIpUpdateRequest,
    RobotMacAddress, RobotMacAddressError,
};

#[cfg(feature = "serde")]
pub use ip::{
    CheckedRobotIp, MAX_ROBOT_IP_LIST_ITEMS, PreparedRobotIp, RobotIp,
    RobotIpCanonicalPlanFingerprint, RobotIpDecodeError, RobotIpDestructivePermit, RobotIpList,
    RobotIpMac, RobotIpMutationPermit, RobotIpPermitAttempt, RobotIpPermitRequest,
    RobotIpPlanConfirmation, RobotIpPlanFingerprintDigest, RobotIpPlanSubject,
    RobotIpSharedDestructivePermit, RobotIpSharedMutationPermit, RobotIpSummary,
    RobotIpTrafficPolicy, build_robot_ip_canonical_plan, build_robot_ip_plan_digest,
    decode_robot_ip, decode_robot_ip_list, decode_robot_ip_mac,
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

#[cfg(feature = "alloc")]
pub use subnet::{
    RobotSubnetGetRequest, RobotSubnetListRequest, RobotSubnetMacDeleteRequest,
    RobotSubnetMacGetRequest, RobotSubnetMacSetRequest, RobotSubnetRequestError,
    RobotSubnetTrafficUpdate, RobotSubnetUpdateRequest,
};

#[cfg(feature = "serde")]
pub use subnet::{
    CheckedRobotSubnet, MAX_ROBOT_SUBNET_LIST_ITEMS, MAX_ROBOT_SUBNET_MAC_OPTIONS,
    PreparedRobotSubnet, RobotSubnet, RobotSubnetCanonicalPlanFingerprint, RobotSubnetDecodeError,
    RobotSubnetDestructivePermit, RobotSubnetFailureCode, RobotSubnetList, RobotSubnetMac,
    RobotSubnetMacOption, RobotSubnetMutationPermit, RobotSubnetPermitAttempt,
    RobotSubnetPermitRequest, RobotSubnetPlanConfirmation, RobotSubnetPlanFingerprintDigest,
    RobotSubnetPlanSubject, RobotSubnetSharedDestructivePermit, RobotSubnetSharedMutationPermit,
    RobotSubnetTrafficPolicy, build_robot_subnet_canonical_plan, build_robot_subnet_plan_digest,
    decode_robot_subnet, decode_robot_subnet_list, decode_robot_subnet_mac,
};

#[cfg(feature = "serde")]
pub use server::{
    MAX_ROBOT_SERVER_ADDRESSES, MAX_ROBOT_SERVER_LIST_ITEMS, ProtectedIpAddr, RobotServer,
    RobotServerCapabilities, RobotServerDate, RobotServerDecodeError, RobotServerList,
    RobotServerStatus, RobotServerSubnet, RobotServerSummary, RobotStorageBoxNumber,
    decode_robot_server, decode_robot_server_list,
};
