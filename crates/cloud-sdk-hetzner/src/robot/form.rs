//! Bounded atomic `application/x-www-form-urlencoded` encoding for Robot.

use cloud_sdk::buffer::{SnapshotEncoder, encode_snapshot_bounded, measure_snapshot_bounded};
use cloud_sdk_sanitization::sanitize_bytes;

/// Maximum fields admitted in one Robot form body.
pub const MAX_ROBOT_FORM_FIELDS: usize = 4_096;
/// Maximum UTF-8 bytes admitted in one Robot form field name.
pub const MAX_ROBOT_FORM_NAME_BYTES: usize = 256;
/// Maximum UTF-8 bytes admitted in one Robot form field value.
pub const MAX_ROBOT_FORM_VALUE_BYTES: usize = 1_048_576;
/// Maximum encoded Robot form body bytes.
pub const MAX_ROBOT_FORM_BODY_BYTES: usize = cloud_sdk::operation::LARGE_BODY_BYTES;

/// Sensitivity carried from form fields to an encoded request body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotFormSensitivity {
    /// The form contains no caller-marked secret value.
    Public,
    /// At least one form value is secret-bearing.
    Sensitive,
}

/// Failure while validating or encoding a Robot form body.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum RobotFormError {
    /// The form contains more than [`MAX_ROBOT_FORM_FIELDS`] fields.
    TooManyFields,
    /// A field name is empty.
    EmptyName,
    /// A field name exceeds [`MAX_ROBOT_FORM_NAME_BYTES`].
    NameTooLong,
    /// A field name is outside the reviewed Robot parameter grammar.
    InvalidName,
    /// A field value exceeds [`MAX_ROBOT_FORM_VALUE_BYTES`].
    ValueTooLong,
    /// The encoded form exceeds [`MAX_ROBOT_FORM_BODY_BYTES`] or overflows.
    BodyTooLong,
    /// The caller-owned destination is smaller than the exact encoded body.
    BufferTooSmall,
}

impl_static_error!(RobotFormError,
    Self::TooManyFields => "Robot form has too many fields",
    Self::EmptyName => "Robot form field name is empty",
    Self::NameTooLong => "Robot form field name exceeds the length limit",
    Self::InvalidName => "Robot form field name is invalid",
    Self::ValueTooLong => "Robot form field value exceeds the length limit",
    Self::BodyTooLong => "Robot form body exceeds the encoded length limit",
    Self::BufferTooSmall => "Robot form output buffer is too small",
);

/// One borrowed, validated Robot form field.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RobotFormField<'a> {
    name: &'a str,
    value: &'a str,
    sensitivity: RobotFormSensitivity,
}

impl<'a> RobotFormField<'a> {
    /// Creates a public form field.
    pub fn public(name: &'a str, value: &'a str) -> Result<Self, RobotFormError> {
        Self::new(name, value, RobotFormSensitivity::Public)
    }

    /// Creates a secret-bearing form field.
    ///
    /// This marks the complete encoded body sensitive. The borrowed source
    /// remains caller-owned and must be cleared separately when applicable.
    pub fn sensitive(name: &'a str, value: &'a str) -> Result<Self, RobotFormError> {
        Self::new(name, value, RobotFormSensitivity::Sensitive)
    }

    fn new(
        name: &'a str,
        value: &'a str,
        sensitivity: RobotFormSensitivity,
    ) -> Result<Self, RobotFormError> {
        validate_name(name)?;
        if value.len() > MAX_ROBOT_FORM_VALUE_BYTES {
            return Err(RobotFormError::ValueTooLong);
        }
        Ok(Self {
            name,
            value,
            sensitivity,
        })
    }

    /// Returns the public provider parameter name.
    #[must_use]
    pub const fn name(self) -> &'a str {
        self.name
    }

    /// Returns whether this field marks its value sensitive.
    #[must_use]
    pub const fn sensitivity(self) -> RobotFormSensitivity {
        self.sensitivity
    }
}

impl core::fmt::Debug for RobotFormField<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotFormField")
            .field("name", &self.name)
            .field("value", &"[redacted]")
            .field("sensitivity", &self.sensitivity)
            .finish()
    }
}

/// Immutable ordered Robot form snapshot.
#[derive(Clone, Copy)]
pub struct RobotForm<'a> {
    fields: &'a [RobotFormField<'a>],
    sensitivity: RobotFormSensitivity,
}

impl<'a> RobotForm<'a> {
    /// Validates an ordered field snapshot.
    ///
    /// Duplicate names are retained in caller order because Robot uses them
    /// for repeated array parameters such as `server[]`.
    pub fn new(fields: &'a [RobotFormField<'a>]) -> Result<Self, RobotFormError> {
        if fields.len() > MAX_ROBOT_FORM_FIELDS {
            return Err(RobotFormError::TooManyFields);
        }
        let sensitivity = if fields
            .iter()
            .any(|field| field.sensitivity == RobotFormSensitivity::Sensitive)
        {
            RobotFormSensitivity::Sensitive
        } else {
            RobotFormSensitivity::Public
        };
        Ok(Self {
            fields,
            sensitivity,
        })
    }

    /// Returns the exact encoded body length after checked preflight.
    pub fn encoded_len(self) -> Result<usize, RobotFormError> {
        measure_snapshot_bounded(
            self.fields,
            MAX_ROBOT_FORM_BODY_BYTES,
            RobotFormError::BodyTooLong,
            encode_fields,
        )
    }

    /// Atomically encodes into caller-owned storage.
    ///
    /// Validation and capacity failures leave `output` unchanged. Once exact
    /// capacity is admitted, the complete output is cleared before writing so
    /// a shorter body cannot retain a secret tail from earlier use. The
    /// returned guard clears the complete output on drop.
    pub fn encode<'output>(
        self,
        output: &'output mut [u8],
    ) -> Result<EncodedRobotForm<'output>, RobotFormError> {
        let required = self.encoded_len()?;
        if output.len() < required {
            return Err(RobotFormError::BufferTooSmall);
        }

        sanitize_bytes(output);
        let len = encode_snapshot_bounded(
            self.fields,
            output,
            MAX_ROBOT_FORM_BODY_BYTES,
            RobotFormError::BodyTooLong,
            encode_fields,
        )?;
        Ok(EncodedRobotForm {
            output,
            len,
            sensitivity: self.sensitivity,
        })
    }

    /// Returns the number of ordered fields.
    #[must_use]
    pub const fn len(self) -> usize {
        self.fields.len()
    }

    /// Reports whether the form contains no fields.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.fields.is_empty()
    }

    /// Returns the aggregate sensitivity classification.
    #[must_use]
    pub const fn sensitivity(self) -> RobotFormSensitivity {
        self.sensitivity
    }
}

impl core::fmt::Debug for RobotForm<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("RobotForm")
            .field("fields", &self.fields.len())
            .field("sensitivity", &self.sensitivity)
            .finish()
    }
}

/// Encoded Robot form body that clears its complete backing buffer on drop.
pub struct EncodedRobotForm<'output> {
    output: &'output mut [u8],
    len: usize,
    sensitivity: RobotFormSensitivity,
}

impl EncodedRobotForm<'_> {
    /// Returns the encoded body bytes for transport.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.output.get(..self.len).unwrap_or_default()
    }

    /// Returns the exact encoded byte length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the encoded body is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Returns the aggregate sensitivity classification.
    #[must_use]
    pub const fn sensitivity(&self) -> RobotFormSensitivity {
        self.sensitivity
    }
}

impl Drop for EncodedRobotForm<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.output);
    }
}

impl core::fmt::Debug for EncodedRobotForm<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("EncodedRobotForm")
            .field("body", &"[redacted]")
            .field("len", &self.len)
            .field("sensitivity", &self.sensitivity)
            .finish()
    }
}

fn validate_name(name: &str) -> Result<(), RobotFormError> {
    if name.is_empty() {
        return Err(RobotFormError::EmptyName);
    }
    if name.len() > MAX_ROBOT_FORM_NAME_BYTES {
        return Err(RobotFormError::NameTooLong);
    }
    if name.bytes().any(|byte| {
        !matches!(
            byte,
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'_' | b'-' | b'[' | b']'
        )
    }) {
        return Err(RobotFormError::InvalidName);
    }
    Ok(())
}

fn encode_fields(
    fields: &[RobotFormField<'_>],
    encoder: &mut SnapshotEncoder<'_, RobotFormError>,
) -> Result<(), RobotFormError> {
    let mut first = true;
    for field in fields {
        if first {
            first = false;
        } else {
            encoder.byte(b'&')?;
        }
        encoder.form_component(field.name)?;
        encoder.byte(b'=')?;
        encoder.form_component(field.value)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
