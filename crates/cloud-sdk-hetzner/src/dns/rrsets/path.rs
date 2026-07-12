//! RRSet endpoint path construction.

use cloud_sdk::buffer;

use crate::cloud::shared::CloudRequestError;
use crate::dns::zones::ZoneReference;
use crate::request::EndpointPath;

use super::{RrsetReference, RrsetRequestError};

pub(crate) fn write_collection_path(
    output: &mut [u8],
    zone: ZoneReference<'_>,
) -> Result<usize, RrsetRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        "/zones/",
        CloudRequestError::PathBufferTooSmall,
    )?;
    match zone {
        ZoneReference::Id(id) => buffer::write_u64(
            output,
            &mut len,
            id.get(),
            CloudRequestError::PathBufferTooSmall,
        )?,
        ZoneReference::Name(name) => buffer::write_str(
            output,
            &mut len,
            name.as_str(),
            CloudRequestError::PathBufferTooSmall,
        )?,
    }
    buffer::write_str(
        output,
        &mut len,
        "/rrsets",
        CloudRequestError::PathBufferTooSmall,
    )?;
    validate_path(output, len)?;
    Ok(len)
}

pub(crate) fn write_rrset_path(
    output: &mut [u8],
    rrset: RrsetReference<'_>,
    suffix: &str,
) -> Result<usize, RrsetRequestError> {
    let (zone, name, rr_type) = rrset.parts();
    let mut len = write_collection_path(output, zone)?;
    buffer::write_byte(
        output,
        &mut len,
        b'/',
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_percent_encoded(
        output,
        &mut len,
        name.as_str(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_byte(
        output,
        &mut len,
        b'/',
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        rr_type.as_api_str(),
        CloudRequestError::PathBufferTooSmall,
    )?;
    buffer::write_str(
        output,
        &mut len,
        suffix,
        CloudRequestError::PathBufferTooSmall,
    )?;
    validate_path(output, len)?;
    Ok(len)
}

fn validate_path(output: &[u8], len: usize) -> Result<(), RrsetRequestError> {
    let value = core::str::from_utf8(
        output
            .get(..len)
            .ok_or(CloudRequestError::PathBufferTooSmall)?,
    )
    .map_err(|_| CloudRequestError::PathEncodingFailed)?;
    EndpointPath::new(value).map_err(CloudRequestError::InvalidPath)?;
    Ok(())
}
