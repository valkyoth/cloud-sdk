//! Pricing, folder, and sensitive text models.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::SecretString;

use super::cloud_schema::validate_model;
use super::{
    CloudObject, ResponseModelError, checked_text, object, required, validate_text, value_text,
};
use crate::serde::strict_json::{Map, Value};

const MAX_FOLDERS: usize = 4096;

/// Sensitive provider text requiring closure-scoped access.
///
/// The owned allocation is cleared when this value is dropped. This type does
/// not implement `Clone`; callers must not duplicate protected response text
/// through an infallible allocation path.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::SensitiveText;
///
/// fn duplicate(secret: SensitiveText) {
///     let _ = secret.clone();
/// }
/// ```
pub struct SensitiveText(SecretString);

impl SensitiveText {
    pub(crate) fn new(value: SecretString) -> Self {
        Self(value)
    }

    /// Runs a closure with temporary read-only access to the sensitive text.
    pub fn try_with_secret<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }

    pub(crate) fn validate(&self, max: usize) -> Result<(), ResponseModelError> {
        self.try_with_secret(|value| validate_text(value, max))
            .map_err(|_| ResponseModelError::InvalidText)?
    }

    pub(crate) fn validate_multiline(&self, max: usize) -> Result<(), ResponseModelError> {
        self.try_with_secret(|value| {
            if value.is_empty()
                || value.len() > max
                || value.chars().any(|character| {
                    !matches!(character, '\t' | '\n' | '\r')
                        && (character.is_control()
                            || matches!(
                                character,
                                '\u{061c}'
                                    | '\u{200b}'..='\u{200f}'
                                    | '\u{202a}'..='\u{202e}'
                                    | '\u{2060}'..='\u{2069}'
                                    | '\u{feff}'
                            ))
                })
            {
                Err(ResponseModelError::InvalidText)
            } else {
                Ok(())
            }
        })
        .map_err(|_| ResponseModelError::InvalidText)?
    }
}

impl PartialEq for SensitiveText {
    fn eq(&self, other: &Self) -> bool {
        other
            .try_with_secret(|value| self.0.constant_time_eq(value))
            .unwrap_or(false)
    }
}

impl Eq for SensitiveText {}

impl fmt::Debug for SensitiveText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SensitiveText([redacted])")
    }
}

/// Exported DNS zonefile with redacted diagnostics.
#[derive(Eq, PartialEq)]
pub struct ZoneFile(SensitiveText);

impl ZoneFile {
    /// Runs a closure with temporary access to the sensitive zonefile.
    pub fn try_with_zonefile<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }
}

impl fmt::Debug for ZoneFile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ZoneFile([redacted])")
    }
}

/// Validated pricing summary.
#[derive(PartialEq)]
pub struct Pricing {
    currency: String,
    vat_rate: String,
    fields: CloudObject,
}

impl Pricing {
    /// Fallibly copies the complete pricing response.
    pub fn try_clone(&self) -> Result<Self, ResponseModelError> {
        Ok(Self {
            currency: checked_text(&self.currency, 16)?,
            vat_rate: checked_text(&self.vat_rate, 64)?,
            fields: self.fields.try_clone()?,
        })
    }

    /// Returns the provider currency code.
    #[must_use]
    pub fn currency(&self) -> &str {
        &self.currency
    }

    /// Returns the decimal VAT rate text.
    #[must_use]
    pub fn vat_rate(&self) -> &str {
        &self.vat_rate
    }

    /// Returns the number of server-type price records.
    #[must_use]
    pub fn server_type_prices(&self) -> usize {
        match self.fields.get("server_types") {
            Some(super::CloudValue::Array(values)) => values.len(),
            _ => 0,
        }
    }

    /// Returns the number of load-balancer-type price records.
    #[must_use]
    pub fn load_balancer_type_prices(&self) -> usize {
        match self.fields.get("load_balancer_types") {
            Some(super::CloudValue::Array(values)) => values.len(),
            _ => 0,
        }
    }

    /// Returns every source-known and future pricing field in stable order.
    #[must_use]
    pub const fn fields(&self) -> &CloudObject {
        &self.fields
    }
}

impl fmt::Debug for Pricing {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Pricing")
            .field("currency", &"[redacted]")
            .field("vat_rate", &"[redacted]")
            .field("fields", &"[redacted]")
            .finish()
    }
}

/// Bounded Storage Box folder paths.
#[derive(Clone, Eq, PartialEq)]
pub struct FolderList(Vec<String>);

impl FolderList {
    /// Returns validated folder paths.
    #[must_use]
    pub fn folders(&self) -> &[String] {
        &self.0
    }
}

impl fmt::Debug for FolderList {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FolderList")
            .field("count", &self.0.len())
            .field("folders", &"[redacted]")
            .finish()
    }
}

pub(crate) fn parse_zonefile(value: &mut Value) -> Result<ZoneFile, ResponseModelError> {
    let secret = value
        .take_string()
        .map(SensitiveText::new)
        .ok_or(ResponseModelError::WrongType)?;
    secret.validate_multiline(8_388_608)?;
    Ok(ZoneFile(secret))
}

pub(crate) fn parse_folders(value: &Value) -> Result<FolderList, ResponseModelError> {
    let values = value.as_array().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_FOLDERS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut folders = Vec::new();
    folders
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        folders.push(value_text(value, 4096)?);
    }
    Ok(FolderList(folders))
}

pub(crate) fn parse_pricing(value: &Value) -> Result<Pricing, ResponseModelError> {
    validate_model("pricing", value)?;
    let fields = object(value)?;
    Ok(Pricing {
        currency: text(fields, "currency", 16)?,
        vat_rate: text(fields, "vat_rate", 64)?,
        fields: CloudObject::from_value(value)?,
    })
}

fn text(fields: &Map, key: &str, max: usize) -> Result<String, ResponseModelError> {
    value_text(required(fields, key)?, max)
}

#[cfg(test)]
mod tests {
    use cloud_sdk_sanitization::SecretString;

    use super::SensitiveText;

    #[test]
    fn sensitive_text_adopts_protected_storage_without_another_allocation() {
        let protected = SecretString::from_secret_str("temporary secret");
        let before = protected.with_secret_bytes(|value| value.as_ptr());
        let secret = SensitiveText::new(protected);
        let after = secret.0.with_secret_bytes(|value| value.as_ptr());

        assert_eq!(before, after);
        assert_eq!(
            secret.try_with_secret(|value| value == "temporary secret"),
            Ok(true)
        );
        assert!(!alloc::format!("{secret:?}").contains("temporary secret"));
    }

    #[test]
    fn sensitive_text_equality_compares_secret_contents() {
        let left = SensitiveText::new(SecretString::from_secret_str("same"));
        let equal = SensitiveText::new(SecretString::from_secret_str("same"));
        let different = SensitiveText::new(SecretString::from_secret_str("different"));

        assert_eq!(left, equal);
        assert_ne!(left, different);
    }
}
