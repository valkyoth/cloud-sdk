use alloc::string::String;
use alloc::vec::Vec;

use super::{
    BEGIN_CERTIFICATE, Certificate, CertificateError, CertificateIssuanceState, CertificateKind,
    CertificateRenewalState, CertificateStatus, CertificateUse, END_CERTIFICATE,
    MAX_CERTIFICATE_BYTES, MAX_CERTIFICATES_IN_CHAIN, MAX_DOMAINS, MAX_LABELS, MAX_USES,
};
use crate::response::ApiErrorCode;
use crate::security::certificates::CertificateDomainName;
use crate::serde::models::cloud_schema::validate_model;
use crate::serde::models::wipe_string::{WipeString, WipeStrings};
use crate::serde::models::{
    ResponseModelError, SensitiveText, UtcTimestamp, object, parse_labels, required,
    valid_error_code, value_text,
};
use crate::serde::strict_json::{Map, Value};

pub(crate) fn parse_certificate(value: &mut Value) -> Result<Certificate, ResponseModelError> {
    validate_model("certificate", value)?;
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let id = positive_u64(fields, "id")?;
    let name = WipeString::new(text(fields, "name", 256)?);
    let labels = parse_labels(required(fields, "labels")?, MAX_LABELS)?;
    let kind = fields.get("type").map(parse_kind).transpose()?;
    let certificate = take_nullable_secret(fields, "certificate", MAX_CERTIFICATE_BYTES)?;
    if let Some(certificate) = &certificate {
        certificate
            .try_with_secret(validate_certificate_chain)
            .map_err(|_| ResponseModelError::InvalidText)??;
    }
    let created = timestamp(fields, "created")?;
    let not_valid_before = nullable_timestamp(fields, "not_valid_before")?;
    let not_valid_after = nullable_timestamp(fields, "not_valid_after")?;
    let domain_names = domain_list(required(fields, "domain_names")?)?;
    let fingerprint = nullable_guarded_text(fields, "fingerprint", 256)?;
    let status = fields
        .get_mut("status")
        .map(parse_status)
        .transpose()?
        .flatten();
    validate_kind_status(kind, certificate.as_ref(), status.as_ref())?;
    let used_by = parse_uses(required(fields, "used_by")?)?;
    Ok(Certificate {
        id,
        name: name.into_inner(),
        labels,
        kind,
        certificate,
        created,
        not_valid_before,
        not_valid_after,
        domain_names: domain_names.into_inner(),
        fingerprint: fingerprint.map(WipeString::into_inner),
        status,
        used_by,
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
    let issuance = fields.get("issuance").map(parse_issuance).transpose()?;
    let renewal = fields.get("renewal").map(parse_renewal).transpose()?;
    let error = fields
        .get_mut("error")
        .map(parse_error)
        .transpose()?
        .flatten();
    let failed = issuance == Some(CertificateIssuanceState::Failed)
        || renewal == Some(CertificateRenewalState::Failed);
    if failed != error.is_some() {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(Some(CertificateStatus {
        issuance,
        renewal,
        error,
    }))
}

fn parse_issuance(value: &Value) -> Result<CertificateIssuanceState, ResponseModelError> {
    value
        .try_with_str(|value| match value {
            "pending" => Some(CertificateIssuanceState::Pending),
            "completed" => Some(CertificateIssuanceState::Completed),
            "failed" => Some(CertificateIssuanceState::Failed),
            _ => None,
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .flatten()
        .ok_or(ResponseModelError::UnknownEnumValue)
}

fn parse_renewal(value: &Value) -> Result<CertificateRenewalState, ResponseModelError> {
    value
        .try_with_str(|value| match value {
            "scheduled" => Some(CertificateRenewalState::Scheduled),
            "pending" => Some(CertificateRenewalState::Pending),
            "failed" => Some(CertificateRenewalState::Failed),
            "unavailable" => Some(CertificateRenewalState::Unavailable),
            _ => None,
        })
        .map_err(|_| ResponseModelError::InvalidText)?
        .flatten()
        .ok_or(ResponseModelError::UnknownEnumValue)
}

fn parse_error(value: &mut Value) -> Result<Option<CertificateError>, ResponseModelError> {
    if value.is_null() {
        return Ok(None);
    }
    let fields = value.as_object_mut().ok_or(ResponseModelError::WrongType)?;
    let code = WipeString::new(text(fields, "code", 128)?);
    if !valid_error_code(code.as_str(), 128) {
        return Err(ResponseModelError::InvalidText);
    }
    let message = take_secret(fields, "message", 16_384)?;
    let classified = ApiErrorCode::from_api_str(code.as_str());
    Ok(Some(CertificateError {
        code: classified,
        code_text: code.into_inner(),
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
        let resource_type = WipeString::new(text(fields, "type", 128)?);
        uses.push(CertificateUse {
            id: positive_u64(fields, "id")?,
            resource_type: resource_type.into_inner(),
        });
    }
    Ok(uses)
}

fn domain_list(value: &Value) -> Result<WipeStrings, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_DOMAINS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut output = WipeStrings::with_capacity(values.len())?;
    for value in values {
        let domain = WipeString::new(value_text(value, 256)?);
        CertificateDomainName::new(domain.as_str()).map_err(|_| ResponseModelError::InvalidText)?;
        output.push(domain);
    }
    Ok(output)
}

fn validate_kind_status(
    kind: Option<CertificateKind>,
    certificate: Option<&SensitiveText>,
    status: Option<&CertificateStatus>,
) -> Result<(), ResponseModelError> {
    if kind == Some(CertificateKind::Uploaded) && (certificate.is_none() || status.is_some()) {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(())
}

fn validate_certificate_chain(value: &str) -> Result<(), ResponseModelError> {
    let mut remainder = value.trim();
    let mut count = 0usize;
    while !remainder.is_empty() {
        let body = remainder
            .strip_prefix(BEGIN_CERTIFICATE)
            .ok_or(ResponseModelError::InvalidText)?;
        let end = body
            .find(END_CERTIFICATE)
            .ok_or(ResponseModelError::InvalidText)?;
        let encoded = body
            .get(..end)
            .ok_or(ResponseModelError::InvalidText)?
            .trim();
        if encoded.is_empty() || !encoded.bytes().all(is_pem_byte) {
            return Err(ResponseModelError::InvalidText);
        }
        count = count
            .checked_add(1)
            .ok_or(ResponseModelError::TooManyItems)?;
        if count > MAX_CERTIFICATES_IN_CHAIN {
            return Err(ResponseModelError::TooManyItems);
        }
        let certificate_end = end
            .checked_add(END_CERTIFICATE.len())
            .ok_or(ResponseModelError::InvalidText)?;
        remainder = body
            .get(certificate_end..)
            .ok_or(ResponseModelError::InvalidText)?
            .trim();
    }
    if count == 0 {
        Err(ResponseModelError::InvalidText)
    } else {
        Ok(())
    }
}

fn is_pem_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'=' | b'\r' | b'\n')
}

fn take_nullable_secret(
    fields: &mut Map,
    key: &str,
    max: usize,
) -> Result<Option<SensitiveText>, ResponseModelError> {
    if required(fields, key)?.is_null() {
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

fn timestamp(fields: &Map, key: &str) -> Result<UtcTimestamp, ResponseModelError> {
    required(fields, key)?
        .try_with_str(UtcTimestamp::try_new)
        .map_err(|_| ResponseModelError::InvalidText)?
        .ok_or(ResponseModelError::WrongType)?
}

fn nullable_timestamp(fields: &Map, key: &str) -> Result<Option<UtcTimestamp>, ResponseModelError> {
    if required(fields, key)?.is_null() {
        Ok(None)
    } else {
        timestamp(fields, key).map(Some)
    }
}

fn nullable_guarded_text(
    fields: &Map,
    key: &str,
    max: usize,
) -> Result<Option<WipeString>, ResponseModelError> {
    let value = required(fields, key)?;
    if value.is_null() {
        Ok(None)
    } else {
        value_text(value, max).map(WipeString::new).map(Some)
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
    use super::{parse_error, validate_certificate_chain};
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
            assert!(matches!(
                parse_error(&mut value),
                Err(ResponseModelError::InvalidText)
            ));
        }
    }

    #[test]
    fn certificate_chain_accepts_five_and_rejects_six_or_malformed_blocks() {
        let block = "-----BEGIN CERTIFICATE-----\nYQ==\n-----END CERTIFICATE-----";
        assert_eq!(
            validate_certificate_chain(&alloc::vec![block; 5].join("\n")),
            Ok(())
        );
        assert_eq!(
            validate_certificate_chain(&alloc::vec![block; 6].join("\n")),
            Err(ResponseModelError::TooManyItems)
        );
        assert_eq!(
            validate_certificate_chain("-----BEGIN CERTIFICATE-----\n!\n-----END CERTIFICATE-----"),
            Err(ResponseModelError::InvalidText)
        );
    }
}
