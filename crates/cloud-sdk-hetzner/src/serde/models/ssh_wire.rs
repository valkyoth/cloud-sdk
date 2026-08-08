use alloc::vec::Vec;
use core::str;

use base64_ng::STRICT_STANDARD_PADDED;
use cloud_sdk_sanitization::SecretBuffer;
use md5::Md5;
use sha2::Sha256;

use super::ResponseModelError;
use crate::security::shared::{SshAlgorithm, ssh_algorithm};
use crate::security::ssh_keys::{MAX_SSH_PUBLIC_KEY_BYTES, SshPublicKey};

pub(super) fn validate_key_identity(
    value: &str,
    supplied_fingerprint: [u8; 16],
) -> Result<[u8; 32], ResponseModelError> {
    SshPublicKey::new(value).map_err(|_| ResponseModelError::InvalidText)?;
    let mut fields = value.split(' ');
    let algorithm_text = fields.next().ok_or(ResponseModelError::InvalidText)?;
    let encoded = fields.next().ok_or(ResponseModelError::InvalidText)?;
    let algorithm = ssh_algorithm(algorithm_text).ok_or(ResponseModelError::InvalidText)?;
    let decoded_len = STRICT_STANDARD_PADDED
        .decoded_len(encoded.as_bytes())
        .map_err(|_| ResponseModelError::InvalidText)?;
    if decoded_len == 0 || decoded_len > MAX_SSH_PUBLIC_KEY_BYTES {
        return Err(ResponseModelError::InvalidText);
    }

    let mut storage = Vec::new();
    storage
        .try_reserve_exact(decoded_len)
        .map_err(|_| ResponseModelError::Allocation)?;
    storage.resize(decoded_len, 0);
    let mut wire = SecretBuffer::new(storage.as_mut_slice());
    let written = STRICT_STANDARD_PADDED
        .decode_into(encoded.as_bytes(), wire.as_mut_slice())
        .map_err(|_| ResponseModelError::InvalidText)?;
    if written != decoded_len {
        return Err(ResponseModelError::InvalidText);
    }
    validate_wire(algorithm, wire.as_slice())?;

    let computed_fingerprint: [u8; 16] = <Md5 as md5::Digest>::digest(wire.as_slice()).into();
    if computed_fingerprint != supplied_fingerprint {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(<Sha256 as sha2::Digest>::digest(wire.as_slice()).into())
}

fn validate_wire(algorithm: SshAlgorithm, wire: &[u8]) -> Result<(), ResponseModelError> {
    let mut reader = WireReader::new(wire);
    if reader.string()? != algorithm.as_str().as_bytes() {
        return Err(ResponseModelError::InvalidText);
    }
    match algorithm {
        SshAlgorithm::Ed25519 => validate_ed25519(&mut reader)?,
        SshAlgorithm::Rsa => validate_rsa(&mut reader)?,
        SshAlgorithm::EcdsaNistP256 => validate_ecdsa(&mut reader, b"nistp256", 33, 65)?,
        SshAlgorithm::EcdsaNistP384 => validate_ecdsa(&mut reader, b"nistp384", 49, 97)?,
        SshAlgorithm::EcdsaNistP521 => validate_ecdsa(&mut reader, b"nistp521", 67, 133)?,
        SshAlgorithm::SkEd25519 => {
            validate_ed25519(&mut reader)?;
            validate_application(&mut reader)?;
        }
        SshAlgorithm::SkEcdsaNistP256 => {
            validate_ecdsa(&mut reader, b"nistp256", 33, 65)?;
            validate_application(&mut reader)?;
        }
    }
    reader.finish()
}

fn validate_ed25519(reader: &mut WireReader<'_>) -> Result<(), ResponseModelError> {
    if reader.string()?.len() != 32 {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(())
}

fn validate_rsa(reader: &mut WireReader<'_>) -> Result<(), ResponseModelError> {
    if !is_positive_mpint(reader.string()?) || !is_positive_mpint(reader.string()?) {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(())
}

fn validate_ecdsa(
    reader: &mut WireReader<'_>,
    curve: &[u8],
    compressed_len: usize,
    uncompressed_len: usize,
) -> Result<(), ResponseModelError> {
    if reader.string()? != curve
        || !is_sec1_point(reader.string()?, compressed_len, uncompressed_len)
    {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(())
}

fn validate_application(reader: &mut WireReader<'_>) -> Result<(), ResponseModelError> {
    let value = str::from_utf8(reader.string()?).map_err(|_| ResponseModelError::InvalidText)?;
    if value.is_empty() || value.bytes().any(|byte| byte < 0x20 || byte == 0x7f) {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(())
}

fn is_positive_mpint(value: &[u8]) -> bool {
    let Some((first, rest)) = value.split_first() else {
        return false;
    };
    if *first == 0 {
        rest.first().is_some_and(|next| next & 0x80 != 0)
    } else {
        first & 0x80 == 0
    }
}

fn is_sec1_point(value: &[u8], compressed_len: usize, uncompressed_len: usize) -> bool {
    match value.first() {
        Some(2 | 3) => value.len() == compressed_len,
        Some(4) => value.len() == uncompressed_len,
        _ => false,
    }
}

struct WireReader<'a> {
    remaining: &'a [u8],
}

impl<'a> WireReader<'a> {
    const fn new(remaining: &'a [u8]) -> Self {
        Self { remaining }
    }

    fn string(&mut self) -> Result<&'a [u8], ResponseModelError> {
        let length_bytes = self
            .remaining
            .get(..4)
            .ok_or(ResponseModelError::InvalidText)?;
        let length_bytes: [u8; 4] = length_bytes
            .try_into()
            .map_err(|_| ResponseModelError::InvalidText)?;
        let length = usize::try_from(u32::from_be_bytes(length_bytes))
            .map_err(|_| ResponseModelError::InvalidText)?;
        let after_length = self
            .remaining
            .get(4..)
            .ok_or(ResponseModelError::InvalidText)?;
        let value = after_length
            .get(..length)
            .ok_or(ResponseModelError::InvalidText)?;
        self.remaining = after_length
            .get(length..)
            .ok_or(ResponseModelError::InvalidText)?;
        Ok(value)
    }

    fn finish(self) -> Result<(), ResponseModelError> {
        if self.remaining.is_empty() {
            Ok(())
        } else {
            Err(ResponseModelError::InvalidText)
        }
    }
}
