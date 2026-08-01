use core::fmt;

use cloud_sdk_sanitization::{sanitize_bytes, sanitize_value};

/// Maximum exact-byte quota extension name length.
pub const MAX_QUOTA_EXTENSION_NAME_BYTES: usize = 32;
/// Maximum exact-byte quota extension value length.
pub const MAX_QUOTA_EXTENSION_VALUE_BYTES: usize = 64;

/// Bounded informational quota extension.
///
/// Values are retained exactly but redacted from `Debug` because provider
/// partition keys and policy parameters can disclose account structure.
#[derive(Clone, Eq, PartialEq)]
pub struct QuotaExtension {
    name: [u8; MAX_QUOTA_EXTENSION_NAME_BYTES],
    name_len: u8,
    value: [u8; MAX_QUOTA_EXTENSION_VALUE_BYTES],
    value_len: u8,
}

impl QuotaExtension {
    /// Copies one visible-ASCII name and value into fixed-capacity storage.
    pub fn new(name: &[u8], value: &[u8]) -> Result<Self, QuotaExtensionError> {
        validate(name, MAX_QUOTA_EXTENSION_NAME_BYTES, true)?;
        validate(value, MAX_QUOTA_EXTENSION_VALUE_BYTES, false)?;
        let mut result = Self {
            name: [0; MAX_QUOTA_EXTENSION_NAME_BYTES],
            name_len: u8::try_from(name.len()).map_err(|_| QuotaExtensionError::NameTooLong)?,
            value: [0; MAX_QUOTA_EXTENSION_VALUE_BYTES],
            value_len: u8::try_from(value.len()).map_err(|_| QuotaExtensionError::ValueTooLong)?,
        };
        let name_target = result
            .name
            .get_mut(..name.len())
            .ok_or(QuotaExtensionError::NameTooLong)?;
        name_target.copy_from_slice(name);
        let value_target = result
            .value
            .get_mut(..value.len())
            .ok_or(QuotaExtensionError::ValueTooLong)?;
        value_target.copy_from_slice(value);
        Ok(result)
    }

    /// Returns the exact extension name bytes.
    #[must_use]
    pub fn name(&self) -> &[u8] {
        self.name
            .get(..usize::from(self.name_len))
            .unwrap_or_default()
    }

    /// Returns the exact extension value bytes.
    #[must_use]
    pub fn value(&self) -> &[u8] {
        self.value
            .get(..usize::from(self.value_len))
            .unwrap_or_default()
    }

    fn clear(&mut self) {
        sanitize_bytes(&mut self.name);
        sanitize_bytes(&mut self.value);
        sanitize_value(&mut self.name_len);
        sanitize_value(&mut self.value_len);
    }
}

impl Drop for QuotaExtension {
    fn drop(&mut self) {
        self.clear();
    }
}

impl fmt::Debug for QuotaExtension {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("QuotaExtension")
            .field("name", &self.name())
            .field("value", &"[redacted]")
            .finish()
    }
}

fn validate(value: &[u8], maximum: usize, name: bool) -> Result<(), QuotaExtensionError> {
    if value.is_empty() {
        return Err(if name {
            QuotaExtensionError::NameEmpty
        } else {
            QuotaExtensionError::ValueEmpty
        });
    }
    if value.len() > maximum {
        return Err(if name {
            QuotaExtensionError::NameTooLong
        } else {
            QuotaExtensionError::ValueTooLong
        });
    }
    if !value.iter().all(|byte| byte.is_ascii_graphic()) {
        return Err(QuotaExtensionError::InvalidByte);
    }
    Ok(())
}

/// Invalid informational quota extension.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaExtensionError {
    /// The extension name was empty.
    NameEmpty,
    /// The extension name exceeded its bound.
    NameTooLong,
    /// The extension value was empty.
    ValueEmpty,
    /// The extension value exceeded its bound.
    ValueTooLong,
    /// A name or value contained whitespace, a control, or non-ASCII data.
    InvalidByte,
}

impl_static_error!(QuotaExtensionError,
    Self::NameEmpty => "quota extension name is empty",
    Self::NameTooLong => "quota extension name exceeds its length limit",
    Self::ValueEmpty => "quota extension value is empty",
    Self::ValueTooLong => "quota extension value exceeds its length limit",
    Self::InvalidByte => "quota extension contains an invalid byte",
);

#[cfg(test)]
mod tests {
    use super::QuotaExtension;

    #[test]
    fn explicit_cleanup_clears_complete_name_value_and_lengths() {
        let extension = QuotaExtension::new(b"partition", b"account-42");
        assert!(extension.is_ok());
        let Ok(mut extension) = extension else {
            return;
        };

        extension.clear();

        assert!(extension.name.iter().all(|byte| *byte == 0));
        assert!(extension.value.iter().all(|byte| *byte == 0));
        assert_eq!(extension.name_len, 0);
        assert_eq!(extension.value_len, 0);
    }
}
