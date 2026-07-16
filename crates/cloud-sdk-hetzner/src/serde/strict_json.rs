//! Bounded duplicate-rejecting JSON admission with protected string storage.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use cloud_sdk_sanitization::SecretString;

mod parser;

pub(super) const MAX_JSON_DEPTH: usize = 64;
pub(super) const MAX_JSON_CONTAINER_ENTRIES: usize = 4096;
pub(super) const MAX_JSON_NODES: usize = 65_536;
pub(super) const MAX_JSON_STRING_BYTES: usize = 1_048_576;

pub(super) type Map = BTreeMap<String, Value>;

pub(super) enum Number {
    Unsigned(u64),
    Signed(i64),
    Float(f64),
}

/// Private parser tree whose string values clear their full allocation on drop.
pub(super) enum Value {
    Null,
    Bool,
    Number(Number),
    String(SecretString),
    Array(Vec<Self>),
    Object(Map),
}

impl Value {
    pub(super) const fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub(super) fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Number(Number::Unsigned(value)) => Some(*value),
            Self::Number(Number::Signed(value)) => u64::try_from(*value).ok(),
            _ => None,
        }
    }

    pub(super) fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(Number::Unsigned(value)) => Some(*value as f64),
            Self::Number(Number::Signed(value)) => Some(*value as f64),
            Self::Number(Number::Float(value)) => Some(*value),
            _ => None,
        }
    }

    pub(super) fn as_array(&self) -> Option<&[Self]> {
        match self {
            Self::Array(values) => Some(values),
            _ => None,
        }
    }

    pub(super) fn as_object(&self) -> Option<&Map> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    pub(super) fn as_object_mut(&mut self) -> Option<&mut Map> {
        match self {
            Self::Object(values) => Some(values),
            _ => None,
        }
    }

    pub(super) fn try_with_str<R>(
        &self,
        inspect: impl FnOnce(&str) -> R,
    ) -> Result<Option<R>, core::str::Utf8Error> {
        match self {
            Self::String(value) => value.try_with_secret(inspect).map(Some),
            _ => Ok(None),
        }
    }

    pub(super) fn take_string(&mut self) -> Option<SecretString> {
        let value = core::mem::replace(self, Self::Null);
        match value {
            Self::String(value) => Some(value),
            other => {
                *self = other;
                None
            }
        }
    }
}

pub(super) fn parse(bytes: &[u8]) -> Result<Value, parser::JsonError> {
    parser::parse(bytes)
}

#[cfg(test)]
mod tests {
    use super::{MAX_JSON_DEPTH, MAX_JSON_NODES, Value, parse};
    use alloc::format;
    use alloc::string::String;

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
}
