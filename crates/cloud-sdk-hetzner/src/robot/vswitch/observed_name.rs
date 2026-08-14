use cloud_sdk_sanitization::SecretBoxBytes;

use super::{MAX_ROBOT_VSWITCH_NAME_BYTES, RobotVSwitchName, RobotVSwitchValueError};

/// Protected name observed in a Robot vSwitch response.
///
/// Provider names outside the high-assurance outbound profile remain bounded,
/// protected, and explicitly quarantined instead of invalidating an inventory.
pub struct RobotVSwitchObservedName(ObservedName);

enum ObservedName {
    HighAssurance(RobotVSwitchName),
    Quarantined(SecretBoxBytes),
}

impl RobotVSwitchObservedName {
    pub(super) fn from_provider(value: &str) -> Result<Self, RobotVSwitchValueError> {
        if value.is_empty() || value.len() > MAX_ROBOT_VSWITCH_NAME_BYTES {
            return Err(RobotVSwitchValueError::InvalidName);
        }
        match RobotVSwitchName::new(value) {
            Ok(name) => Ok(Self(ObservedName::HighAssurance(name))),
            Err(RobotVSwitchValueError::InvalidName) => {
                SecretBoxBytes::try_from_slice(value.as_bytes(), MAX_ROBOT_VSWITCH_NAME_BYTES)
                    .map(|value| Self(ObservedName::Quarantined(value)))
                    .map_err(|_| RobotVSwitchValueError::Allocation)
            }
            Err(error) => Err(error),
        }
    }

    /// Reports whether the provider name satisfies the outbound policy.
    #[must_use]
    pub const fn is_high_assurance(&self) -> bool {
        matches!(&self.0, ObservedName::HighAssurance(_))
    }

    /// Reports whether the provider name requires untrusted-data handling.
    #[must_use]
    pub const fn is_quarantined(&self) -> bool {
        matches!(&self.0, ObservedName::Quarantined(_))
    }

    /// Returns the reusable high-assurance name, if admitted.
    #[must_use]
    pub const fn as_high_assurance(&self) -> Option<&RobotVSwitchName> {
        match &self.0 {
            ObservedName::HighAssurance(name) => Some(name),
            ObservedName::Quarantined(_) => None,
        }
    }

    /// Runs a closure with temporary access to the exact provider text.
    ///
    /// Text from a quarantined name remains untrusted and must not be rendered
    /// in operator interfaces or reused in requests without validation.
    pub fn try_with_text<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        match &self.0 {
            ObservedName::HighAssurance(name) => name.try_with_text(inspect),
            ObservedName::Quarantined(value) => {
                value.with_secret(|bytes| core::str::from_utf8(bytes).map(inspect))
            }
        }
    }

    pub(super) fn matches(&self, expected: &RobotVSwitchName) -> bool {
        self.as_high_assurance() == Some(expected)
    }
}

impl core::fmt::Debug for RobotVSwitchObservedName {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("RobotVSwitchObservedName([redacted])")
    }
}
