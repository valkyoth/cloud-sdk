//! Bounded duplicate-rejecting JSON admission.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use ::serde::de::{DeserializeSeed, Error as _, MapAccess, SeqAccess, Visitor};
use serde_json::{Map, Number, Value};

pub(super) const MAX_JSON_DEPTH: usize = 64;
pub(super) const MAX_JSON_CONTAINER_ENTRIES: usize = 4096;
pub(super) const MAX_JSON_STRING_BYTES: usize = 1_048_576;

pub(super) fn parse(bytes: &[u8]) -> Result<Value, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(bytes);
    let value = StrictValueSeed { depth: 0 }.deserialize(&mut deserializer)?;
    deserializer.end()?;
    Ok(value)
}

struct StrictValueSeed {
    depth: usize,
}

impl<'de> DeserializeSeed<'de> for StrictValueSeed {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if self.depth > MAX_JSON_DEPTH {
            return Err(D::Error::custom("JSON nesting exceeds the response limit"));
        }
        deserializer.deserialize_any(StrictValueVisitor { depth: self.depth })
    }
}

struct StrictValueVisitor {
    depth: usize,
}

impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = Value;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded JSON value without duplicate object keys")
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(Value::Bool(value))
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
        Ok(Value::String(value))
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
            })?;
            values.insert(key, value);
        }
        Ok(Value::Object(values))
    }
}

fn checked_string(value: &str) -> Option<String> {
    (value.len() <= MAX_JSON_STRING_BYTES).then(|| String::from(value))
}

#[cfg(test)]
mod tests {
    use super::{MAX_JSON_DEPTH, parse};
    use alloc::format;

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
}
