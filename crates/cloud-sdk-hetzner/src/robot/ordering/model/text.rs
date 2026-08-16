use cloud_sdk_sanitization::SecretString;

use super::super::RobotOrderCatalogDecodeError;

/// Maximum bytes admitted for one provider-owned catalog text field.
pub(super) const MAX_PROVIDER_TEXT_BYTES: usize = 4_096;

/// Bounded protected provider-owned catalog text.
pub struct RobotOrderText(pub(super) SecretString);

impl RobotOrderText {
    pub(in crate::robot::ordering) fn from_provider(
        value: SecretString,
    ) -> Result<Self, RobotOrderCatalogDecodeError> {
        let valid = value
            .try_with_secret(|text| {
                !text.is_empty()
                    && text.len() <= MAX_PROVIDER_TEXT_BYTES
                    && !text.bytes().any(|byte| byte == 0)
            })
            .map_err(|_| RobotOrderCatalogDecodeError::InvalidText)?;
        if valid {
            Ok(Self(value))
        } else {
            Err(RobotOrderCatalogDecodeError::InvalidText)
        }
    }

    /// Runs a closure with temporary access to untrusted provider text.
    ///
    /// The caller must escape this value for its destination context.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }

    pub(in crate::robot::ordering) fn compare(&self, other: &Self) -> core::cmp::Ordering {
        self.try_with_text(|left| {
            other
                .try_with_text(|right| left.cmp(right))
                .unwrap_or(core::cmp::Ordering::Equal)
        })
        .unwrap_or(core::cmp::Ordering::Equal)
    }
}

impl core::fmt::Debug for RobotOrderText {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotOrderText([redacted])")
    }
}
