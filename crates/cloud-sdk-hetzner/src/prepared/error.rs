//! Payload-free preparation failures.

use cloud_sdk::operation::{
    OperationIdError, OperationMetadataError, ResponsePolicyValidationError,
};
use cloud_sdk::transport::{EndpointIdentityError, RequestTargetError};

/// Failure while producing a complete Hetzner transport request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HetznerPreparationError {
    /// The endpoint path could not be encoded into caller storage.
    Path,
    /// The query could not be encoded into caller storage.
    Query,
    /// The JSON body could not be encoded into caller storage.
    Body,
    /// The endpoint requires a query that was not supplied.
    MissingQuery,
    /// The endpoint does not admit a query.
    UnexpectedQuery,
    /// The endpoint requires a request body that was not supplied.
    MissingBody,
    /// The endpoint does not admit a request body.
    UnexpectedBody,
    /// The supplied typed query or body belongs to another operation.
    OperationMismatch,
    /// The source-locked operation identifier was invalid.
    InvalidOperationId(OperationIdError),
    /// The complete origin-form target failed provider-neutral validation.
    InvalidTarget(RequestTargetError),
    /// The canonical official endpoint identity was invalid.
    InvalidOfficialEndpoint(EndpointIdentityError),
    /// Source-locked operation metadata was internally inconsistent.
    InvalidMetadata(OperationMetadataError),
    /// Source-locked response policy was internally inconsistent.
    InvalidResponsePolicy(ResponsePolicyValidationError),
}

impl core::fmt::Display for HetznerPreparationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::Path => "Hetzner endpoint path preparation failed",
            Self::Query => "Hetzner query preparation failed",
            Self::Body => "Hetzner request body preparation failed",
            Self::MissingQuery => "Hetzner operation requires a query",
            Self::UnexpectedQuery => "Hetzner operation does not accept a query",
            Self::MissingBody => "Hetzner operation requires a request body",
            Self::UnexpectedBody => "Hetzner operation does not accept a request body",
            Self::OperationMismatch => "Hetzner request component belongs to another operation",
            Self::InvalidOperationId(_) => "Hetzner operation identifier is invalid",
            Self::InvalidTarget(_) => "Hetzner request target is invalid",
            Self::InvalidOfficialEndpoint(_) => "official Hetzner endpoint is invalid",
            Self::InvalidMetadata(_) => "Hetzner operation metadata is invalid",
            Self::InvalidResponsePolicy(_) => "Hetzner response policy is invalid",
        })
    }
}

impl core::error::Error for HetznerPreparationError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::InvalidTarget(error) => Some(error),
            Self::InvalidOperationId(error) => Some(error),
            Self::InvalidOfficialEndpoint(error) => Some(error),
            Self::InvalidMetadata(error) => Some(error),
            Self::InvalidResponsePolicy(error) => Some(error),
            _ => None,
        }
    }
}
