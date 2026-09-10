use super::{DiscoveryError as Error, MAX_DISCOVERY_ITEMS};
use alloc::{string::String, vec::Vec};
use cloud_sdk::incremental_json::{
    IncrementalJsonEvent as Event, IncrementalJsonVisitor, VisitControl,
};
use cloud_sdk_sanitization::{SecretString, try_append_secret_string};

enum Kind {
    Null,
    Bool(bool),
    Number(SecretString),
    Text(SecretString),
    Array(Vec<DiscoveryValue>),
    Object(Vec<(SecretString, DiscoveryValue)>),
}

/// Bounded JSON retained only for schema-open badge objects.
/// Text and number inspection is closure-scoped; Debug never prints payloads.
/// It is data, not a request, URL, credential or authorization capability.
pub struct DiscoveryValue(Kind);
impl core::fmt::Debug for DiscoveryValue {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("DiscoveryValue([redacted])")
    }
}
impl DiscoveryValue {
    /// Reports explicit JSON null.
    #[must_use]
    pub const fn is_null(&self) -> bool {
        matches!(self.0, Kind::Null)
    }
    /// Returns a boolean, without numeric coercion.
    pub fn boolean(&self) -> Result<bool, Error> {
        match self.0 {
            Kind::Bool(value) => Ok(value),
            _ => Err(Error::Schema),
        }
    }
    /// Inspects protected string text.
    pub fn with_text<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, Error> {
        match &self.0 {
            Kind::Text(value) => value.try_with_secret(inspect).map_err(|_| Error::Value),
            _ => Err(Error::Schema),
        }
    }
    /// Inspects exact number spelling, without floating-point precision loss.
    pub fn with_number<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, Error> {
        match &self.0 {
            Kind::Number(value) => value.try_with_secret(inspect).map_err(|_| Error::Value),
            _ => Err(Error::Schema),
        }
    }
    /// Returns an array, or a schema error for another type.
    pub fn array(&self) -> Result<&[Self], Error> {
        match &self.0 {
            Kind::Array(values) => Ok(values),
            _ => Err(Error::Schema),
        }
    }
    /// Finds one exact object field, preserving missing versus explicit null.
    pub fn get(&self, name: &str) -> Result<Option<&Self>, Error> {
        let Kind::Object(fields) = &self.0 else {
            return Err(Error::Schema);
        };
        for (key, value) in fields {
            if key
                .try_with_secret(|key| key == name)
                .map_err(|_| Error::Value)?
            {
                return Ok(Some(value));
            }
        }
        Ok(None)
    }
    pub(super) fn required(&self, name: &str) -> Result<&Self, Error> {
        self.get(name)?.ok_or(Error::Schema)
    }
    pub(super) fn object(&self) -> Result<(), Error> {
        if matches!(self.0, Kind::Object(_)) {
            Ok(())
        } else {
            Err(Error::Schema)
        }
    }
    pub(super) fn text(&self, maximum: usize) -> Result<String, Error> {
        self.with_text(|text| {
            if text.len() > maximum {
                return Err(Error::Limit);
            }
            let mut out = String::new();
            out.try_reserve_exact(text.len())
                .map_err(|_| Error::Allocation)?;
            out.push_str(text);
            Ok(out)
        })?
    }
    pub(super) fn count(&self, maximum: u64) -> Result<u64, Error> {
        self.with_number(|text| {
            if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit()) {
                return Err(Error::Value);
            }
            text.parse::<u64>()
                .ok()
                .filter(|n| *n <= maximum)
                .ok_or(Error::Value)
        })?
    }
    pub(super) fn take(&mut self, name: &str) -> Result<Self, Error> {
        let Kind::Object(fields) = &mut self.0 else {
            return Err(Error::Schema);
        };
        for (key, value) in fields {
            if key
                .try_with_secret(|key| key == name)
                .map_err(|_| Error::Value)?
            {
                return Ok(core::mem::replace(value, Self(Kind::Null)));
            }
        }
        Err(Error::Schema)
    }
    pub(super) fn into_array(self) -> Result<Vec<Self>, Error> {
        match self.0 {
            Kind::Array(values) => Ok(values),
            _ => Err(Error::Schema),
        }
    }
}

struct Frame {
    value: DiscoveryValue,
    key: Option<SecretString>,
}
#[derive(Default)]
pub(super) struct Builder {
    stack: Vec<Frame>,
    root: Option<DiscoveryValue>,
    text: Option<SecretString>,
    nodes: usize,
}
impl Builder {
    pub(super) fn finish(mut self) -> Result<DiscoveryValue, Error> {
        if !self.stack.is_empty() || self.text.is_some() {
            return Err(Error::Json);
        }
        self.root.take().ok_or(Error::Json)
    }
    fn add(&mut self, value: DiscoveryValue) -> Result<(), Error> {
        self.nodes = self
            .nodes
            .checked_add(1)
            .filter(|v| *v <= 16_384)
            .ok_or(Error::Limit)?;
        if let Some(frame) = self.stack.last_mut() {
            match &mut frame.value.0 {
                Kind::Array(values) => push(values, value, MAX_DISCOVERY_ITEMS),
                Kind::Object(values) => {
                    let key = frame.key.take().ok_or(Error::Json)?;
                    push(values, (key, value), 64)
                }
                _ => Err(Error::Json),
            }
        } else if self.root.is_none() {
            self.root = Some(value);
            Ok(())
        } else {
            Err(Error::Json)
        }
    }
}
pub(super) fn push<T>(values: &mut Vec<T>, value: T, maximum: usize) -> Result<(), Error> {
    if values.len() >= maximum {
        return Err(Error::Limit);
    }
    values.try_reserve(1).map_err(|_| Error::Allocation)?;
    values.push(value);
    Ok(())
}
fn protected(text: &str, maximum: usize) -> Result<SecretString, Error> {
    let mut value = SecretString::empty();
    append(&mut value, text, maximum)?;
    Ok(value)
}
fn append(value: &mut SecretString, text: &str, maximum: usize) -> Result<(), Error> {
    use cloud_sdk_sanitization::SecretStringAppendError as E;
    try_append_secret_string(value, text, maximum).map_err(|error| match error {
        E::Allocation => Error::Allocation,
        E::InvalidUtf8 => Error::Value,
        _ => Error::Limit,
    })
}
impl IncrementalJsonVisitor for Builder {
    type Error = Error;
    fn visit(&mut self, event: Event<'_>) -> Result<VisitControl, Error> {
        match event {
            Event::StartObject | Event::StartArray => {
                let value = DiscoveryValue(if event == Event::StartObject {
                    Kind::Object(Vec::new())
                } else {
                    Kind::Array(Vec::new())
                });
                push(&mut self.stack, Frame { value, key: None }, 32)?;
            }
            Event::EndObject | Event::EndArray => {
                let frame = self.stack.pop().ok_or(Error::Json)?;
                if frame.key.is_some() {
                    return Err(Error::Json);
                }
                self.add(frame.value)?;
            }
            Event::Key(key) => {
                let frame = self.stack.last_mut().ok_or(Error::Json)?;
                if frame.key.is_some() {
                    return Err(Error::Json);
                }
                frame.key = Some(protected(key, 256)?);
            }
            Event::StringStart => {
                self.text = Some(SecretString::empty());
            }
            Event::StringFragment(text) => {
                append(self.text.as_mut().ok_or(Error::Json)?, text, 65_536)?
            }
            Event::StringEnd => {
                let text = self.text.take().ok_or(Error::Json)?;
                self.add(DiscoveryValue(Kind::Text(text)))?;
            }
            Event::Number(text) => self.add(DiscoveryValue(Kind::Number(protected(text, 256)?)))?,
            Event::Bool(value) => self.add(DiscoveryValue(Kind::Bool(value)))?,
            Event::Null => self.add(DiscoveryValue(Kind::Null))?,
            _ => return Err(Error::Json),
        }
        Ok(VisitControl::Continue)
    }
}
