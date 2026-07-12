//! Private Serde wire representations and bounded visitors.

use alloc::borrow::Cow;
use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;
use core::marker::PhantomData;

use ::serde::Deserialize;
use ::serde::de::value::MapAccessDeserializer;
use ::serde::de::{Error as _, MapAccess, SeqAccess, Visitor};

use super::{
    ActionResource, ApiErrorResponse, MAX_ACTION_RESPONSE_RESOURCES, MAX_API_ERROR_CODE_BYTES,
    MAX_API_ERROR_MESSAGE_BYTES, validate_text,
};
use crate::response::ApiErrorCode;

#[derive(Deserialize)]
pub(super) struct ApiErrorEnvelopeWire<'a> {
    #[serde(borrow)]
    pub(super) error: ApiErrorWire<'a>,
}

#[derive(Deserialize)]
pub(super) struct ApiErrorWire<'a> {
    #[serde(borrow)]
    code: Cow<'a, str>,
    #[serde(borrow)]
    message: Cow<'a, str>,
}

impl<'a> ApiErrorWire<'a> {
    pub(super) fn try_into_response<E>(self) -> Result<ApiErrorResponse<'a>, E>
    where
        E: ::serde::de::Error,
    {
        validate_text::<E>(
            self.code.as_ref(),
            MAX_API_ERROR_CODE_BYTES,
            "API error code is invalid",
        )?;
        validate_text::<E>(
            self.message.as_ref(),
            MAX_API_ERROR_MESSAGE_BYTES,
            "API error message is invalid",
        )?;
        Ok(ApiErrorResponse {
            code: ApiErrorCode::from_api_str(self.code.as_ref()),
            message: self.message,
        })
    }
}

#[derive(Deserialize)]
pub(super) struct ActionResourceWire<'a> {
    pub(super) id: u64,
    #[serde(borrow, rename = "type")]
    pub(super) resource_type: Cow<'a, str>,
}

#[derive(Deserialize)]
pub(super) struct ActionEnvelopeWire<'a> {
    #[serde(borrow)]
    pub(super) action: ActionWire<'a>,
}

#[derive(Deserialize)]
pub(super) struct ActionWire<'a> {
    pub(super) id: u64,
    #[serde(borrow)]
    pub(super) command: Cow<'a, str>,
    #[serde(borrow)]
    pub(super) status: Cow<'a, str>,
    pub(super) progress: u8,
    #[serde(borrow)]
    pub(super) started: Cow<'a, str>,
    #[serde(borrow)]
    pub(super) finished: RequiredNullableText<'a>,
    #[serde(borrow)]
    pub(super) resources: ActionResources<'a>,
    #[serde(borrow)]
    pub(super) error: RequiredNullableApiError<'a>,
}

pub(super) struct RequiredNullableText<'a>(pub(super) Option<Cow<'a, str>>);

impl<'de: 'a, 'a> Deserialize<'de> for RequiredNullableText<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RequiredNullableTextVisitor)
    }
}

struct RequiredNullableTextVisitor;

impl<'de> Visitor<'de> for RequiredNullableTextVisitor {
    type Value = RequiredNullableText<'de>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a required nullable string")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(RequiredNullableText(None))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(RequiredNullableText(None))
    }

    fn visit_borrowed_str<E>(self, value: &'de str) -> Result<Self::Value, E> {
        Ok(RequiredNullableText(Some(Cow::Borrowed(value))))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(RequiredNullableText(Some(Cow::Owned(value.into()))))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(RequiredNullableText(Some(Cow::Owned(value))))
    }
}

pub(super) struct RequiredNullableApiError<'a>(pub(super) Option<ApiErrorWire<'a>>);

impl<'de: 'a, 'a> Deserialize<'de> for RequiredNullableApiError<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(RequiredNullableApiErrorVisitor(PhantomData))
    }
}

struct RequiredNullableApiErrorVisitor<'a>(PhantomData<&'a ()>);

impl<'de: 'a, 'a> Visitor<'de> for RequiredNullableApiErrorVisitor<'a> {
    type Value = RequiredNullableApiError<'a>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a required nullable API error")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(RequiredNullableApiError(None))
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(RequiredNullableApiError(None))
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        ApiErrorWire::deserialize(MapAccessDeserializer::new(map))
            .map(Some)
            .map(RequiredNullableApiError)
    }
}

pub(super) struct ActionResources<'a>(pub(super) Vec<ActionResource<'a>>);

impl<'de: 'a, 'a> Deserialize<'de> for ActionResources<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        deserializer.deserialize_seq(ActionResourcesVisitor(PhantomData))
    }
}

struct ActionResourcesVisitor<'a>(PhantomData<&'a ()>);

impl<'de: 'a, 'a> Visitor<'de> for ActionResourcesVisitor<'a> {
    type Value = ActionResources<'a>;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a bounded action resource array")
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let capacity = sequence
            .size_hint()
            .unwrap_or(0)
            .min(MAX_ACTION_RESPONSE_RESOURCES);
        let mut resources = Vec::with_capacity(capacity);
        while let Some(resource) = sequence.next_element()? {
            if resources.len() >= MAX_ACTION_RESPONSE_RESOURCES {
                return Err(A::Error::custom("too many action resources"));
            }
            resources.push(resource);
        }
        Ok(ActionResources(resources))
    }
}
