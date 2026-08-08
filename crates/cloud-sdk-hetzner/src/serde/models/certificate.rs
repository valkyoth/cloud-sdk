//! Source-complete certificate response model.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::{Labels, SensitiveText, UtcTimestamp};
use crate::response::ApiErrorCode;

mod parser;

pub(crate) use parser::parse_certificate;

const MAX_LABELS: usize = 64;
const MAX_DOMAINS: usize = 1_024;
const MAX_USES: usize = 1_024;
const MAX_CERTIFICATE_BYTES: usize = 1_048_576;
const MAX_CERTIFICATES_IN_CHAIN: usize = 5;
const BEGIN_CERTIFICATE: &str = "-----BEGIN CERTIFICATE-----";
const END_CERTIFICATE: &str = "-----END CERTIFICATE-----";

/// Certificate kind returned by Hetzner Cloud.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateKind {
    /// Caller-uploaded certificate.
    Uploaded,
    /// Provider-managed certificate.
    Managed,
}

/// Managed-certificate issuance state.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateIssuanceState {
    /// Issuance is still pending.
    Pending,
    /// Issuance completed.
    Completed,
    /// Issuance failed.
    Failed,
}

/// Managed-certificate renewal state.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateRenewalState {
    /// Renewal is scheduled.
    Scheduled,
    /// Renewal is pending.
    Pending,
    /// Renewal failed.
    Failed,
    /// Renewal is unavailable.
    Unavailable,
}

/// One resource using a certificate.
#[non_exhaustive]
pub struct CertificateUse {
    id: u64,
    resource_type: String,
}

impl CertificateUse {
    /// Returns the referenced resource identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the additive provider resource type.
    #[must_use]
    pub fn resource_type(&self) -> &str {
        &self.resource_type
    }
}

impl fmt::Debug for CertificateUse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CertificateUse([redacted])")
    }
}

impl Drop for CertificateUse {
    fn drop(&mut self) {
        sanitize_string(&mut self.resource_type);
    }
}

/// Provider diagnostic embedded in managed-certificate status.
pub struct CertificateError {
    code: ApiErrorCode,
    message: SensitiveText,
}

impl CertificateError {
    /// Returns the stable provider error code.
    #[must_use]
    pub const fn code(&self) -> ApiErrorCode {
        self.code
    }

    /// Inspects the protected provider message without returning a borrowed secret.
    pub fn try_with_message<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.message.try_with_secret(inspect)
    }
}

impl fmt::Debug for CertificateError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CertificateError")
            .field("code", &self.code)
            .field("message", &"[redacted]")
            .finish()
    }
}

/// Optional managed-certificate issuance and renewal state.
#[non_exhaustive]
pub struct CertificateStatus {
    issuance: Option<CertificateIssuanceState>,
    renewal: Option<CertificateRenewalState>,
    error: Option<CertificateError>,
}

impl CertificateStatus {
    /// Returns the issuance state supplied by the provider.
    #[must_use]
    pub const fn issuance(&self) -> Option<CertificateIssuanceState> {
        self.issuance
    }

    /// Returns the renewal state supplied by the provider.
    #[must_use]
    pub const fn renewal(&self) -> Option<CertificateRenewalState> {
        self.renewal
    }

    /// Returns protected failure detail supplied by the provider.
    #[must_use]
    pub const fn error(&self) -> Option<&CertificateError> {
        self.error.as_ref()
    }
}

impl fmt::Debug for CertificateStatus {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CertificateStatus")
            .field("issuance", &self.issuance)
            .field("renewal", &self.renewal)
            .field("error", &self.error.as_ref().map(|_| "[redacted]"))
            .finish()
    }
}

/// Source-complete certificate returned by the checked decoder.
///
/// Ordinary equality is intentionally unavailable because this value can own
/// a complete certificate chain and protected provider diagnostics.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::Certificate;
/// fn compare(left: Certificate, right: Certificate) -> bool { left == right }
/// ```
#[non_exhaustive]
pub struct Certificate {
    id: u64,
    name: String,
    labels: Labels,
    kind: Option<CertificateKind>,
    certificate: Option<SensitiveText>,
    created: UtcTimestamp,
    not_valid_before: Option<UtcTimestamp>,
    not_valid_after: Option<UtcTimestamp>,
    domain_names: Vec<String>,
    fingerprint: Option<String>,
    status: Option<CertificateStatus>,
    used_by: Vec<CertificateUse>,
}

impl Certificate {
    /// Returns the provider identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }
    /// Returns the resource name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
    /// Returns user-defined labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }
    /// Returns the source-known certificate kind when supplied.
    #[must_use]
    pub const fn kind(&self) -> Option<CertificateKind> {
        self.kind
    }
    /// Returns the protected certificate chain when supplied.
    #[must_use]
    pub const fn certificate(&self) -> Option<&SensitiveText> {
        self.certificate.as_ref()
    }
    /// Returns the creation timestamp.
    #[must_use]
    pub const fn created(&self) -> &UtcTimestamp {
        &self.created
    }
    /// Returns the beginning of the validity interval.
    #[must_use]
    pub const fn not_valid_before(&self) -> Option<&UtcTimestamp> {
        self.not_valid_before.as_ref()
    }
    /// Returns the end of the validity interval.
    #[must_use]
    pub const fn not_valid_after(&self) -> Option<&UtcTimestamp> {
        self.not_valid_after.as_ref()
    }
    /// Returns covered domains.
    #[must_use]
    pub fn domain_names(&self) -> &[String] {
        &self.domain_names
    }
    /// Returns the certificate fingerprint when supplied.
    #[must_use]
    pub fn fingerprint(&self) -> Option<&str> {
        self.fingerprint.as_deref()
    }
    /// Returns managed-certificate state when supplied.
    #[must_use]
    pub const fn status(&self) -> Option<&CertificateStatus> {
        self.status.as_ref()
    }
    /// Returns resources using this certificate.
    #[must_use]
    pub fn used_by(&self) -> &[CertificateUse] {
        &self.used_by
    }
}

impl fmt::Debug for Certificate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Certificate")
            .field("id", &"[redacted]")
            .field("kind", &self.kind)
            .field(
                "certificate",
                &self.certificate.as_ref().map(|_| "[redacted]"),
            )
            .field("status", &self.status)
            .field("fields", &"[redacted]")
            .finish()
    }
}

impl Drop for Certificate {
    fn drop(&mut self) {
        sanitize_string(&mut self.name);
        for domain in &mut self.domain_names {
            sanitize_string(domain);
        }
        if let Some(fingerprint) = &mut self.fingerprint {
            sanitize_string(fingerprint);
        }
    }
}
