//! Retry fingerprint test helpers.

use super::super::{DigestAlgorithm, FingerprintHasher};

pub(super) fn contains_field(mut input: &[u8], tag: u8, expected: &[u8]) -> bool {
    input = input
        .strip_prefix(b"cloud-sdk/retry-fingerprint/v2\0")
        .unwrap_or_default();
    while input.len() >= 9 {
        let Some(current_tag) = input.first().copied() else {
            return false;
        };
        let mut length = [0_u8; 8];
        let Some(encoded_length) = input.get(1..9) else {
            return false;
        };
        length.copy_from_slice(encoded_length);
        let Ok(length) = usize::try_from(u64::from_be_bytes(length)) else {
            return false;
        };
        let Some(end) = 9_usize.checked_add(length) else {
            return false;
        };
        let Some(value) = input.get(9..end) else {
            return false;
        };
        if current_tag == tag && value == expected {
            return true;
        }
        input = input.get(end..).unwrap_or_default();
    }
    false
}

pub(super) struct WrongLength;

impl FingerprintHasher for WrongLength {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xA5);
        Ok(31)
    }
}
