//! Hetzner Robot Webservice request primitives.
//!
//! Robot uses HTTP Basic authentication and form bodies rather than the
//! bearer-token JSON protocol used by Hetzner Cloud APIs. The module provides
//! bounded forms plus an allocation-gated protected credential and lockout
//! policy. Endpoint-family operations remain later milestones.

#[cfg(feature = "alloc")]
mod credentials;
mod form;

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
