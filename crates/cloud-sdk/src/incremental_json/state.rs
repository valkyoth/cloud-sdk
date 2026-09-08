//! Private grammar and lexical state.

use alloc::vec::Vec;
use core::cmp::Ordering;

use cloud_sdk_sanitization::{
    SecretString, SecretStringAppendError, sanitize_bytes, try_append_secret_string,
};

pub(super) enum Terminal {
    Active,
    Complete,
    Stopped,
    Failed,
}

pub(super) enum DocumentPhase {
    Value,
    Trailing,
}

pub(super) enum Frame {
    Array(ArrayPhase),
    Object(ObjectFrame),
}

pub(super) enum ArrayPhase {
    ValueOrEnd,
    Value,
    CommaOrEnd,
}

pub(super) struct ObjectFrame {
    pub(super) phase: ObjectPhase,
    pub(super) fields: usize,
    pub(super) keys: Vec<IncrementalKey>,
}

pub(super) enum ObjectPhase {
    KeyOrEnd,
    Key,
    Colon,
    Value,
    CommaOrEnd,
}

pub(super) enum Lexical {
    Literal(Literal),
    Number(Number),
    String(JsonString),
}

pub(super) struct Literal {
    pub(super) expected: &'static [u8],
    pub(super) position: usize,
    pub(super) value: LiteralValue,
}

pub(super) enum LiteralValue {
    Null,
    Bool(bool),
}

pub(super) struct Number {
    pub(super) text: SecretString,
    pub(super) phase: NumberPhase,
    pub(super) exponent_digits: usize,
}

pub(super) enum NumberPhase {
    Minus,
    Zero,
    Integer,
    Dot,
    Fraction,
    Exponent,
    ExponentSign,
    ExponentDigits,
}

impl NumberPhase {
    pub(super) const fn complete(&self) -> bool {
        matches!(
            self,
            Self::Zero | Self::Integer | Self::Fraction | Self::ExponentDigits
        )
    }
}

pub(super) struct JsonString {
    pub(super) kind: StringKind,
    pub(super) mode: StringMode,
    pub(super) decoded_bytes: usize,
    pub(super) utf8: [u8; 4],
    pub(super) utf8_len: usize,
    pub(super) utf8_expected: usize,
    pub(super) key: Option<IncrementalKey>,
}

pub(super) struct IncrementalKey(SecretString);

impl IncrementalKey {
    pub(super) fn try_with_capacity(capacity: usize) -> Result<Self, SecretStringAppendError> {
        SecretString::try_with_capacity(capacity)
            .map(Self)
            .map_err(|_| SecretStringAppendError::Allocation)
    }

    pub(super) fn try_push_str(
        &mut self,
        value: &str,
        maximum_bytes: usize,
    ) -> Result<(), SecretStringAppendError> {
        try_append_secret_string(&mut self.0, value, maximum_bytes)
    }

    pub(super) fn try_with_str<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<R, core::str::Utf8Error> {
        self.0.try_with_secret(inspect)
    }
}

impl PartialEq for IncrementalKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for IncrementalKey {}

impl PartialOrd for IncrementalKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IncrementalKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0
            .with_secret_bytes(|left| other.0.with_secret_bytes(|right| left.cmp(right)))
    }
}

impl Drop for JsonString {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.utf8);
    }
}

pub(super) enum StringKind {
    Key,
    Value,
}

pub(super) enum StringMode {
    Normal,
    Escape,
    Unicode { value: u16, digits: u8 },
    LowSlash { high: u16 },
    LowU { high: u16 },
    LowUnicode { high: u16, value: u16, digits: u8 },
}
