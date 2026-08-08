//! Source-complete certificate response model.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use super::{
    Labels, ResponseModelError, SensitiveText, object, parse_labels, required, valid_error_code,
    value_text,
};
use crate::response::ApiErrorCode;
use crate::serde::strict_json::{Map, Value};

const MAX_LABELS: usize = 64;
const MAX_DOMAINS: usize = 1_024;
const MAX_USES: usize = 1_024;

/// Certificate kind returned by Hetzner Cloud.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateKind {
    /// Caller-uploaded certificate.
    Uploaded,
    /// Provider-managed certificate.
    Managed,
}

/// One resource using a certificate.
#[non_exhaustive]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateUse {
    /// Referenced resource identifier.
    pub id: u64,
    /// Referenced resource type.
    pub resource_type: String,
}

/// Provider diagnostic embedded in managed-certificate status.
#[non_exhaustive]
#[derive(Eq, PartialEq)]
pub struct CertificateError {
    /// Stable provider error code.
    pub code: ApiErrorCode,
    message: SensitiveText,
}

impl CertificateError {
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
#[derive(Debug, Eq, PartialEq)]
pub struct CertificateStatus {
    /// Issuance state when supplied by the provider.
    pub issuance: Option<String>,
    /// Renewal state when supplied by the provider.
    pub renewal: Option<String>,
    /// Protected failure detail when supplied by the provider.
    pub error: Option<CertificateError>,
}

/// Source-complete `get_certificate` result.
#[non_exhaustive]
#[derive(Eq, PartialEq)]
pub struct Certificate {
    /// Provider resource identifier.
    pub id: u64,
    /// Resource name.
    pub name: String,
    /// User-defined labels.
    pub labels: Labels,
    /// Certificate type when supplied by the provider.
    pub kind: Option<CertificateKind>,
    /// Protected certificate chain in PEM format.
    pub certificate: Option<SensitiveText>,
    /// Creation timestamp text.
    pub created: String,
    /// Earliest validity timestamp.
    pub not_valid_before: Option<String>,
    /// Expiry timestamp.
    pub not_valid_after: Option<String>,
    /// Covered domains.
    pub domain_names: Vec<String>,
    /// SHA-256 certificate fingerprint.
    pub fingerprint: Option<String>,
    /// Optional managed-certificate state.
    pub status: Option<CertificateStatus>,
    /// Resources currently using the certificate.
    pub used_by: Vec<CertificateUse>,
}

impl fmt::Debug for Certificate {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Certificate")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("labels", &self.labels)
            .field("kind", &self.kind)
            .field(
                "certificate",
                &self.certificate.as_ref().map(|_| "[redacted]"),
            )
            .field("created", &self.created)
            .field("not_valid_before", &self.not_valid_before)
            .field("not_valid_after", &self.not_valid_after)
            .field("domain_names", &self.domain_names)
            .field("fingerprint", &self.fingerprint)
            .field("status", &self.status)
            .field("used_by", &self.used_by)
            .finish()
    }
}

pub(crate) fn parse_certificate(value: &mut Value) -> Result<Certificate, ResponseModelError> {
    let envelope = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let fields = envelope
        .get_mut("certificate")
        .ok_or(ResponseModelError::MissingField)?
        .as_object_mut()
        .ok_or(ResponseModelError::WrongType)?;
    let kind = fields.get("type").map(parse_kind).transpose()?;
    let certificate = take_nullable_secret(fields, "certificate", 1_048_576)?;
    let status = fields
        .get_mut("status")
        .map(parse_status)
        .transpose()?
        .flatten();
    Ok(Certificate {
        id: positive_u64(fields, "id")?,
        name: text(fields, "name", 256)?,
        labels: parse_labels(required(fields, "labels")?, MAX_LABELS)?,
        kind,
        certificate,
        created: text(fields, "created", 64)?,
        not_valid_before: nullable_text(fields, "not_valid_before", 64)?,
        not_valid_after: nullable_text(fields, "not_valid_after", 64)?,
        domain_names: text_list(required(fields, "domain_names")?, MAX_DOMAINS, 256)?,
        fingerprint: nullable_text(fields, "fingerprint", 256)?,
        status,
        used_by: parse_uses(required(fields, "used_by")?)?,
    })
}

fn parse_kind(value: &Value) -> Result<CertificateKind, ResponseModelError> {
    value
        .try_with_str(|value| match value {
            "uploaded" => Some(CertificateKind::Uploaded),
            "managed" => Some(CertificateKind::Managed),
            _ => None,
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .flatten()
        .ok_or(ResponseModelError::UnknownEnumValue)
}

fn parse_status(value: &mut Value) -> Result<Option<CertificateStatus>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let issuance = fields
        .get("issuance")
        .map(|value| enum_text(value, &["pending", "completed", "failed"]))
        .transpose()?;
    let renewal = fields
        .get("renewal")
        .map(|value| enum_text(value, &["scheduled", "pending", "failed", "unavailable"]))
        .transpose()?;
    let error = fields
        .get_mut("error")
        .map(parse_error)
        .transpose()?
        .flatten();
    Ok(Some(CertificateStatus {
        issuance,
        renewal,
        error,
    }))
}

fn parse_error(value: &mut Value) -> Result<Option<CertificateError>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let code = text(fields, "code", 128)?;
    if !valid_error_code(&code, 128) {
        return Err(ResponseModelError::InvalidText);
    }
    let message = take_secret(fields, "message", 16_384)?;
    message.validate(16_384)?;
    Ok(Some(CertificateError {
        code: ApiErrorCode::from_api_str(&code),
        message,
    }))
}

fn parse_uses(value: &Value) -> Result<Vec<CertificateUse>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_USES {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut uses = Vec::new();
    uses.try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        let fields = object(value)?;
        uses.push(CertificateUse {
            id: positive_u64(fields, "id")?,
            resource_type: text(fields, "type", 128)?,
        });
    }
    Ok(uses)
}

fn text_list(value: &Value, limit: usize, max: usize) -> Result<Vec<String>, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > limit {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        output.push(value_text(value, max)?);
    }
    Ok(output)
}

fn take_nullable_secret(
    fields: &mut Map,
    key: &str,
    max: usize,
) -> Result<Option<SensitiveText>, ResponseModelError> {
    let value = fields
        .get_mut(key)
        .ok_or(ResponseModelError::MissingField)?;
    if value.is_null() {
        return Ok(None);
    }
    take_secret(fields, key, max).map(Some)
}

fn take_secret(
    fields: &mut Map,
    key: &str,
    max: usize,
) -> Result<SensitiveText, ResponseModelError> {
    let secret = fields
        .get_mut(key)
        .ok_or(ResponseModelError::MissingField)?
        .take_string()
        .map(SensitiveText::new)
        .ok_or(ResponseModelError::WrongType)?;
    secret.validate_multiline(max)?;
    Ok(secret)
}

fn enum_text(value: &Value, known: &[&str]) -> Result<String, ResponseModelError> {
    let value = value_text(value, 64)?;
    known
        .contains(&value.as_str())
        .then_some(value)
        .ok_or(ResponseModelError::UnknownEnumValue)
}

fn nullable_text(
    fields: &Map,
    key: &str,
    max: usize,
) -> Result<Option<String>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value_text(value, max).map(Some)
    }
}

fn text(fields: &Map, key: &str, max: usize) -> Result<String, ResponseModelError> {
    value_text(required(fields, key)?, max)
}

fn positive_u64(fields: &Map, key: &str) -> Result<u64, ResponseModelError> {
    required(fields, key)?
        .as_u64()
        .filter(|value| *value != 0 && *value <= 9_007_199_254_740_991)
        .ok_or(ResponseModelError::InvalidNumber)
}

#[cfg(test)]
mod tests {
    use super::parse_error;
    use crate::serde::models::ResponseModelError;
    use crate::serde::strict_json::parse;

    #[test]
    fn certificate_error_codes_reject_non_ascii_identifiers() {
        for code in [
            "line\\u2028break",
            "paragraph\\u2029break",
            "soft\\u00adhyphen",
            "direction\\u206acontrol",
        ] {
            let input = alloc::format!("{{\"code\":\"{code}\",\"message\":\"safe\"}}");
            let Ok(mut value) = parse(input.as_bytes()) else {
                unreachable!("certificate error fixture failed to parse")
            };
            assert_eq!(
                parse_error(&mut value),
                Err(ResponseModelError::InvalidText)
            );
        }
    }
}
