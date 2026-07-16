//! Bounded duplicate-rejecting JSON admission with protected string storage.

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;
use core::cell::Cell;
use core::fmt;

use ::serde::de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use cloud_sdk_sanitization::SecretString;
use serde_json::Number;

pub(super) const MAX_JSON_DEPTH: usize = 64;
pub(super) const MAX_JSON_CONTAINER_ENTRIES: usize = 4096;
pub(super) const MAX_JSON_NODES: usize = 65_536;
pub(super) const MAX_JSON_STRING_BYTES: usize = 1_048_576;

pub(super) type Map = BTreeMap<String, Value>;

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
            Self::Number(value) => value.as_u64(),
            _ => None,
        }
    }

    pub(super) fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Number(value) => value.as_f64(),
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

pub(super) fn parse(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let budget = ParseBudget::new();
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictValueSeed {
        depth: 0,
        budget: &budget,
    }
    .deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

struct ParseBudget {
    nodes: Cell<usize>,
}

impl ParseBudget {
    const fn new() -> Self {
        Self {
            nodes: Cell::new(0),
        }
    }

    fn charge<E: ::serde::de::Error>(&self) -> Result<(), E> {
        let next = self
            .nodes
            .get()
            .checked_add(1)
            .ok_or_else(|| E::custom("JSON node budget overflow"))?;
        if next > MAX_JSON_NODES {
            return Err(E::custom("JSON aggregate node limit exceeded"));
        }
        self.nodes.set(next);
        Ok(())
    }
}

struct StrictValueSeed<'a> {
    depth: usize,
    budget: &'a ParseBudget,
}

impl<'de> DeserializeSeed<'de> for StrictValueSeed<'_> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        self.budget.charge::<D::Error>()?;
        if self.depth > MAX_JSON_DEPTH {
            return Err(D::Error::custom("JSON nesting exceeds the response limit"));
        }
        deserializer.deserialize_any(StrictValueVisitor {
            depth: self.depth,
            budget: self.budget,
        })
    }
}

struct StrictValueVisitor<'a> {
    depth: usize,
    budget: &'a ParseBudget,
}

impl<'de> Visitor<'de> for StrictValueVisitor<'_> {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, _value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool)
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(Value::Number(Number::from(value)))
    }

    fn visit_f64<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: ::serde::de::Error,
    {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or_else(|| E::custom("JSON number is not finite"))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(Value::Null)
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E>
    where
        E: ::serde::de::Error,
    {
        checked_string(value)
            .map(Value::String)
            .ok_or_else(|| E::custom("JSON string exceeds the response limit"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: ::serde::de::Error,
    {
        self.visit_borrowed_str(value)
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
    where
        E: ::serde::de::Error,
    {
        if value.len() > MAX_JSON_STRING_BYTES {
            return Err(E::custom("JSON string exceeds the response limit"));
        }
        Ok(Value::String(SecretString::from_string(value)))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::with_capacity(
            sequence
                .size_hint()
                .unwrap_or(0)
                .min(MAX_JSON_CONTAINER_ENTRIES),
        );
        while let Some(value) = sequence.next_element_seed(StrictValueSeed {
            depth: self.depth.saturating_add(1),
            budget: self.budget,
        })? {
            if values.len() >= MAX_JSON_CONTAINER_ENTRIES {
                return Err(A::Error::custom("JSON array exceeds the response limit"));
            }
            values.push(value);
        }
        Ok(Value::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = Map::new();
        while let Some(key) = map.next_key::<String>()? {
            if key.len() > MAX_JSON_STRING_BYTES {
                return Err(A::Error::custom(
                    "JSON object key exceeds the response limit",
                ));
            }
            if values.len() >= MAX_JSON_CONTAINER_ENTRIES {
                return Err(A::Error::custom("JSON object exceeds the response limit"));
            }
            if values.contains_key(&key) {
                return Err(A::Error::custom("JSON object contains a duplicate key"));
            }
            let value = map.next_value_seed(StrictValueSeed {
                depth: self.depth.saturating_add(1),
                budget: self.budget,
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

fn checked_string(value: &str) -> Option<SecretString> {
    (value.len() <= MAX_JSON_STRING_BYTES).then(|| SecretString::from_secret_str(value))
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
    fn parser_strings_use_protected_storage_and_can_move_without_copying() {
        let parsed = parse(br#""temporary secret""#);
        let Ok(mut parsed) = parsed else {
            panic!("protected parser string must be accepted");
        };
        let Value::String(secret) = &parsed else {
            panic!("JSON string must use protected storage");
        };
        let before = secret.with_secret_bytes(|bytes| bytes.as_ptr());
        let Some(secret) = parsed.take_string() else {
            panic!("protected parser string must remain movable");
        };
        let after = secret.with_secret_bytes(|bytes| bytes.as_ptr());

        assert_eq!(before, after);
        assert_eq!(
            secret.try_with_secret(|value| value == "temporary secret"),
            Ok(true)
        );
    }
}
