//! Strict deserialization for shared pagination metadata.

use core::fmt;

use ::serde::Deserialize;
use ::serde::de::{Error as _, Visitor};

use crate::pagination::{Page, PaginationMetadata, PerPage};

/// Response envelope that extracts validated `meta.pagination` fields while
/// ignoring resource-specific and additive fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationEnvelope {
    pagination: PaginationMetadata,
}

impl PaginationEnvelope {
    /// Returns validated Hetzner pagination metadata.
    #[must_use]
    pub const fn pagination(self) -> PaginationMetadata {
        self.pagination
    }
}

impl<'de> Deserialize<'de> for PaginationEnvelope {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        let wire = EnvelopeWire::deserialize(deserializer)?;
        let page = Page::new(wire.meta.pagination.page)
            .map_err(|_| D::Error::custom("pagination page is invalid"))?;
        let per_page = u16::try_from(wire.meta.pagination.per_page)
            .ok()
            .and_then(|value| PerPage::new(value).ok())
            .ok_or_else(|| D::Error::custom("pagination per_page is invalid"))?;
        let previous = optional_page::<D::Error>(wire.meta.pagination.previous_page.0)?;
        let next = optional_page::<D::Error>(wire.meta.pagination.next_page.0)?;
        let last = optional_page::<D::Error>(wire.meta.pagination.last_page.0)?;
        let pagination = PaginationMetadata::new(
            page,
            per_page,
            previous,
            next,
            last,
            wire.meta.pagination.total_entries.0,
        )
        .map_err(|_| D::Error::custom("pagination navigation is invalid"))?;
        Ok(Self { pagination })
    }
}

fn optional_page<E>(value: Option<u64>) -> Result<Option<Page>, E>
where
    E: ::serde::de::Error,
{
    value
        .map(|value| Page::new(value).map_err(|_| E::custom("pagination page is invalid")))
        .transpose()
}

#[derive(Deserialize)]
struct EnvelopeWire {
    meta: MetaWire,
}

#[derive(Deserialize)]
struct MetaWire {
    pagination: PaginationWire,
}

#[derive(Deserialize)]
struct PaginationWire {
    page: u64,
    per_page: u64,
    previous_page: RequiredNullableU64,
    next_page: RequiredNullableU64,
    last_page: RequiredNullableU64,
    total_entries: RequiredNullableU64,
}

struct RequiredNullableU64(Option<u64>);

impl<'de> Deserialize<'de> for RequiredNullableU64 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RequiredNullableU64Visitor)
    }
}

struct RequiredNullableU64Visitor;

impl<'de> Visitor<'de> for RequiredNullableU64Visitor {
    type Value = RequiredNullableU64;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a required nullable unsigned integer")
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(RequiredNullableU64(None))
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(RequiredNullableU64(None))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(RequiredNullableU64(Some(value)))
    }
}

#[cfg(test)]
mod tests {
    use super::PaginationEnvelope;
    use crate::pagination::Page;

    const VALID: &str = r#"{
        "servers":[],"additive":true,
        "meta":{"additive":true,"pagination":{
            "page":2,"per_page":25,"previous_page":1,"next_page":3,
            "last_page":4,"total_entries":100,"additive":true
        }}
    }"#;

    #[test]
    fn parses_coherent_required_metadata_and_ignores_additive_fields() {
        let parsed = serde_json::from_str::<PaginationEnvelope>(VALID);
        assert!(parsed.is_ok());
        let Ok(parsed) = parsed else { return };
        assert_eq!(parsed.pagination().page().get(), 2);
        assert_eq!(parsed.pagination().next_page().map(Page::get), Some(3));
        assert_eq!(parsed.pagination().as_core().total_entries(), Some(100));
    }

    #[test]
    fn rejects_missing_nullability_bounds_and_navigation_ambiguity() {
        for invalid in [
            r#"{"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1}}}"#,
            r#"{"meta":{"pagination":{"page":1,"per_page":51,"previous_page":null,"next_page":null,"last_page":1,"total_entries":1}}}"#,
            r#"{"meta":{"pagination":{"page":2,"per_page":25,"previous_page":1,"next_page":2,"last_page":4,"total_entries":100}}}"#,
            r#"{"meta":{"pagination":{"page":2,"per_page":25,"previous_page":1,"next_page":4,"last_page":4,"total_entries":100}}}"#,
            r#"{"meta":{"pagination":{"page":2,"per_page":25,"previous_page":0,"next_page":3,"last_page":4,"total_entries":100}}}"#,
            r#"{"meta":{"pagination":{"page":3,"per_page":25,"previous_page":1,"next_page":4,"last_page":4,"total_entries":100}}}"#,
            r#"{"meta":{"pagination":{"page":1,"per_page":25,"previous_page":null,"next_page":null,"last_page":1,"total_entries":-1}}}"#,
        ] {
            assert!(
                serde_json::from_str::<PaginationEnvelope>(invalid).is_err(),
                "accepted invalid pagination: {invalid}"
            );
        }
    }

    #[test]
    fn permits_documented_nullable_navigation_fields() {
        let terminal = VALID
            .replace("\"page\":2", "\"page\":1")
            .replace("\"previous_page\":1", "\"previous_page\":null")
            .replace("\"next_page\":3", "\"next_page\":null")
            .replace("\"last_page\":4", "\"last_page\":null")
            .replace("\"total_entries\":100", "\"total_entries\":null");
        assert!(serde_json::from_str::<PaginationEnvelope>(&terminal).is_ok());
    }
}
