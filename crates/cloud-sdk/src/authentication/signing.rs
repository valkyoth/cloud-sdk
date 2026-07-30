//! Bounded canonical inputs for provider-owned request signing.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::buffer::{SnapshotEncoder, encode_snapshot_bounded, measure_snapshot_bounded};
use crate::transport::{RequestHeader, TransportRequest};

/// Maximum request-body digest bytes accepted by the canonical format.
pub const MAX_SIGNING_BODY_DIGEST_BYTES: usize = 128;
/// Maximum caller-provided nonce bytes accepted by the canonical format.
pub const MAX_SIGNING_NONCE_BYTES: usize = 256;
/// Maximum selected request headers.
pub const MAX_SIGNING_HEADERS: usize = 32;
/// Maximum complete canonical signing-input bytes.
pub const MAX_CANONICAL_SIGNING_INPUT_BYTES: usize = 12_288;

const SIGNING_DOMAIN: &[u8] = b"cloud-sdk-signing-v1\0";

/// Bounded signing value validation failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SigningValueError {
    /// The value must not be empty.
    Empty,
    /// The value exceeds its type-specific byte limit.
    TooLong,
}

impl_static_error!(SigningValueError,
    Self::Empty => "signing value is empty",
    Self::TooLong => "signing value exceeds the length limit",
);

/// Borrowed request-body digest produced by caller-selected hashing.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SigningBodyDigest<'a>(&'a [u8]);

impl<'a> SigningBodyDigest<'a> {
    /// Validates a caller-produced request-body digest.
    pub fn new(value: &'a [u8]) -> Result<Self, SigningValueError> {
        validate_bounded(value, MAX_SIGNING_BODY_DIGEST_BYTES)?;
        Ok(Self(value))
    }

    /// Returns the exact digest bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for SigningBodyDigest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SigningBodyDigest([redacted])")
    }
}

/// Borrowed caller-provided nonce.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SigningNonce<'a>(&'a [u8]);

impl<'a> SigningNonce<'a> {
    /// Validates a nonce without generating randomness.
    pub fn new(value: &'a [u8]) -> Result<Self, SigningValueError> {
        validate_bounded(value, MAX_SIGNING_NONCE_BYTES)?;
        Ok(Self(value))
    }

    /// Returns the exact nonce bytes.
    #[must_use]
    pub const fn as_bytes(self) -> &'a [u8] {
        self.0
    }
}

impl fmt::Debug for SigningNonce<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SigningNonce([redacted])")
    }
}

/// Caller-observed Unix time in whole seconds.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UnixTime(u64);

impl UnixTime {
    /// Wraps a caller-observed timestamp. Core acquires no clock.
    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Returns whole seconds since the Unix epoch.
    #[must_use]
    pub const fn as_seconds(self) -> u64 {
        self.0
    }
}

/// Canonically ordered request headers selected by provider signing policy.
#[derive(Clone, Copy)]
pub struct SigningHeaders<'a> {
    entries: &'a [RequestHeader<'a>],
}

impl<'a> SigningHeaders<'a> {
    /// Requires a strictly ascending, case-insensitive header-name order.
    pub fn new(entries: &'a [RequestHeader<'a>]) -> Result<Self, SigningInputError> {
        if entries.len() > MAX_SIGNING_HEADERS {
            return Err(SigningInputError::TooManyHeaders);
        }
        for pair in entries.windows(2) {
            let Some(left) = pair.first() else {
                return Err(SigningInputError::HeaderOrder);
            };
            let Some(right) = pair.get(1) else {
                return Err(SigningInputError::HeaderOrder);
            };
            if left.name() >= right.name() {
                return Err(SigningInputError::HeaderOrder);
            }
        }
        Ok(Self { entries })
    }

    /// Returns the selected canonical header sequence.
    #[must_use]
    pub const fn as_slice(self) -> &'a [RequestHeader<'a>] {
        self.entries
    }
}

impl fmt::Debug for SigningHeaders<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SigningHeaders")
            .field("count", &self.entries.len())
            .field("values", &"[redacted]")
            .finish()
    }
}

/// Canonical signing-input construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SigningInputError {
    /// More than [`MAX_SIGNING_HEADERS`] headers were selected.
    TooManyHeaders,
    /// Selected headers are duplicated or not in canonical order.
    HeaderOrder,
    /// A selected header is absent from or differs from the request.
    HeaderMismatch,
    /// A field length cannot be represented by the canonical format.
    LengthOverflow,
    /// The canonical input exceeds its aggregate bound.
    InputTooLarge,
    /// Caller output cannot hold the complete canonical input.
    OutputTooSmall,
    /// Transactional replay did not reproduce the measured bytes.
    SnapshotChanged,
}

impl_static_error!(SigningInputError,
    Self::TooManyHeaders => "too many signing headers were selected",
    Self::HeaderOrder => "signing headers are not in canonical order",
    Self::HeaderMismatch => "signing header does not match the request",
    Self::LengthOverflow => "signing field length cannot be represented",
    Self::InputTooLarge => "canonical signing input exceeds the length limit",
    Self::OutputTooSmall => "canonical signing output is too small",
    Self::SnapshotChanged => "canonical signing snapshot changed during encoding",
);

/// Caller-provided request-body hashing implementation.
pub trait RequestBodyHasher {
    /// Hashing failure.
    type Error;

    /// Hashes the exact request body into caller-owned output.
    fn hash_body(&self, body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Caller-provided request-signing implementation.
pub trait RequestSigner {
    /// Signing failure.
    type Error;

    /// Signs the exact versioned canonical input into caller-owned output.
    fn sign(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Cleanup-owning canonical request-signing input.
pub struct CanonicalSigningInput<'a> {
    storage: &'a mut [u8],
    len: usize,
}

impl<'a> CanonicalSigningInput<'a> {
    /// Builds a versioned canonical input without acquiring time or randomness.
    pub fn new(
        request: TransportRequest<'_>,
        selected_headers: SigningHeaders<'_>,
        body_digest: SigningBodyDigest<'_>,
        nonce: SigningNonce<'_>,
        time: UnixTime,
        output: &'a mut [u8],
    ) -> Result<Self, SigningInputError> {
        validate_selected_headers(request, selected_headers)?;
        let snapshot = SigningSnapshot {
            request,
            selected_headers,
            body_digest,
            nonce,
            time,
        };
        let required = measure_snapshot_bounded(
            snapshot,
            MAX_CANONICAL_SIGNING_INPUT_BYTES,
            SigningInputError::InputTooLarge,
            encode_signing_snapshot,
        )?;
        if output.len() < required {
            return Err(SigningInputError::OutputTooSmall);
        }
        let len = encode_snapshot_bounded(
            snapshot,
            output,
            MAX_CANONICAL_SIGNING_INPUT_BYTES,
            SigningInputError::SnapshotChanged,
            encode_signing_snapshot,
        )?;
        Ok(Self {
            storage: output,
            len,
        })
    }

    /// Returns the exact bytes to hash or sign.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.storage.get(..self.len).unwrap_or_default()
    }

    /// Invokes a caller-provided signer over the exact canonical bytes.
    pub fn sign_with<S: RequestSigner>(
        &self,
        signer: &S,
        output: &mut [u8],
    ) -> Result<usize, S::Error> {
        signer.sign(self.as_bytes(), output)
    }
}

impl fmt::Debug for CanonicalSigningInput<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalSigningInput")
            .field("len", &self.len)
            .field("input", &"[redacted]")
            .finish()
    }
}

impl Drop for CanonicalSigningInput<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
    }
}

#[derive(Clone, Copy)]
struct SigningSnapshot<'a> {
    request: TransportRequest<'a>,
    selected_headers: SigningHeaders<'a>,
    body_digest: SigningBodyDigest<'a>,
    nonce: SigningNonce<'a>,
    time: UnixTime,
}

fn validate_bounded(value: &[u8], maximum: usize) -> Result<(), SigningValueError> {
    if value.is_empty() {
        return Err(SigningValueError::Empty);
    }
    if value.len() > maximum {
        return Err(SigningValueError::TooLong);
    }
    Ok(())
}

fn validate_selected_headers(
    request: TransportRequest<'_>,
    selected: SigningHeaders<'_>,
) -> Result<(), SigningInputError> {
    for expected in selected.as_slice() {
        let Some(actual) = request.headers().get(expected.name().as_str()) else {
            return Err(SigningInputError::HeaderMismatch);
        };
        if actual.value().as_str().as_bytes() != expected.value().as_str().as_bytes()
            || actual.sensitivity() != expected.sensitivity()
        {
            return Err(SigningInputError::HeaderMismatch);
        }
    }
    Ok(())
}

fn encode_signing_snapshot(
    snapshot: SigningSnapshot<'_>,
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
) -> Result<(), SigningInputError> {
    encoder.bytes(SIGNING_DOMAIN)?;
    encode_u8_len(encoder, snapshot.request.method().as_str().as_bytes())?;
    encode_u16_len(encoder, snapshot.request.target().as_str().as_bytes())?;
    let count = u8::try_from(snapshot.selected_headers.as_slice().len())
        .map_err(|_| SigningInputError::LengthOverflow)?;
    encoder.byte(count)?;
    for header in snapshot.selected_headers.as_slice() {
        encode_lowercase_name(encoder, header.name().as_str())?;
        encode_u16_len(encoder, header.value().as_str().as_bytes())?;
    }
    encode_u8_len(encoder, snapshot.body_digest.as_bytes())?;
    encode_u16_len(encoder, snapshot.nonce.as_bytes())?;
    encoder.bytes(&snapshot.time.as_seconds().to_be_bytes())
}

fn encode_lowercase_name(
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
    name: &str,
) -> Result<(), SigningInputError> {
    let len = u8::try_from(name.len()).map_err(|_| SigningInputError::LengthOverflow)?;
    encoder.byte(len)?;
    for byte in name.bytes() {
        encoder.byte(byte.to_ascii_lowercase())?;
    }
    Ok(())
}

fn encode_u8_len(
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
    value: &[u8],
) -> Result<(), SigningInputError> {
    let len = u8::try_from(value.len()).map_err(|_| SigningInputError::LengthOverflow)?;
    encoder.byte(len)?;
    encoder.bytes(value)
}

fn encode_u16_len(
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
    value: &[u8],
) -> Result<(), SigningInputError> {
    let len = u16::try_from(value.len()).map_err(|_| SigningInputError::LengthOverflow)?;
    encoder.bytes(&len.to_be_bytes())?;
    encoder.bytes(value)
}

#[cfg(test)]
mod tests;
