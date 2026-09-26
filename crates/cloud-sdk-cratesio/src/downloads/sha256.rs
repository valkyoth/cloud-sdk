use super::{ArtifactChecksum, ArtifactError};
use sha2::Digest;

/// Incremental RustCrypto SHA-256 for public archive integrity. Available with
/// `artifact-sha256`, including both bundled transport features. This is not
/// a signature verifier or a FIPS claim; expected hashes need trusted provenance.
pub struct Sha256Checksum(Option<sha2::Sha256>);
impl Sha256Checksum {
    /// Creates a fresh single-use checksum state.
    #[must_use]
    pub fn new() -> Self {
        Self(Some(sha2::Sha256::new()))
    }
}
impl Default for Sha256Checksum {
    fn default() -> Self {
        Self::new()
    }
}
impl core::fmt::Debug for Sha256Checksum {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("Sha256Checksum([redacted])")
    }
}
impl ArtifactChecksum for Sha256Checksum {
    fn update(&mut self, bytes: &[u8]) -> Result<(), ArtifactError> {
        self.0
            .as_mut()
            .ok_or(ArtifactError::Checksum)?
            .update(bytes);
        Ok(())
    }
    fn finish(&mut self) -> Result<[u8; 32], ArtifactError> {
        Ok(self
            .0
            .take()
            .ok_or(ArtifactError::Checksum)?
            .finalize()
            .into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sha256_known_answer_and_single_use() {
        // Public SHA-256 known-answer vector; no credential or encryption key.
        let expected = [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ];
        let mut hash = Sha256Checksum::new();
        assert_eq!(hash.update(b"a"), Ok(()));
        assert_eq!(hash.update(b"bc"), Ok(()));
        assert_eq!(hash.finish(), Ok(expected));
        assert_eq!(hash.finish(), Err(ArtifactError::Checksum));
        assert_eq!(hash.update(b"d"), Err(ArtifactError::Checksum));
    }
}
