//! Allocation-free JSON request-body writing.

use cloud_sdk::buffer;

use crate::cloud::shared::CloudLabels;
use crate::labels::{LabelKey, LabelValue};

use super::HetznerPreparationError;

const BODY_ERROR: HetznerPreparationError = HetznerPreparationError::Body;

/// Small JSON token writer over one caller-owned output buffer.
pub(crate) struct JsonWriter<'a> {
    output: &'a mut [u8],
    len: usize,
}

impl<'a> JsonWriter<'a> {
    pub(crate) fn new(output: &'a mut [u8]) -> Self {
        Self { output, len: 0 }
    }

    pub(crate) const fn len(&self) -> usize {
        self.len
    }

    pub(crate) fn begin_object(&mut self) -> Result<(), HetznerPreparationError> {
        self.byte(b'{')
    }

    pub(crate) fn end_object(&mut self) -> Result<(), HetznerPreparationError> {
        self.byte(b'}')
    }

    pub(crate) fn begin_array(&mut self) -> Result<(), HetznerPreparationError> {
        self.byte(b'[')
    }

    pub(crate) fn end_array(&mut self) -> Result<(), HetznerPreparationError> {
        self.byte(b']')
    }

    pub(crate) fn field(
        &mut self,
        first: &mut bool,
        name: &str,
    ) -> Result<(), HetznerPreparationError> {
        self.separator(first)?;
        self.string(name)?;
        self.byte(b':')
    }

    pub(crate) fn value(&mut self, first: &mut bool) -> Result<(), HetznerPreparationError> {
        self.separator(first)
    }

    pub(crate) fn string(&mut self, value: &str) -> Result<(), HetznerPreparationError> {
        buffer::write_json_string(self.output, &mut self.len, value, BODY_ERROR)
    }

    pub(crate) fn u64(&mut self, value: u64) -> Result<(), HetznerPreparationError> {
        buffer::write_u64(self.output, &mut self.len, value, BODY_ERROR)
    }

    pub(crate) fn bool(&mut self, value: bool) -> Result<(), HetznerPreparationError> {
        self.raw(if value { "true" } else { "false" })
    }

    pub(crate) fn null(&mut self) -> Result<(), HetznerPreparationError> {
        self.raw("null")
    }

    pub(crate) fn field_string(
        &mut self,
        first: &mut bool,
        name: &str,
        value: &str,
    ) -> Result<(), HetznerPreparationError> {
        self.field(first, name)?;
        self.string(value)
    }

    pub(crate) fn field_u64(
        &mut self,
        first: &mut bool,
        name: &str,
        value: u64,
    ) -> Result<(), HetznerPreparationError> {
        self.field(first, name)?;
        self.u64(value)
    }

    pub(crate) fn field_bool(
        &mut self,
        first: &mut bool,
        name: &str,
        value: bool,
    ) -> Result<(), HetznerPreparationError> {
        self.field(first, name)?;
        self.bool(value)
    }

    pub(crate) fn field_null(
        &mut self,
        first: &mut bool,
        name: &str,
    ) -> Result<(), HetznerPreparationError> {
        self.field(first, name)?;
        self.null()
    }

    pub(crate) fn field_sensitive<F, E>(
        &mut self,
        first: &mut bool,
        name: &str,
        write: F,
    ) -> Result<(), HetznerPreparationError>
    where
        F: FnOnce(&mut [u8]) -> Result<usize, E>,
    {
        self.field(first, name)?;
        let remaining = self
            .output
            .get_mut(self.len..)
            .ok_or(HetznerPreparationError::Body)?;
        let written = write(remaining).map_err(|_| HetznerPreparationError::Body)?;
        remaining
            .get(..written)
            .ok_or(HetznerPreparationError::Body)?;
        self.len = self
            .len
            .checked_add(written)
            .ok_or(HetznerPreparationError::Body)?;
        Ok(())
    }

    pub(crate) fn field_labels(
        &mut self,
        first: &mut bool,
        name: &str,
        labels: CloudLabels<'_>,
    ) -> Result<(), HetznerPreparationError> {
        self.field_label_entries(first, name, labels.entries())
    }

    pub(crate) fn field_label_entries(
        &mut self,
        first: &mut bool,
        name: &str,
        labels: &[(LabelKey<'_>, LabelValue<'_>)],
    ) -> Result<(), HetznerPreparationError> {
        self.field(first, name)?;
        self.begin_object()?;
        let mut first_label = true;
        for (key, value) in labels {
            self.field_string(&mut first_label, key.as_str(), value.as_str())?;
        }
        self.end_object()
    }

    fn separator(&mut self, first: &mut bool) -> Result<(), HetznerPreparationError> {
        if *first {
            *first = false;
            Ok(())
        } else {
            self.byte(b',')
        }
    }

    fn byte(&mut self, value: u8) -> Result<(), HetznerPreparationError> {
        buffer::write_byte(self.output, &mut self.len, value, BODY_ERROR)
    }

    fn raw(&mut self, value: &str) -> Result<(), HetznerPreparationError> {
        buffer::write_str(self.output, &mut self.len, value, BODY_ERROR)
    }
}
