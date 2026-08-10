//! Reviewed SHA-256 plan-fingerprint adapter.

use cloud_sdk::retry::{DigestAlgorithm, FingerprintHasher};

/// SHA-256 implementation for collision-resistant plan fingerprints.
///
/// This uses the allocation-free `sha2` implementation already admitted by
/// the `serde` feature for checked SSH-key response models.
#[derive(Clone, Copy, Debug, Default)]
pub struct Sha256PlanHasher;

impl FingerprintHasher for Sha256PlanHasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        use sha2::Digest;

        let digest = sha2::Sha256::digest(input);
        let Some(target) = output.get_mut(..digest.len()) else {
            return Ok(0);
        };
        target.copy_from_slice(&digest);
        Ok(digest.len())
    }
}
