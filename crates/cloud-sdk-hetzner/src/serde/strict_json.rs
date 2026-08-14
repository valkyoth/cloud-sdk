//! Bounded duplicate-rejecting JSON admission with protected string storage.

use alloc::string::String;
use alloc::vec::Vec;
use core::borrow::Borrow;

use cloud_sdk_sanitization::{SecretString, sanitize_string};

mod parser;
mod protected;
pub(crate) use parser::JsonError;
use protected::ProtectedBoolean;

#[cfg(test)]
mod allocation_failure;
#[cfg(test)]
pub(crate) use allocation_failure::with_next_failure;

pub(super) const MAX_JSON_DEPTH: usize = 64;
pub(super) const MAX_JSON_CONTAINER_ENTRIES: usize = 4096;
pub(super) const MAX_JSON_OBJECT_FIELDS: usize = 256;
pub(super) const MAX_JSON_NODES: usize = 65_536;
pub(super) const MAX_JSON_STRING_BYTES: usize = 1_048_576;
pub(super) const MAX_JSON_NUMBER_BYTES: usize = 128;

pub(crate) struct Map(Vec<(ProtectedKey, Value)>);

impl Map {
    pub(super) const fn new() -> Self {
        Self(Vec::new())
    }

    pub(super) fn try_reserve(&mut self, additional: usize) -> Result<(), ()> {
        self.0.try_reserve(additional).map_err(|_| ())
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(super) fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }

    pub(super) fn insert_reserved(&mut self, key: ProtectedKey, value: Value) {
        self.0.push((key, value));
    }

    pub(super) fn finish(mut self) -> Result<Self, JsonError> {
        self.0
            .sort_unstable_by(|left, right| left.0.as_str().cmp(right.0.as_str()));
        if self.0.windows(2).any(|pair| match pair {
            [(left, _), (right, _)] => left.as_str() == right.as_str(),
            _ => false,
        }) {
            return Err(JsonError::DuplicateKey);
        }
        Ok(self)
    }

    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        self.0
            .binary_search_by(|(candidate, _)| candidate.as_str().cmp(key))
            .ok()
            .and_then(|index| self.0.get(index))
            .map(|(_, value)| value)
    }

    pub(crate) fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        let index = self
            .0
            .binary_search_by(|(candidate, _)| candidate.as_str().cmp(key))
            .ok()?;
        self.0.get_mut(index).map(|(_, value)| value)
    }

    pub(super) fn iter(&self) -> impl Iterator<Item = (&ProtectedKey, &Value)> {
        self.0.iter().map(|(key, value)| (key, value))
    }

    pub(super) fn iter_mut(&mut self) -> impl Iterator<Item = (&ProtectedKey, &mut Value)> {
        self.0.iter_mut().map(|(key, value)| (&*key, value))
    }

    pub(crate) fn try_for_each<E>(
        &self,
        mut inspect: impl FnMut(&str, &Value) -> Result<(), E>,
    ) -> Result<(), E> {
        for (key, value) in &self.0 {
            inspect(key.as_str(), value)?;
        }
        Ok(())
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
pub(super) struct ProtectedKey(String);

impl ProtectedKey {
    pub(super) fn try_with_capacity(capacity: usize) -> Result<Self, ()> {
        let mut value = String::new();
        value.try_reserve_exact(capacity).map_err(|_| ())?;
        Ok(Self(value))
    }

    pub(super) fn push_str(&mut self, value: &str) {
        self.0.push_str(value);
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }
}

impl Borrow<str> for ProtectedKey {
    fn borrow(&self) -> &str {
        &self.0
    }
}

impl Drop for ProtectedKey {
    fn drop(&mut self) {
        sanitize_string(&mut self.0);
    }
}

pub(crate) enum Number {
    Unsigned(LexicalNumber),
    Signed(LexicalNumber),
    Float(LexicalNumber),
}

pub(crate) struct LexicalNumber {
    lexical: String,
}

impl LexicalNumber {
    pub(super) fn try_new(lexical: &str) -> Result<Self, ()> {
        let mut owned = String::new();
        owned.try_reserve_exact(lexical.len()).map_err(|_| ())?;
        owned.push_str(lexical);
        Ok(Self { lexical: owned })
    }

    pub(super) fn into_lexical(mut self) -> String {
        core::mem::take(&mut self.lexical)
    }

    pub(super) fn as_str(&self) -> &str {
        &self.lexical
    }
}

impl Drop for LexicalNumber {
    fn drop(&mut self) {
        sanitize_string(&mut self.lexical);
    }
}

/// Private parser tree whose string values clear their full allocation on drop.
pub(crate) enum Value {
    Null,
    Bool(ProtectedBoolean),
    Number(Number),
    String(SecretString),
    Array(Vec<Self>),
    Object(Map),
}

impl Value {
    pub(crate) const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub(crate) fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(Number::Unsigned(value)) => value.as_str().parse().ok(),
            Self::Number(Number::Signed(value)) => value
                .as_str()
                .parse::<i64>()
                .ok()
                .and_then(|value| u64::try_from(value).ok()),
            _ => None,
        }
    }

    pub(crate) fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Number(Number::Unsigned(value) | Number::Signed(value)) => {
                value.as_str().parse().ok()
            }
            _ => None,
        }
    }

    pub(super) const fn is_integer(&self) -> bool {
        matches!(self, Self::Number(Number::Unsigned(_) | Number::Signed(_)))
    }

    pub(super) const fn is_number(&self) -> bool {
        matches!(self, Self::Number(_))
    }

    pub(super) const fn is_string(&self) -> bool {
        matches!(self, Self::String(_))
    }

    pub(super) fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(
                Number::Unsigned(value) | Number::Signed(value) | Number::Float(value),
            ) => value.as_str().parse().ok(),
            _ => None,
        }
    }

    pub(crate) fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(value) => Some(value.value()),
            _ => None,
        }
    }

    pub(crate) fn copy_bool_byte_to(&self, destination: &mut u8) -> Option<()> {
        match self {
            Self::Bool(value) => {
                value.copy_byte_to(destination);
                Some(())
            }
            _ => None,
        }
    }

    pub(crate) const fn is_bool(&self) -> bool {
        matches!(self, Self::Bool(_))
    }

    pub(crate) fn try_with_unsigned_lexical<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Option<R> {
        match self {
            Self::Number(Number::Unsigned(value)) => Some(inspect(value.as_str())),
            _ => None,
        }
    }

    pub(crate) fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub(super) fn as_array_mut(&mut self) -> Option<&mut [Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub(crate) fn as_object(&self) -> Option<&Map> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    pub(crate) fn as_object_mut(&mut self) -> Option<&mut Map> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    pub(crate) fn try_with_str<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<Option<R>, core::str::Utf8Error> {
        match self {
            Self::String(value) => value.try_with_secret(inspect).map(Some),
            _ => Ok(None),
        }
    }

    pub(crate) fn take_string(&mut self) -> Option<SecretString> {
        let value = core::mem::replace(self, Self::Null);
        match value {
            Self::String(value) => Some(value),
            other => {
                *self = other;
                None
            }
        }
    }

    pub(super) fn take_number_lexical(&mut self) -> Option<String> {
        let value = core::mem::replace(self, Self::Null);
        match value {
            Self::Number(Number::Unsigned(value)) => Some(value.into_lexical()),
            Self::Number(Number::Signed(value)) => Some(value.into_lexical()),
            Self::Number(Number::Float(value)) => Some(value.into_lexical()),
            other => {
                *self = other;
                None
            }
        }
    }

    pub(crate) fn take_array(&mut self) -> Option<Vec<Self>> {
        let value = core::mem::replace(self, Self::Null);
        match value {
            Self::Array(value) => Some(value),
            other => {
                *self = other;
                None
            }
        }
    }
}

#[cfg(test)]
pub(super) fn parse(bytes: &[u8]) -> Result<Value, parser::JsonError> {
    parser::parse(bytes)
}

pub(crate) fn parse_with_scratch(
    bytes: &[u8],
    scratch: &mut [u8],
) -> Result<Value, parser::JsonError> {
    parser::parse_with_scratch(bytes, scratch)
}

#[cfg(test)]
mod tests {
    use super::{
        JsonError, MAX_JSON_DEPTH, MAX_JSON_NODES, MAX_JSON_OBJECT_FIELDS, ProtectedKey, Value,
        parse,
    };
    use alloc::format;
    use alloc::string::String;
    use cloud_sdk_sanitization::sanitize_string;

    #[test]
    fn rejects_duplicates_trailing_documents_and_excessive_depth() {
        assert!(parse(br#"{"id":1,"id":2}"#).is_err());
        assert!(parse(br#"{} {}"#).is_err());
        let nested = format!(
            "{}0{}",
            "[".repeat(MAX_JSON_DEPTH.saturating_add(2)),
            "]".repeat(MAX_JSON_DEPTH.saturating_add(2))
        );
        assert!(parse(nested.as_bytes()).is_err());
        assert!(parse(br#"{"id":1,"future":true}"#).is_ok());
    }

    #[test]
    fn object_fields_are_sorted_for_lookup_and_source_bounded() {
        let parsed = parse(br#"{"z":1,"middle":2,"a":3}"#);
        let Ok(Value::Object(fields)) = parsed else {
            unreachable!("bounded object fixture failed")
        };
        assert_eq!(fields.get("a").and_then(Value::as_u64), Some(3));
        assert_eq!(fields.get("middle").and_then(Value::as_u64), Some(2));
        assert_eq!(fields.get("z").and_then(Value::as_u64), Some(1));

        let mut input = String::from("{");
        for index in 0..MAX_JSON_OBJECT_FIELDS {
            if index != 0 {
                input.push(',');
            }
            input.push_str(&format!("\"field{index:03}\":null"));
        }
        input.push('}');
        assert!(parse(input.as_bytes()).is_ok());
        input.pop();
        input.push_str(",\"overflow\":null}");
        assert_eq!(
            parse(input.as_bytes()).err(),
            Some(JsonError::ContainerLimit)
        );
        sanitize_string(&mut input);
    }

    #[test]
    fn rejects_aggregate_nodes_below_container_and_wire_limits() {
        let mut input = String::from("[");
        let inner =
            "[null,null,null,null,null,null,null,null,null,null,null,null,null,null,null,null]";
        let containers = MAX_JSON_NODES / 17 + 1;
        for index in 0..containers {
            if index != 0 {
                input.push(',');
            }
            input.push_str(inner);
        }
        input.push(']');

        assert!(containers < super::MAX_JSON_CONTAINER_ENTRIES);
        assert!(input.len() < 1_000_000);
        assert!(parse(input.as_bytes()).is_err());
    }

    #[test]
    fn parser_strings_use_protected_storage_and_can_move_without_copying()
    -> Result<(), &'static str> {
        let mut parsed =
            parse(br#""temporary secret""#).map_err(|_| "protected parser string was rejected")?;
        let before = match &parsed {
            Value::String(secret) => secret.with_secret_bytes(|bytes| bytes.as_ptr()),
            _ => return Err("JSON string did not use protected storage"),
        };
        let secret = parsed
            .take_string()
            .ok_or("protected parser string was not movable")?;
        let after = secret.with_secret_bytes(|bytes| bytes.as_ptr());

        assert_eq!(before, after);
        assert_eq!(
            secret.try_with_secret(|value| value == "temporary secret"),
            Ok(true)
        );
        Ok(())
    }

    #[test]
    fn decodes_escaped_strings_directly_into_protected_storage() -> Result<(), &'static str> {
        let mut parsed = parse(br#""line\nquote: \" snowman: \u2603 music: \uD834\uDD1E""#)
            .map_err(|_| "escaped protected parser string was rejected")?;
        let secret = parsed
            .take_string()
            .ok_or("escaped JSON string did not use protected storage")?;

        assert_eq!(
            secret.try_with_secret(|value| value == "line\nquote: \" snowman: ☃ music: 𝄞"),
            Ok(true)
        );
        Ok(())
    }

    #[test]
    fn accepts_complete_json_grammar_and_rejects_malformed_boundaries() {
        for valid in [
            br#"{"key":[null,true,false,-0,0,1.25,6.02e23,"text","\u0000","\uD834\uDD1E"]}"#
                .as_slice(),
            br#"{"escaped\u0020key":"snowman: \u2603"}"#,
            br#"18446744073709551616"#,
            br#"-9223372036854775809"#,
            " \"é\" \n".as_bytes(),
        ] {
            assert!(parse(valid).is_ok());
        }

        for invalid in [
            b"".as_slice(),
            b"+1",
            b".1",
            b"01",
            b"-01",
            b"1.",
            b"1e",
            b"1e+",
            b"NaN",
            b"1e400",
            br#""\x""#,
            br#""\uD800""#,
            br#""\uDC00""#,
            br#""\uD800\u0041""#,
            b"\"raw\ncontrol\"",
            b"[1,]",
            b"{\"key\":1,}",
        ] {
            assert!(parse(invalid).is_err());
        }
    }

    #[test]
    fn object_keys_use_capacity_wiping_storage() {
        let mut key = ProtectedKey::try_with_capacity(32)
            .unwrap_or_else(|()| unreachable!("test key allocation failed"));
        key.push_str("potentially sensitive key");
        assert_eq!(key.0.capacity(), 32);
        sanitize_string(&mut key.0);
        assert!(key.0.is_empty());
        assert_eq!(key.0.capacity(), 32);
    }
}
