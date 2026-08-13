use alloc::vec::Vec;
use core::str;

use base64_ng::STRICT_STANDARD_PADDED;
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use md5::Md5;
use sha2::Sha256;

use super::ResponseModelError;
use crate::security::shared::{SshAlgorithm, ssh_algorithm};
use crate::security::ssh_keys::{MAX_SSH_PUBLIC_KEY_BYTES, SshPublicKey};

pub(crate) struct SshKeyIdentity {
    algorithm: SshAlgorithm,
    bits: u32,
    md5: [u8; 16],
    sha256: [u8; 32],
}

impl SshKeyIdentity {
    pub(crate) const fn algorithm(&self) -> SshAlgorithm {
        self.algorithm
    }

    pub(crate) const fn bits(&self) -> u32 {
        self.bits
    }

    pub(crate) const fn md5(&self) -> &[u8; 16] {
        &self.md5
    }

    pub(crate) const fn sha256(&self) -> &[u8; 32] {
        &self.sha256
    }

    pub(crate) fn take_sha256(&mut self) -> [u8; 32] {
        core::mem::take(&mut self.sha256)
    }
}

impl Drop for SshKeyIdentity {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.md5);
        sanitize_bytes(&mut self.sha256);
    }
}

pub(super) fn validate_key_identity(
    value: &str,
    supplied_fingerprint: &[u8],
) -> Result<[u8; 32], ResponseModelError> {
    let mut identity = parse_openssh_key_identity(value)?;
    if identity.md5().as_slice() != supplied_fingerprint {
        return Err(ResponseModelError::EnvelopeMismatch);
    }
    Ok(identity.take_sha256())
}

pub(crate) fn parse_openssh_key_identity(
    value: &str,
) -> Result<SshKeyIdentity, ResponseModelError> {
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
    parse_wire_identity(algorithm, wire.as_slice())
}

pub(crate) fn parse_ssh2_wire_identity(wire: &[u8]) -> Result<SshKeyIdentity, ResponseModelError> {
    let mut reader = WireReader::new(wire);
    let algorithm = str::from_utf8(reader.string()?)
        .ok()
        .and_then(ssh_algorithm)
        .ok_or(ResponseModelError::InvalidText)?;
    parse_wire_identity(algorithm, wire)
}

fn parse_wire_identity(
    algorithm: SshAlgorithm,
    wire: &[u8],
) -> Result<SshKeyIdentity, ResponseModelError> {
    let mut reader = WireReader::new(wire);
    if reader.string()? != algorithm.as_str().as_bytes() {
        return Err(ResponseModelError::InvalidText);
    }
    let bits = match algorithm {
        SshAlgorithm::Ed25519 => {
            validate_ed25519(&mut reader)?;
            256
        }
        SshAlgorithm::Rsa => validate_rsa(&mut reader)?,
        SshAlgorithm::EcdsaNistP256 => {
            validate_ecdsa(&mut reader, b"nistp256", 33, 65)?;
            256
        }
        SshAlgorithm::EcdsaNistP384 => {
            validate_ecdsa(&mut reader, b"nistp384", 49, 97)?;
            384
        }
        SshAlgorithm::EcdsaNistP521 => {
            validate_ecdsa(&mut reader, b"nistp521", 67, 133)?;
            521
        }
        SshAlgorithm::SkEd25519 => {
            validate_ed25519(&mut reader)?;
            validate_application(&mut reader)?;
            256
        }
        SshAlgorithm::SkEcdsaNistP256 => {
            validate_ecdsa(&mut reader, b"nistp256", 33, 65)?;
            validate_application(&mut reader)?;
            256
        }
    };
    reader.finish()?;
    Ok(SshKeyIdentity {
        algorithm,
        bits,
        md5: <Md5 as md5::Digest>::digest(wire).into(),
        sha256: <Sha256 as sha2::Digest>::digest(wire).into(),
    })
}

fn validate_ed25519(reader: &mut WireReader<'_>) -> Result<(), ResponseModelError> {
    if reader.string()?.len() != 32 {
        return Err(ResponseModelError::InvalidText);
    }
    Ok(())
}

fn validate_rsa(reader: &mut WireReader<'_>) -> Result<u32, ResponseModelError> {
    let exponent = reader.string()?;
    let modulus = reader.string()?;
    if !is_positive_mpint(exponent) || !is_positive_mpint(modulus) {
        return Err(ResponseModelError::InvalidText);
    }
    mpint_bits(modulus).ok_or(ResponseModelError::InvalidText)
}

fn mpint_bits(value: &[u8]) -> Option<u32> {
    let significant = if value.first() == Some(&0) {
        value.get(1..)?
    } else {
        value
    };
    let first = *significant.first()?;
    let tail_bits = u32::try_from(significant.len().checked_sub(1)?.checked_mul(8)?).ok()?;
    tail_bits.checked_add(8_u32.checked_sub(first.leading_zeros())?)
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
