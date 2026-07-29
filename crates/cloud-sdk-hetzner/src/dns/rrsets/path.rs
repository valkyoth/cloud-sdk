//! RRSet endpoint path construction.

use cloud_sdk::buffer;

use crate::cloud::shared::CloudRequestError;
use crate::dns::zones::ZoneReference;
use crate::request::{EndpointPath, MAX_ENDPOINT_PATH_BYTES};

use super::{RrsetReference, RrsetRequestError};

pub(crate) fn write_collection_path(
    output: &mut [u8],
    zone: ZoneReference<'_>,
) -> Result<usize, RrsetRequestError> {
    let len = buffer::encode_snapshot_bounded(
        zone,
        output,
        MAX_ENDPOINT_PATH_BYTES,
        CloudRequestError::PathBufferTooSmall,
        |zone, encoder| {
            encoder.string("/zones/")?;
            match zone {
                ZoneReference::Id(id) => encoder.u64(id.get())?,
                ZoneReference::Name(name) => encoder.string(name.as_str())?,
            }
            encoder.string("/rrsets")
        },
    )?;
    validate_or_clear_path(output, len)?;
    Ok(len)
}

pub(crate) fn write_rrset_path(
    output: &mut [u8],
    rrset: RrsetReference<'_>,
    suffix: &str,
) -> Result<usize, RrsetRequestError> {
    let len = buffer::encode_snapshot_bounded(
        (rrset, suffix),
        output,
        MAX_ENDPOINT_PATH_BYTES,
        CloudRequestError::PathBufferTooSmall,
        |(rrset, suffix), encoder| {
            let (zone, name, rr_type) = rrset.parts();
            encoder.string("/zones/")?;
            match zone {
                ZoneReference::Id(id) => encoder.u64(id.get())?,
                ZoneReference::Name(name) => encoder.string(name.as_str())?,
            }
            encoder.string("/rrsets/")?;
            encoder.percent_encoded(name.as_str())?;
            encoder.byte(b'/')?;
            encoder.string(rr_type.as_api_str())?;
            encoder.string(suffix)
        },
    )?;
    validate_or_clear_path(output, len)?;
    Ok(len)
}

fn validate_or_clear_path(output: &mut [u8], len: usize) -> Result<(), RrsetRequestError> {
    let value = core::str::from_utf8(
        output
            .get(..len)
            .ok_or(CloudRequestError::PathBufferTooSmall)?,
    )
    .map_err(|_| CloudRequestError::PathEncodingFailed)?;
    if let Err(error) = EndpointPath::new(value).map_err(CloudRequestError::InvalidPath) {
        if let Some(path) = output.get_mut(..len) {
            cloud_sdk_sanitization::sanitize_bytes(path);
        }
        return Err(error.into());
    }
    Ok(())
}
