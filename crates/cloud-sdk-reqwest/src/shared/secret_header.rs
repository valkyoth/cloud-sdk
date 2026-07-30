use bytes::Bytes;
use cloud_sdk_sanitization::sanitize_bytes;
use reqwest::header::HeaderValue;
#[cfg(test)]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
use std::vec::Vec;

pub(crate) fn sensitive_header_value(source: &[u8]) -> Result<HeaderValue, ()> {
    let owner = SanitizedHeaderValue::copy_from(source)?;
    into_header_value(owner)
}

#[cfg(test)]
pub(crate) fn sensitive_header_value_with_probe(
    source: &[u8],
    probe: Option<Arc<AtomicUsize>>,
) -> Result<HeaderValue, ()> {
    let mut owner = SanitizedHeaderValue::copy_from(source)?;
    owner.drop_probe = probe;
    into_header_value(owner)
}

fn into_header_value(owner: SanitizedHeaderValue) -> Result<HeaderValue, ()> {
    let mut value = HeaderValue::from_maybe_shared(Bytes::from_owner(owner)).map_err(|_| ())?;
    value.set_sensitive(true);
    Ok(value)
}

struct SanitizedHeaderValue {
    bytes: Vec<u8>,
    #[cfg(test)]
    drop_probe: Option<Arc<AtomicUsize>>,
}

impl SanitizedHeaderValue {
    fn copy_from(source: &[u8]) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(source.len()).map_err(|_| ())?;
        bytes.extend_from_slice(source);
        Ok(Self {
            bytes,
            #[cfg(test)]
            drop_probe: None,
        })
    }
}

impl AsRef<[u8]> for SanitizedHeaderValue {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for SanitizedHeaderValue {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.bytes);
        #[cfg(test)]
        if let Some(probe) = &self.drop_probe {
            probe.fetch_add(1, Ordering::SeqCst);
        }
    }
}
