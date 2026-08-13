use core::fmt;

/// Maximum UTF-8 bytes accepted for an OS, distribution, or language value.
pub const MAX_ROBOT_BOOT_VALUE_BYTES: usize = 256;
/// Maximum bytes accepted for an authorized-key fingerprint or returned key.
pub const MAX_ROBOT_BOOT_KEY_BYTES: usize = 16_384;
/// Maximum authorized-key fingerprints accepted by one activation.
pub const MAX_ROBOT_BOOT_AUTHORIZED_KEYS: usize = 64;

/// Failure while validating a bounded Robot boot value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotBootValueError {
    /// The value was empty.
    Empty,
    /// The value exceeded its source-policy bound.
    TooLong,
    /// The value contained a form-hostile control byte.
    InvalidByte,
}

impl_static_error!(RobotBootValueError,
    Self::Empty => "Robot boot value is empty",
    Self::TooLong => "Robot boot value exceeds its size limit",
    Self::InvalidByte => "Robot boot value contains an invalid byte",
);

fn validate(value: &str, maximum: usize) -> Result<(), RobotBootValueError> {
    if value.is_empty() {
        return Err(RobotBootValueError::Empty);
    }
    if value.len() > maximum {
        return Err(RobotBootValueError::TooLong);
    }
    if value.chars().any(|character| {
        character.is_control()
            || matches!(
                character,
                '\u{061c}'
                    | '\u{200b}'..='\u{200f}'
                    | '\u{202a}'..='\u{202e}'
                    | '\u{2060}'..='\u{2069}'
                    | '\u{feff}'
            )
    }) {
        return Err(RobotBootValueError::InvalidByte);
    }
    Ok(())
}

/// Bounded OS, distribution, or language selector.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotBootValue<'a>(&'a str);

impl<'a> RobotBootValue<'a> {
    /// Validates one provider-advertised selector.
    pub fn new(value: &'a str) -> Result<Self, RobotBootValueError> {
        validate(value, MAX_ROBOT_BOOT_VALUE_BYTES)?;
        Ok(Self(value))
    }

    pub(super) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl fmt::Debug for RobotBootValue<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotBootValue([redacted])")
    }
}

/// Bounded SSH-key fingerprint supplied to boot activation.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotBootKey<'a>(&'a str);

impl<'a> RobotBootKey<'a> {
    /// Validates one fingerprint without assuming an undocumented algorithm.
    pub fn new(value: &'a str) -> Result<Self, RobotBootValueError> {
        validate(value, MAX_ROBOT_BOOT_KEY_BYTES)?;
        Ok(Self(value))
    }

    pub(super) const fn as_str(self) -> &'a str {
        self.0
    }
}

impl fmt::Debug for RobotBootKey<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotBootKey([redacted])")
    }
}

/// Bounded keyboard layout accepted by Rescue activation.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotKeyboardLayout<'a>(RobotBootValue<'a>);

impl<'a> RobotKeyboardLayout<'a> {
    /// Validates a keyboard layout selector.
    pub fn new(value: &'a str) -> Result<Self, RobotBootValueError> {
        Ok(Self(RobotBootValue::new(value)?))
    }

    pub(super) const fn as_str(self) -> &'a str {
        self.0.as_str()
    }
}

impl fmt::Debug for RobotKeyboardLayout<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotKeyboardLayout([redacted])")
    }
}
