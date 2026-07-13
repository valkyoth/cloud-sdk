use bytes::Bytes;
use cloud_sdk_sanitization::sanitize_bytes;
use std::vec::Vec;

pub(super) struct SanitizedBuffer {
    bytes: Vec<u8>,
}

impl SanitizedBuffer {
    pub(super) fn copy_from(source: &[u8]) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(source.len()).map_err(|_| ())?;
        bytes.extend_from_slice(source);
        Ok(Self { bytes })
    }

    pub(super) fn with_capacity(capacity: usize) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(capacity).map_err(|_| ())?;
        Ok(Self { bytes })
    }

    pub(super) fn extend_bounded(&mut self, source: &[u8], limit: usize) -> Result<(), ()> {
        let new_len = self.bytes.len().checked_add(source.len()).ok_or(())?;
        if new_len > limit || new_len > self.bytes.capacity() {
            return Err(());
        }
        self.bytes.extend_from_slice(source);
        Ok(())
    }

    pub(super) fn len(&self) -> usize {
        self.bytes.len()
    }

    pub(super) fn into_bytes(self) -> Bytes {
        Bytes::from_owner(self)
    }
}

impl AsRef<[u8]> for SanitizedBuffer {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}

impl Drop for SanitizedBuffer {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.bytes);
    }
}
