//! Versioned canonical request identity and caller-supplied strong digests.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;
use subtle::ConstantTimeEq;

use crate::operation::PreparedRequest;
use crate::transport::EndpointIdentity;

mod encoding;
mod writer;
use encoding::{encode, encoded_len};
use writer::Writer;

const DOMAIN: &[u8] = b"cloud-sdk/retry-fingerprint/v2\0";
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
    pub(crate) const fn output_len(self) -> usize {
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
    /// The prepared service policy does not admit the fingerprint endpoint.
    EndpointNotAdmitted,
    /// Sensitive request bodies may only use collision-resistant digests.
    SensitiveBodyRequiresDigest,
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
            Self::EndpointNotAdmitted => "FingerprintBuildError::EndpointNotAdmitted",
            Self::SensitiveBodyRequiresDigest => {
                "FingerprintBuildError::SensitiveBodyRequiresDigest"
            }
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
            Self::EndpointNotAdmitted => "retry fingerprint endpoint is not admitted",
            Self::SensitiveBodyRequiresDigest => {
                "sensitive request body requires a collision-resistant retry digest"
            }
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
pub struct CanonicalFingerprint<'output, 'request> {
    storage: &'output mut [u8],
    len: usize,
    prepared: PreparedRequest<'request>,
}

impl<'output, 'request> CanonicalFingerprint<'output, 'request> {
    /// Returns a redacted comparison reference without exposing diagnostics.
    #[must_use]
    pub fn as_ref(&self) -> FingerprintRef<'_> {
        FingerprintRef(FingerprintKind::Exact(self.as_bytes()))
    }

    /// Binds this fingerprint to the exact prepared request used to build it.
    #[must_use]
    pub fn subject(&self) -> RetrySubject<'request, '_> {
        RetrySubject {
            prepared: &self.prepared,
            fingerprint: self.as_ref(),
        }
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

impl fmt::Debug for CanonicalFingerprint<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CanonicalFingerprint")
            .field("len", &self.len)
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Drop for CanonicalFingerprint<'_, '_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
    }
}

/// Caller-buffer collision-resistant digest cleared on drop.
pub struct FingerprintDigest<'output, 'request> {
    algorithm: DigestAlgorithm,
    storage: &'output mut [u8],
    len: usize,
    prepared: PreparedRequest<'request>,
}

impl<'output, 'request> FingerprintDigest<'output, 'request> {
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
            bytes: self.storage.get(..self.len).unwrap_or_default(),
        })
    }

    /// Binds this digest to the exact prepared request used to build it.
    #[must_use]
    pub fn subject(&self) -> RetrySubject<'request, '_> {
        RetrySubject {
            prepared: &self.prepared,
            fingerprint: self.as_ref(),
        }
    }
}

impl fmt::Debug for FingerprintDigest<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FingerprintDigest")
            .field("algorithm", &self.algorithm)
            .field("bytes", &"[redacted]")
            .finish()
    }
}

impl Drop for FingerprintDigest<'_, '_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
        self.len = 0;
    }
}

/// Borrowed exact or collision-resistant request fingerprint.
#[derive(Clone, Copy)]
pub struct FingerprintRef<'a>(FingerprintKind<'a>);

/// One prepared request inseparably bound to its canonical fingerprint.
///
/// Fields are private and only fingerprint guards can construct this value.
///
/// ```compile_fail
/// use cloud_sdk::operation::PreparedRequest;
/// use cloud_sdk::retry::{FingerprintRef, RetrySubject};
///
/// fn forge<'request, 'fingerprint>(
///     prepared: &'fingerprint PreparedRequest<'request>,
///     fingerprint: FingerprintRef<'fingerprint>,
/// ) -> RetrySubject<'request, 'fingerprint> {
///     RetrySubject { prepared, fingerprint }
/// }
/// ```
#[derive(Clone, Copy)]
pub struct RetrySubject<'request, 'fingerprint> {
    prepared: &'fingerprint PreparedRequest<'request>,
    fingerprint: FingerprintRef<'fingerprint>,
}

impl<'request, 'fingerprint> RetrySubject<'request, 'fingerprint> {
    pub(crate) const fn prepared(self) -> &'fingerprint PreparedRequest<'request> {
        self.prepared
    }

    pub(crate) const fn fingerprint(self) -> FingerprintRef<'fingerprint> {
        self.fingerprint
    }
}

impl fmt::Debug for RetrySubject<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RetrySubject")
            .field("prepared", &self.prepared)
            .field("fingerprint", &"[redacted]")
            .finish()
    }
}

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
}

impl fmt::Debug for FingerprintRef<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FingerprintRef([redacted])")
    }
}

/// Builds exact versioned canonical bytes into caller-owned storage.
pub fn build_canonical_fingerprint<'output, 'request>(
    request: PreparedRequest<'request>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    output: &'output mut [u8],
) -> Result<CanonicalFingerprint<'output, 'request>, FingerprintBuildError<core::convert::Infallible>>
{
    sanitize_bytes(output);
    if request.body_sensitivity().requires_digest() {
        return Err(FingerprintBuildError::SensitiveBodyRequiresDigest);
    }
    build_canonical_fingerprint_inner(request, endpoint, scope, output)
}

#[allow(
    clippy::large_types_passed_by_value,
    reason = "the returned fingerprint must own the complete prepared request"
)]
fn build_canonical_fingerprint_inner<'output, 'request>(
    request: PreparedRequest<'request>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    output: &'output mut [u8],
) -> Result<CanonicalFingerprint<'output, 'request>, FingerprintBuildError<core::convert::Infallible>>
{
    sanitize_bytes(output);
    if !request.service().endpoint_policy().admits(endpoint) {
        return Err(FingerprintBuildError::EndpointNotAdmitted);
    }
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
        prepared: request,
    })
}

/// Builds and hashes canonical bytes, clearing caller scratch on every path.
pub fn build_fingerprint_digest<'output, 'request, H: FingerprintHasher>(
    request: PreparedRequest<'request>,
    endpoint: EndpointIdentity<'_>,
    scope: FingerprintScope<'_>,
    scratch: &mut [u8],
    output: &'output mut [u8],
    hasher: &H,
) -> Result<FingerprintDigest<'output, 'request>, FingerprintBuildError<H::Error>> {
    sanitize_bytes(output);
    let canonical = build_canonical_fingerprint_inner(request, endpoint, scope, scratch)
        .map_err(map_infallible_error)?;
    let algorithm = hasher.algorithm();
    let expected = algorithm.output_len();
    let mut digest = FingerprintDigest {
        algorithm,
        storage: output,
        len: 0,
        prepared: request,
    };
    let output = digest
        .storage
        .get_mut(..expected)
        .ok_or(FingerprintBuildError::OutputTooSmall)?;
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
        FingerprintBuildError::EndpointNotAdmitted => FingerprintBuildError::EndpointNotAdmitted,
        FingerprintBuildError::SensitiveBodyRequiresDigest => {
            FingerprintBuildError::SensitiveBodyRequiresDigest
        }
        FingerprintBuildError::InvalidDigestLength => FingerprintBuildError::InvalidDigestLength,
        FingerprintBuildError::Hasher(never) => match never {},
    }
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    left.len() == right.len() && bool::from(left.ct_eq(right))
}

#[cfg(test)]
mod tests;
