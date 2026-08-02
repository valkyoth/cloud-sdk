//! Transactional canonical fingerprint byte writer.

use super::FingerprintBuildError;
use crate::transport::CanonicalHost;

pub(super) const fn canonical_host_len(host: CanonicalHost<'_>) -> usize {
    match host {
        CanonicalHost::Dns(value) => value.len().saturating_add(1),
        CanonicalHost::Ipv4(_) => 5,
        CanonicalHost::Ipv6(_) => 17,
    }
}

pub(super) struct Writer<'a> {
    output: &'a mut [u8],
    len: usize,
}

impl<'a> Writer<'a> {
    pub(super) const fn new(output: &'a mut [u8]) -> Self {
        Self { output, len: 0 }
    }

    pub(super) fn raw<E>(&mut self, bytes: &[u8]) -> Result<(), FingerprintBuildError<E>> {
        let end = self
            .len
            .checked_add(bytes.len())
            .ok_or(FingerprintBuildError::LengthOverflow)?;
        self.output
            .get_mut(self.len..end)
            .ok_or(FingerprintBuildError::OutputTooSmall)?
            .copy_from_slice(bytes);
        self.len = end;
        Ok(())
    }

    pub(super) fn field<E>(
        &mut self,
        tag: u8,
        bytes: &[u8],
    ) -> Result<(), FingerprintBuildError<E>> {
        self.raw(&[tag])?;
        let len = u64::try_from(bytes.len()).map_err(|_| FingerprintBuildError::LengthOverflow)?;
        self.raw(&len.to_be_bytes())?;
        self.raw(bytes)
    }

    pub(super) fn lowercase_field<E>(
        &mut self,
        tag: u8,
        bytes: &[u8],
    ) -> Result<(), FingerprintBuildError<E>> {
        self.raw(&[tag])?;
        let len = u64::try_from(bytes.len()).map_err(|_| FingerprintBuildError::LengthOverflow)?;
        self.raw(&len.to_be_bytes())?;
        for byte in bytes {
            self.raw(&[byte.to_ascii_lowercase()])?;
        }
        Ok(())
    }

    pub(super) fn canonical_host_field<E>(
        &mut self,
        tag: u8,
        host: CanonicalHost<'_>,
    ) -> Result<(), FingerprintBuildError<E>> {
        self.raw(&[tag])?;
        let len = u64::try_from(canonical_host_len(host))
            .map_err(|_| FingerprintBuildError::LengthOverflow)?;
        self.raw(&len.to_be_bytes())?;
        match host {
            CanonicalHost::Dns(value) => {
                self.raw(&[0])?;
                self.raw(value.as_bytes())
            }
            CanonicalHost::Ipv4(octets) => {
                self.raw(&[1])?;
                self.raw(&octets)
            }
            CanonicalHost::Ipv6(octets) => {
                self.raw(&[2])?;
                self.raw(&octets)
            }
        }
    }
}
