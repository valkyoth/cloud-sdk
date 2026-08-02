//! Versioned canonical request identity and caller-supplied strong digests.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::operation::PreparedRequest;
use crate::transport::{EndpointIdentity, EndpointScheme};

mod writer;
use writer::{Writer, canonical_host_len};

const DOMAIN: &[u8] = b"cloud-sdk/retry-fingerprint/v1\0";
/// Maximum account or tenant scope bytes admitted to a fingerprint.
pub const MAX_FINGERPRINT_SCOPE_BYTES: usize = 1024;
/// Maximum supported collision-resistant digest output.
pub const MAX_FINGERPRINT_DIGEST_BYTES: usize = 64;

/// Explicit account or tenant scope bound into a request fingerprint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FingerprintScope<'a> {
    /// The provider operation has no additional account or tenant scope.
    Absent,
    /// Exact caller-provided account or tenant scope bytes.
    Value(&'a [u8]),
}

/// Admitted collision-resistant digest algorithms.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DigestAlgorithm {
    /// SHA-256 with a 32-byte output.
    Sha256,
    /// SHA-384 with a 48-byte output.
    Sha384,
    /// SHA-512 with a 64-byte output.
    Sha512,
    /// BLAKE3 with its standard 32-byte output.
    Blake3,
}

impl DigestAlgorithm {
    const fn output_len(self) -> usize {
        match self {
            Self::Sha256 | Self::Blake3 => 32,
            Self::Sha384 => 48,
            Self::Sha512 => 64,
        }
    }
}

/// Caller-provided collision-resistant fingerprint digest implementation.
///
/// Implementations must compute the declared algorithm over exactly `input`.
/// Ordinary `Hash`, CRC, and other non-cryptographic digests do not satisfy
/// this security contract.
pub trait FingerprintHasher {
    /// Hashing failure.
    type Error;

    /// Returns the collision-resistant algorithm implemented by this value.
    fn algorithm(&self) -> DigestAlgorithm;

    /// Writes the digest and returns its initialized length.
    fn digest(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Canonical fingerprint or digest construction failure.
pub enum FingerprintBuildError<E> {
    /// Prepared requests used for retry must have an operation identifier.
    MissingOperationId,
    /// Account or tenant scope exceeds the bounded policy.
    ScopeTooLong,
    /// Canonical length arithmetic overflowed.
    LengthOverflow,
    /// Caller storage cannot hold the complete canonical fingerprint.
    OutputTooSmall,
    /// The caller-provided digest implementation failed.
    Hasher(E),
    /// The digest implementation returned the wrong initialized length.
    InvalidDigestLength,
}

impl<E> fmt::Debug for FingerprintBuildError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingOperationId => "FingerprintBuildError::MissingOperationId",
            Self::ScopeTooLong => "FingerprintBuildError::ScopeTooLong",
            Self::LengthOverflow => "FingerprintBuildError::LengthOverflow",
            Self::OutputTooSmall => "FingerprintBuildError::OutputTooSmall",
            Self::Hasher(_) => "FingerprintBuildError::Hasher([redacted])",
            Self::InvalidDigestLength => "FingerprintBuildError::InvalidDigestLength",
        })
    }
}

impl<E> fmt::Display for FingerprintBuildError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::MissingOperationId => "retry fingerprint requires an operation identifier",
            Self::ScopeTooLong => "retry fingerprint scope exceeds the length limit",
            Self::LengthOverflow => "retry fingerprint length overflowed",
            Self::OutputTooSmall => "retry fingerprint output is too small",
            Self::Hasher(_) => "retry fingerprint hashing failed",
            Self::InvalidDigestLength => "retry fingerprint digest length is invalid",
        })
    }
}

impl<E> core::error::Error for FingerprintBuildError<E>
where
    E: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Hasher(error) => Some(error),
            _ => None,
        }
    }
}

/// Caller-buffer canonical request fingerprint cleared on drop.
pub struct CanonicalFingerprint<'a> {
    storage: &'a mut [u8],
    len: usize,
}

impl CanonicalFingerprint<'_> {
    /// Returns a redacted comparison reference without exposing diagnostics.
    #[must_use]
    pub fn as_ref(&self) -> FingerprintRef<'_> {
        FingerprintRef(FingerprintKind::Exact(self.as_bytes()))
    }

    /// Returns the initialized canonical length.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Reports whether the canonical input is empty. It is never empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        false
    }

    fn as_bytes(&self) -> &[u8] {
        self.storage.get(..self.len).unwrap_or_default()
    }
}

impl fmt::Debug for CanonicalFingerprint<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalFingerprint")
            .field("len", &self.len)
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Drop for CanonicalFingerprint<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
    }
}

/// Owned collision-resistant digest cleared on drop.
pub struct FingerprintDigest {
    algorithm: DigestAlgorithm,
    bytes: [u8; MAX_FINGERPRINT_DIGEST_BYTES],
    len: usize,
}

impl FingerprintDigest {
    /// Returns the admitted digest algorithm.
    #[must_use]
    pub const fn algorithm(&self) -> DigestAlgorithm {
        self.algorithm
    }

    /// Returns a redacted comparison reference.
    #[must_use]
    pub fn as_ref(&self) -> FingerprintRef<'_> {
        FingerprintRef(FingerprintKind::Digest {
            algorithm: self.algorithm,
            bytes: self.bytes.get(..self.len).unwrap_or_default(),
        })
    }
}

impl fmt::Debug for FingerprintDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FingerprintDigest")
            .field("algorithm", &self.algorithm)
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Drop for FingerprintDigest {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.bytes);
        self.len = 0;
    }
}

/// Borrowed exact or collision-resistant request fingerprint.
#[derive(Clone, Copy)]
pub struct FingerprintRef<'a>(FingerprintKind<'a>);

#[derive(Clone, Copy)]
enum FingerprintKind<'a> {
    Exact(&'a [u8]),
    Digest {
        algorithm: DigestAlgorithm,
        bytes: &'a [u8],
    },
}

impl<'a> FingerprintRef<'a> {
    pub(crate) fn matches(self, other: Self) -> bool {
        match (self.0, other.0) {
            (FingerprintKind::Exact(left), FingerprintKind::Exact(right)) => {
                constant_time_eq(left, right)
            }
            (
                FingerprintKind::Digest {
                    algorithm: left_algorithm,
                    bytes: left,
                },
                FingerprintKind::Digest {
                    algorithm: right_algorithm,
                    bytes: right,
                },
            ) => left_algorithm == right_algorithm && constant_time_eq(left, right),
            _ => false,
        }
    }

    #[cfg(test)]
    pub(crate) const fn test_exact(bytes: &'a [u8]) -> Self {
        Self(FingerprintKind::Exact(bytes))
    }
}

impl fmt::Debug for FingerprintRef<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FingerprintRef([redacted])")
    }
}

/// Builds exact versioned canonical bytes into caller-owned storage.
pub fn build_canonical_fingerprint<'output>(
    request: PreparedRequest<'_>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    output: &'output mut [u8],
) -> Result<CanonicalFingerprint<'output>, FingerprintBuildError<core::convert::Infallible>> {
    sanitize_bytes(output);
    let required = encoded_len(&request, endpoint, scope)?;
    if output.len() < required {
        return Err(FingerprintBuildError::OutputTooSmall);
    }
    let encoded = {
        let mut writer = Writer::new(output);
        encode(&request, endpoint, scope, &mut writer)
    };
    if let Err(error) = encoded {
        sanitize_bytes(output);
        return Err(error);
    }
    Ok(CanonicalFingerprint {
        storage: output,
        len: required,
    })
}

/// Builds and hashes canonical bytes, clearing caller scratch on every path.
pub fn build_fingerprint_digest<H: FingerprintHasher>(
    request: PreparedRequest<'_>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    scratch: &mut [u8],
    hasher: &H,
) -> Result<FingerprintDigest, FingerprintBuildError<H::Error>> {
    let canonical = build_canonical_fingerprint(request, endpoint, scope, scratch)
        .map_err(map_infallible_error)?;
    let algorithm = hasher.algorithm();
    let expected = algorithm.output_len();
    let mut digest = FingerprintDigest {
        algorithm,
        bytes: [0_u8; MAX_FINGERPRINT_DIGEST_BYTES],
        len: 0,
    };
    let output = digest
        .bytes
        .get_mut(..expected)
        .ok_or(FingerprintBuildError::InvalidDigestLength)?;
    let len = hasher
        .digest(canonical.as_bytes(), output)
        .map_err(FingerprintBuildError::Hasher)?;
    if len != expected {
        return Err(FingerprintBuildError::InvalidDigestLength);
    }
    digest.len = len;
    Ok(digest)
}

fn map_infallible_error<E>(
    error: FingerprintBuildError<core::convert::Infallible>,
) -> FingerprintBuildError<E> {
    match error {
        FingerprintBuildError::MissingOperationId => FingerprintBuildError::MissingOperationId,
        FingerprintBuildError::ScopeTooLong => FingerprintBuildError::ScopeTooLong,
        FingerprintBuildError::LengthOverflow => FingerprintBuildError::LengthOverflow,
        FingerprintBuildError::OutputTooSmall => FingerprintBuildError::OutputTooSmall,
        FingerprintBuildError::InvalidDigestLength => FingerprintBuildError::InvalidDigestLength,
        FingerprintBuildError::Hasher(never) => match never {},
    }
}

fn encoded_len(
    prepared: &PreparedRequest<'_>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
) -> Result<usize, FingerprintBuildError<core::convert::Infallible>> {
    let operation = prepared
        .operation_id()
        .ok_or(FingerprintBuildError::MissingOperationId)?;
    let scope = scope_bytes(scope)?;
    let request = prepared.transport_request();
    let query = request.target().query_bytes().unwrap_or_default();
    let mut len = DOMAIN.len();
    for value in [
        prepared.service().provider_id().as_str().as_bytes(),
        prepared.service().service_id().as_str().as_bytes(),
        operation.as_str().as_bytes(),
        request.method().as_str().as_bytes(),
        endpoint.base_path().as_bytes(),
        request.target().path().as_str().as_bytes(),
        query,
        request.body(),
    ] {
        len = field_len(len, value.len())?;
    }
    len = field_len(len, 1)?;
    len = field_len(len, canonical_host_len(endpoint.canonical_host()))?;
    len = field_len(len, 2)?;
    len = field_len(len, 1)?;
    len = field_len(len, 2)?;
    len = field_len(len, 1)?;
    len = field_len(len, scope.len())?;
    for header in request.headers().as_slice() {
        len = field_len(len, header.name().as_str().len())?;
        len = field_len(len, header.value().as_str().len())?;
    }
    Ok(len)
}

fn field_len<E>(current: usize, value_len: usize) -> Result<usize, FingerprintBuildError<E>> {
    current
        .checked_add(9)
        .and_then(|value| value.checked_add(value_len))
        .ok_or(FingerprintBuildError::LengthOverflow)
}

fn scope_bytes(
    scope: FingerprintScope<'_>,
) -> Result<&[u8], FingerprintBuildError<core::convert::Infallible>> {
    let bytes = match scope {
        FingerprintScope::Absent => &[][..],
        FingerprintScope::Value(bytes) => bytes,
    };
    if bytes.len() > MAX_FINGERPRINT_SCOPE_BYTES {
        return Err(FingerprintBuildError::ScopeTooLong);
    }
    Ok(bytes)
}

fn encode<E>(
    prepared: &PreparedRequest<'_>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    writer: &mut Writer<'_>,
) -> Result<(), FingerprintBuildError<E>> {
    let operation = prepared
        .operation_id()
        .ok_or(FingerprintBuildError::MissingOperationId)?;
    let scope_present = matches!(scope, FingerprintScope::Value(_));
    let scope = scope_bytes(scope).map_err(map_infallible_error)?;
    let request = prepared.transport_request();
    writer.raw(DOMAIN)?;
    writer.field(1, prepared.service().provider_id().as_str().as_bytes())?;
    writer.field(2, prepared.service().service_id().as_str().as_bytes())?;
    writer.field(3, operation.as_str().as_bytes())?;
    writer.field(4, request.method().as_str().as_bytes())?;
    writer.field(
        5,
        &[match endpoint.scheme() {
            EndpointScheme::Http => 0,
            EndpointScheme::Https => 1,
        }],
    )?;
    writer.canonical_host_field(6, endpoint.canonical_host())?;
    writer.field(7, &endpoint.effective_port().to_be_bytes())?;
    writer.field(8, endpoint.base_path().as_bytes())?;
    writer.field(9, request.target().path().as_str().as_bytes())?;
    let query = request.target().query_bytes();
    writer.field(10, &[u8::from(query.is_some())])?;
    writer.field(11, query.unwrap_or_default())?;
    let count = u16::try_from(request.headers().as_slice().len())
        .map_err(|_| FingerprintBuildError::LengthOverflow)?;
    writer.field(12, &count.to_be_bytes())?;
    for header in request.headers().as_slice() {
        writer.lowercase_field(13, header.name().as_str().as_bytes())?;
        writer.field(14, header.value().as_str().as_bytes())?;
    }
    writer.field(15, request.body())?;
    writer.field(16, &[u8::from(scope_present)])?;
    writer.field(17, scope)?;
    Ok(())
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut difference = 0_u8;
    for (left, right) in left.iter().zip(right) {
        difference |= left ^ right;
    }
    difference == 0
}

#[cfg(test)]
mod tests;
