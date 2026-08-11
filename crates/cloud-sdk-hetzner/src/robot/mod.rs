//! Hetzner Robot Webservice request primitives.
//!
//! Robot uses HTTP Basic authentication and form bodies rather than the
//! bearer-token JSON protocol used by Hetzner Cloud APIs. Authentication and
//! endpoint-family operations are introduced by later milestones; this module
//! starts with the bounded form wire contract.

mod form;

pub use form::{
    EncodedRobotForm, MAX_ROBOT_FORM_BODY_BYTES, MAX_ROBOT_FORM_FIELDS, MAX_ROBOT_FORM_NAME_BYTES,
    MAX_ROBOT_FORM_VALUE_BYTES, RobotForm, RobotFormError, RobotFormField, RobotFormSensitivity,
};
