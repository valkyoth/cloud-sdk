//! Bounded canonical inputs for provider-owned request signing.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::buffer::{SnapshotEncoder, encode_snapshot_bounded, measure_snapshot_bounded};
use crate::transport::{CanonicalHost, EndpointScheme, RequestHeader, TransportRequest};

use super::ScopeValue;

mod build;
mod context;
mod output;

pub use build::SigningBuildError;
use build::{DigestScratch, SigningBodyDigest};
pub use context::{
    MAX_SIGNING_ALGORITHM_BYTES, MAX_SIGNING_DIGEST_ALGORITHM_BYTES, MAX_SIGNING_KEY_ID_BYTES,
    SigningAlgorithm, SigningContext, SigningContextValueError, SigningDigestAlgorithm,
    SigningKeyId,
};
pub use output::{RequestSigner, SignedRequest, SigningOutputError};

/// Maximum request-body digest bytes accepted by the canonical format.
pub const MAX_SIGNING_BODY_DIGEST_BYTES: usize = 128;
/// Maximum caller-provided nonce bytes accepted by the canonical format.
pub const MAX_SIGNING_NONCE_BYTES: usize = 256;
/// Maximum selected request headers.
pub const MAX_SIGNING_HEADERS: usize = 32;
/// Maximum complete canonical signing-input bytes.
pub const MAX_CANONICAL_SIGNING_INPUT_BYTES: usize = 12_288;

const SIGNING_DOMAIN: &[u8] = b"cloud-sdk-signing-v2\0";

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

/// Caller-owned nonce and observed time bound into one anti-replay context.
#[derive(Clone, Copy)]
pub struct SigningFreshness<'a> {
    nonce: SigningNonce<'a>,
    time: UnixTime,
}

impl<'a> SigningFreshness<'a> {
    /// Combines caller-provided freshness values without acquiring either.
    #[must_use]
    pub const fn new(nonce: SigningNonce<'a>, time: UnixTime) -> Self {
        Self { nonce, time }
    }

    const fn nonce(self) -> SigningNonce<'a> {
        self.nonce
    }

    const fn time(self) -> UnixTime {
        self.time
    }
}

impl fmt::Debug for SigningFreshness<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SigningFreshness")
            .field("nonce", &"[redacted]")
            .field("time", &self.time)
            .finish()
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

    /// Returns the exact digest algorithm implemented by this hasher.
    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_>;

    /// Hashes the exact request body into caller-owned output.
    fn hash_body(&self, body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Cleanup-owning canonical input that retains the exact hashed request.
pub struct CanonicalSigningInput<'storage, 'request> {
    request: TransportRequest<'request>,
    context: SigningContext<'request>,
    storage: &'storage mut [u8],
    len: usize,
}

impl<'storage, 'request> CanonicalSigningInput<'storage, 'request> {
    /// Hashes the exact request body and builds a security-domain-bound input.
    pub fn new_hashed<H: RequestBodyHasher + ?Sized>(
        request: TransportRequest<'request>,
        context: SigningContext<'request>,
        selected_headers: SigningHeaders<'_>,
        freshness: SigningFreshness<'_>,
        hasher: &H,
        digest_storage: &mut [u8],
        output: &'storage mut [u8],
    ) -> Result<Self, SigningBuildError<H::Error>> {
        let mut digest_storage = DigestScratch::new(digest_storage);
        validate_selected_headers(request, selected_headers).map_err(SigningBuildError::Input)?;
        if hasher.digest_algorithm() != context.digest_algorithm() {
            return Err(SigningBuildError::DigestAlgorithmMismatch);
        }
        let digest_len = hasher
            .hash_body(request.body(), digest_storage.as_mut())
            .map_err(SigningBuildError::Hasher)?;
        let digest_bytes = digest_storage
            .as_slice()
            .get(..digest_len)
            .ok_or(SigningBuildError::Digest(SigningValueError::TooLong))?;
        let body_digest =
            SigningBodyDigest::new(digest_bytes).map_err(SigningBuildError::Digest)?;
        let snapshot = SigningSnapshot {
            request,
            context,
            selected_headers,
            body_digest,
            nonce: freshness.nonce(),
            time: freshness.time(),
        };
        let required = measure_snapshot_bounded(
            snapshot,
            MAX_CANONICAL_SIGNING_INPUT_BYTES,
            SigningInputError::InputTooLarge,
            encode_signing_snapshot,
        )
        .map_err(SigningBuildError::Input)?;
        if output.len() < required {
            return Err(SigningBuildError::Input(SigningInputError::OutputTooSmall));
        }
        let len = encode_snapshot_bounded(
            snapshot,
            output,
            MAX_CANONICAL_SIGNING_INPUT_BYTES,
            SigningInputError::SnapshotChanged,
            encode_signing_snapshot,
        )
        .map_err(SigningBuildError::Input)?;
        Ok(Self {
            request,
            context,
            storage: output,
            len,
        })
    }

    /// Returns the exact bytes to hash or sign.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.storage.get(..self.len).unwrap_or_default()
    }

    /// Returns the exact request retained by the canonical snapshot.
    #[must_use]
    pub const fn request(&self) -> TransportRequest<'request> {
        self.request
    }

    /// Returns the exact security domain retained by the canonical snapshot.
    #[must_use]
    pub const fn context(&self) -> SigningContext<'request> {
        self.context
    }

    /// Signs and returns the exact request with cleanup-owning bounded output.
    pub fn sign_into<'signature, S: RequestSigner>(
        self,
        signer: &S,
        output: &'signature mut [u8],
    ) -> Result<SignedRequest<'signature, 'request>, SigningOutputError<S::Error>> {
        let result =
            SignedRequest::sign(self.request, self.context, self.as_bytes(), signer, output);
        drop(self);
        result
    }
}

impl fmt::Debug for CanonicalSigningInput<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalSigningInput")
            .field("len", &self.len)
            .field("input", &"[redacted]")
            .finish()
    }
}

impl Drop for CanonicalSigningInput<'_, '_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
    }
}

#[derive(Clone, Copy)]
struct SigningSnapshot<'request, 'context, 'headers, 'digest, 'nonce> {
    request: TransportRequest<'request>,
    context: SigningContext<'context>,
    selected_headers: SigningHeaders<'headers>,
    body_digest: SigningBodyDigest<'digest>,
    nonce: SigningNonce<'nonce>,
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
    snapshot: SigningSnapshot<'_, '_, '_, '_, '_>,
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
) -> Result<(), SigningInputError> {
    encoder.bytes(SIGNING_DOMAIN)?;
    encode_u8_len(encoder, snapshot.context.provider().as_str().as_bytes())?;
    encode_u8_len(encoder, snapshot.context.service().as_str().as_bytes())?;
    let endpoint = snapshot.context.endpoint();
    let scheme = match endpoint.scheme() {
        EndpointScheme::Http => b"http".as_slice(),
        EndpointScheme::Https => b"https".as_slice(),
    };
    encode_u8_len(encoder, scheme)?;
    encode_canonical_host(encoder, endpoint.canonical_host())?;
    encoder.bytes(&endpoint.effective_port().to_be_bytes())?;
    encode_u16_len(encoder, endpoint.base_path().as_bytes())?;
    encode_optional_scope(encoder, snapshot.context.audience())?;
    encode_optional_scope(encoder, snapshot.context.account())?;
    encode_optional_scope(encoder, snapshot.context.tenant())?;
    encode_u16_len(encoder, snapshot.context.key_id().as_str().as_bytes())?;
    encode_u16_len(
        encoder,
        snapshot.context.digest_algorithm().as_str().as_bytes(),
    )?;
    encode_u16_len(
        encoder,
        snapshot.context.signature_algorithm().as_str().as_bytes(),
    )?;
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

fn encode_canonical_host(
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
    host: CanonicalHost<'_>,
) -> Result<(), SigningInputError> {
    match host {
        CanonicalHost::Dns(value) => {
            encoder.byte(0)?;
            encode_u16_len(encoder, value.as_bytes())
        }
        CanonicalHost::Ipv4(octets) => {
            encoder.byte(1)?;
            encoder.bytes(&octets)
        }
        CanonicalHost::Ipv6(octets) => {
            encoder.byte(2)?;
            encoder.bytes(&octets)
        }
    }
}

fn encode_optional_scope(
    encoder: &mut SnapshotEncoder<'_, SigningInputError>,
    value: Option<ScopeValue<'_>>,
) -> Result<(), SigningInputError> {
    match value {
        Some(value) => {
            encoder.byte(1)?;
            encode_u16_len(encoder, value.as_str().as_bytes())
        }
        None => encoder.byte(0),
    }
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
