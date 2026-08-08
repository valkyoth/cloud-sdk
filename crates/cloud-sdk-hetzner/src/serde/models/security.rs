//! Dedicated security-resource response dispatch.

use alloc::vec::Vec;
use core::fmt;

use crate::serde::strict_json::Value;

use super::{Certificate, ResponseModelError, SshKey, parse_certificate, parse_ssh_key};

const MAX_SECURITY_RESOURCES: usize = 1_024;

/// Dedicated security resource family.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[non_exhaustive]
pub enum SecurityResourceKind {
    /// TLS certificate.
    Certificate,
    /// SSH public key.
    SshKey,
}

/// Source-complete security resource returned by the checked decoder.
///
/// Ordinary equality is unavailable because each variant can own protected key
/// material.
#[non_exhaustive]
pub enum SecurityResource {
    /// TLS certificate.
    Certificate(Certificate),
    /// SSH public key.
    SshKey(SshKey),
}

impl SecurityResource {
    /// Returns the exact security resource family.
    #[must_use]
    pub const fn kind(&self) -> SecurityResourceKind {
        match self {
            Self::Certificate(_) => SecurityResourceKind::Certificate,
            Self::SshKey(_) => SecurityResourceKind::SshKey,
        }
    }
}

impl fmt::Debug for SecurityResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SecurityResource")
            .field("kind", &self.kind())
            .field("fields", &"[redacted]")
            .finish()
    }
}

pub(crate) fn is_security_resource_root(root: &str) -> bool {
    matches!(
        root,
        "certificate" | "certificates" | "ssh_key" | "ssh_keys"
    )
}

pub(crate) fn parse_security_resource(
    root: &str,
    value: &mut Value,
) -> Result<SecurityResource, ResponseModelError> {
    match root {
        "certificate" | "certificates" => {
            parse_certificate(value).map(SecurityResource::Certificate)
        }
        "ssh_key" | "ssh_keys" => parse_ssh_key(value).map(SecurityResource::SshKey),
        _ => Err(ResponseModelError::EnvelopeMismatch),
    }
}

pub(crate) fn parse_security_resources(
    root: &str,
    value: &mut Value,
) -> Result<Vec<SecurityResource>, ResponseModelError> {
    let values = value.as_array_mut().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_SECURITY_RESOURCES {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut resources = Vec::new();
    resources
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        resources.push(parse_security_resource(root, value)?);
    }
    Ok(resources)
}
