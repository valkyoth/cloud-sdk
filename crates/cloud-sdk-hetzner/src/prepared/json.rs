//! Allocation-free JSON request-body writing.

use cloud_sdk::buffer::{self, SnapshotEncoder};

use crate::cloud::shared::CloudLabels;
use crate::labels::{LabelKey, LabelValue};

use super::HetznerPreparationError;

const BODY_ERROR: HetznerPreparationError = HetznerPreparationError::Body;

/// Maximum encoded JSON request body admitted by preparation.
pub(crate) const MAX_JSON_REQUEST_BYTES: usize = 8_388_608;

/// Secret text that can only emit one correctly escaped JSON string.
pub(crate) trait SensitiveJsonString: Copy {
    fn encode_json(
        self,
        encoder: &mut SnapshotEncoder<'_, HetznerPreparationError>,
    ) -> Result<(), HetznerPreparationError>;
}

pub(crate) fn encode_object<F>(
    output: &mut [u8],
    write: F,
) -> Result<usize, HetznerPreparationError>
where
    F: Copy
        + for<'encoder, 'output> Fn(
            &mut JsonWriter<'encoder, 'output>,
            &mut bool,
        ) -> Result<(), HetznerPreparationError>,
{
    buffer::encode_snapshot_bounded(
        write,
        output,
        MAX_JSON_REQUEST_BYTES,
        BODY_ERROR,
        |write, encoder| {
            let mut writer = JsonWriter::new(encoder);
            writer.begin_object()?;
            let mut first = true;
            write(&mut writer, &mut first)?;
            writer.end_object()
        },
    )
}

/// Small JSON token writer over one caller-owned output buffer.
pub(crate) struct JsonWriter<'encoder, 'output> {
    encoder: &'encoder mut SnapshotEncoder<'output, HetznerPreparationError>,
}

impl<'encoder, 'output> JsonWriter<'encoder, 'output> {
    pub(crate) fn new(
        encoder: &'encoder mut SnapshotEncoder<'output, HetznerPreparationError>,
    ) -> Self {
        Self { encoder }
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
        self.encoder.json_string(value)
    }

    pub(crate) fn u64(&mut self, value: u64) -> Result<(), HetznerPreparationError> {
        self.encoder.u64(value)
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

    pub(crate) fn field_sensitive<T: SensitiveJsonString>(
        &mut self,
        first: &mut bool,
        name: &str,
        value: T,
    ) -> Result<(), HetznerPreparationError> {
        self.field(first, name)?;
        value.encode_json(self.encoder)
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
        self.encoder.byte(value)
    }

    fn raw(&mut self, value: &str) -> Result<(), HetznerPreparationError> {
        self.encoder.string(value)
    }
}
