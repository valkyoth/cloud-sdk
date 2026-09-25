use super::{MAX_PUBLISH_METADATA_BYTES, PublishError as Error, validation};
use crate::discovery::{DiscoveryValue, value::Builder};
use cloud_sdk::incremental_json::{
    IncrementalJsonDecoder, IncrementalJsonLimits, IncrementalJsonProgress,
};

/// Immutable Cargo publish JSON and its checked protected field model.
/// The original bytes are borrowed, not copied or cleared. Callers own their
/// erasure. Archive construction and manifest/metadata equivalence remain caller
/// responsibilities; this does not inspect or unpack an archive.
pub struct PublishMetadata<'a> {
    pub(super) bytes: &'a [u8],
    pub(super) fields: DiscoveryValue,
}
impl<'a> PublishMetadata<'a> {
    /// Validate the complete Cargo JSON shape, bounded values, dependency and
    /// feature grammar, SPDX expression, URLs, paths and minimum Rust version.
    /// Unknown top-level fields require a source-contract update.
    pub fn from_json(bytes: &'a [u8]) -> Result<Self, Error> {
        if bytes.is_empty() || bytes.len() > MAX_PUBLISH_METADATA_BYTES {
            return Err(Error::Limit);
        }
        let mut builder = Builder::default();
        let mut decoder = IncrementalJsonDecoder::with_limits(
            IncrementalJsonLimits::DEFAULT
                .with_input_bytes(MAX_PUBLISH_METADATA_BYTES)
                .map_err(|_| Error::Limit)?,
        );
        decoder
            .push(bytes, &mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?;
        if decoder
            .finish(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let fields = builder.finish()?;
        validation::validate(&fields)?;
        Ok(Self { bytes, fields })
    }
    /// Complete checked Cargo fields, with closure-scoped text and redacted Debug.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.fields
    }
}
impl core::fmt::Debug for PublishMetadata<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PublishMetadata([redacted])")
    }
}
