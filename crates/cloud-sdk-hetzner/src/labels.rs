//! Label and label-selector domains.

/// Maximum label selector length admitted before endpoint implementation.
pub const MAX_LABEL_SELECTOR_BYTES: usize = 1024;

/// Maximum label key length admitted by the conservative SDK policy.
pub const MAX_LABEL_KEY_BYTES: usize = 63;

/// Maximum label value length admitted by the conservative SDK policy.
pub const MAX_LABEL_VALUE_BYTES: usize = 63;

/// Label validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LabelError {
    /// Label keys must not be empty.
    EmptyKey,
    /// Label keys are capped by [`MAX_LABEL_KEY_BYTES`].
    KeyTooLong,
    /// Label keys must start and end with an ASCII alphanumeric byte.
    InvalidKeyBoundary,
    /// Label keys admit only ASCII alphanumeric, dash, underscore, and dot.
    InvalidKeyByte,
    /// Label values are capped by [`MAX_LABEL_VALUE_BYTES`].
    ValueTooLong,
    /// Label values admit only ASCII alphanumeric, dash, underscore, dot, and empty.
    InvalidValueByte,
    /// Label selectors must not be empty.
    EmptySelector,
    /// Label selectors are capped by [`MAX_LABEL_SELECTOR_BYTES`].
    SelectorTooLong,
    /// Label selectors must not contain control bytes.
    InvalidSelectorByte,
    /// Label selectors must use a conservative, non-empty selector structure.
    InvalidSelectorSyntax,
}

/// Borrowed label key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LabelKey<'a> {
    value: &'a str,
}

impl<'a> LabelKey<'a> {
    /// Creates a conservatively validated label key.
    pub fn new(value: &'a str) -> Result<Self, LabelError> {
        if value.is_empty() {
            return Err(LabelError::EmptyKey);
        }
        if value.len() > MAX_LABEL_KEY_BYTES {
            return Err(LabelError::KeyTooLong);
        }
        if !has_alphanumeric_boundaries(value) {
            return Err(LabelError::InvalidKeyBoundary);
        }
        for byte in value.bytes() {
            if !is_label_atom_byte(byte) {
                return Err(LabelError::InvalidKeyByte);
            }
        }
        Ok(Self { value })
    }

    /// Returns the label key.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Borrowed label value.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LabelValue<'a> {
    value: &'a str,
}

impl<'a> LabelValue<'a> {
    /// Creates a conservatively validated label value.
    pub fn new(value: &'a str) -> Result<Self, LabelError> {
        if value.len() > MAX_LABEL_VALUE_BYTES {
            return Err(LabelError::ValueTooLong);
        }
        for byte in value.bytes() {
            if !is_label_atom_byte(byte) {
                return Err(LabelError::InvalidValueByte);
            }
        }
        Ok(Self { value })
    }

    /// Returns the label value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Borrowed label selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LabelSelector<'a> {
    value: &'a str,
}

impl<'a> LabelSelector<'a> {
    /// Creates a bounded label selector.
    pub fn new(value: &'a str) -> Result<Self, LabelError> {
        if value.is_empty() {
            return Err(LabelError::EmptySelector);
        }
        if value.len() > MAX_LABEL_SELECTOR_BYTES {
            return Err(LabelError::SelectorTooLong);
        }
        validate_selector(value)?;
        Ok(Self { value })
    }

    /// Returns the selector string.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

fn has_alphanumeric_boundaries(value: &str) -> bool {
    match (value.as_bytes().first(), value.as_bytes().last()) {
        (Some(first), Some(last)) => is_ascii_alphanumeric(*first) && is_ascii_alphanumeric(*last),
        _ => false,
    }
}

fn is_label_atom_byte(byte: u8) -> bool {
    is_ascii_alphanumeric(byte) || matches!(byte, b'-' | b'_' | b'.')
}

fn validate_selector(value: &str) -> Result<(), LabelError> {
    let mut depth = 0u8;
    let mut term_has_token = false;
    let mut waiting_for_set_value = false;

    for byte in value.bytes() {
        if byte < 0x20 || byte == 0x7f {
            return Err(LabelError::InvalidSelectorByte);
        }
        if !is_selector_byte(byte) {
            return Err(LabelError::InvalidSelectorByte);
        }
        match byte {
            b' ' => {}
            b',' if depth == 0 => {
                if !term_has_token {
                    return Err(LabelError::InvalidSelectorSyntax);
                }
                term_has_token = false;
            }
            b',' => {
                if waiting_for_set_value {
                    return Err(LabelError::InvalidSelectorSyntax);
                }
                waiting_for_set_value = true;
            }
            b'(' => {
                if depth != 0 {
                    return Err(LabelError::InvalidSelectorSyntax);
                }
                depth = match depth.checked_add(1) {
                    Some(next) => next,
                    None => return Err(LabelError::InvalidSelectorSyntax),
                };
                waiting_for_set_value = true;
                term_has_token = true;
            }
            b')' => {
                if depth == 0 || waiting_for_set_value {
                    return Err(LabelError::InvalidSelectorSyntax);
                }
                depth = match depth.checked_sub(1) {
                    Some(next) => next,
                    None => return Err(LabelError::InvalidSelectorSyntax),
                };
                term_has_token = true;
            }
            _ => {
                term_has_token = true;
                waiting_for_set_value = false;
            }
        }
    }

    if depth != 0 || !term_has_token {
        return Err(LabelError::InvalidSelectorSyntax);
    }
    Ok(())
}

fn is_selector_byte(byte: u8) -> bool {
    is_label_atom_byte(byte) || matches!(byte, b' ' | b'!' | b'=' | b',' | b'(' | b')')
}

fn is_ascii_alphanumeric(byte: u8) -> bool {
    byte.is_ascii_alphanumeric()
}

#[cfg(test)]
mod tests {
    use super::{LabelError, LabelKey, LabelSelector, LabelValue};

    #[test]
    fn validates_label_keys() {
        assert_eq!(LabelKey::new(""), Err(LabelError::EmptyKey));
        assert_eq!(LabelKey::new("-bad"), Err(LabelError::InvalidKeyBoundary));
        assert_eq!(LabelKey::new("bad!"), Err(LabelError::InvalidKeyBoundary));
        assert_eq!(
            LabelKey::new("env.prod").map(LabelKey::as_str),
            Ok("env.prod")
        );
    }

    #[test]
    fn validates_label_values() {
        assert_eq!(
            LabelValue::new("bad value"),
            Err(LabelError::InvalidValueByte)
        );
        assert_eq!(LabelValue::new("").map(LabelValue::as_str), Ok(""));
    }

    #[test]
    fn validates_label_selectors() {
        assert_eq!(LabelSelector::new(""), Err(LabelError::EmptySelector));
        assert_eq!(
            LabelSelector::new("env=prod\n"),
            Err(LabelError::InvalidSelectorByte)
        );
        assert_eq!(
            LabelSelector::new(",env=prod"),
            Err(LabelError::InvalidSelectorSyntax)
        );
        assert_eq!(
            LabelSelector::new("tier in ()"),
            Err(LabelError::InvalidSelectorSyntax)
        );
        assert_eq!(
            LabelSelector::new("tier in (web,)"),
            Err(LabelError::InvalidSelectorSyntax)
        );
        assert_eq!(
            LabelSelector::new("env=prod;rm"),
            Err(LabelError::InvalidSelectorByte)
        );
        assert_eq!(
            LabelSelector::new("env=prod,tier in (web,api)").map(LabelSelector::as_str),
            Ok("env=prod,tier in (web,api)")
        );
    }
}
