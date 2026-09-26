use super::{MAX_PUBLISH_ARCHIVE_BYTES, PublishError as Error, PublishMetadata};
use crate::credentials::{ApiToken, CredentialOrigin, TrustedPublishingToken};
use cloud_sdk::transport::{StreamFraming, StreamKind, StreamLimits, StreamPolicy, StreamSinkMode};

/// Validated immutable metadata and exact caller-provided archive length.
pub struct PublishRequest<'a> {
    pub(super) metadata: PublishMetadata<'a>,
    pub(super) archive: u64,
    pub(super) policy: StreamPolicy,
}
impl<'a> PublishRequest<'a> {
    pub(crate) fn crate_matches(&self, name: &str) -> Result<bool, Error> {
        self.metadata
            .fields
            .required("name")?
            .with_text(|s| s == name)
    }
    /// The source must contain exactly `archive_bytes` compressed `.crate` bytes.
    /// No packaging, unpacking, manifest verification or implicit retries occur.
    pub fn new(
        metadata: PublishMetadata<'a>,
        archive_bytes: u64,
        limits: StreamLimits,
    ) -> Result<Self, Error> {
        if archive_bytes == 0
            || archive_bytes > MAX_PUBLISH_ARCHIVE_BYTES
            || u32::try_from(archive_bytes).is_err()
        {
            return Err(Error::Limit);
        }
        let meta = u32::try_from(metadata.bytes.len()).map_err(|_| Error::Limit)?;
        let total = u64::from(meta)
            .checked_add(archive_bytes)
            .and_then(|n| n.checked_add(8))
            .ok_or(Error::Limit)?;
        let policy = StreamPolicy::new(
            StreamKind::FiniteUpload,
            StreamFraming::Declared(total),
            StreamSinkMode::Direct,
            limits,
        )
        .map_err(|_| Error::Limit)?;
        Ok(Self {
            metadata,
            archive: archive_bytes,
            policy,
        })
    }
    /// Exact complete binary framing size, without duplicating the archive.
    pub const fn content_length(&self) -> u64 {
        match self.policy.framing() {
            StreamFraming::Declared(n) => n,
            _ => 0,
        }
    }
    /// Exact compressed archive length, excluding metadata and length words.
    pub const fn archive_length(&self) -> u64 {
        self.archive
    }
    /// Never replay a publication, even after an ambiguous response or timeout.
    pub const fn permits_automatic_retry(&self) -> bool {
        false
    }
    /// Explicitly authorize one attempt with an ordinary API token.
    pub fn confirm_api(self, credential: &'a ApiToken) -> PublishPermit<'a> {
        PublishPermit {
            request: self,
            credential: PublishCredential::Api(credential),
        }
    }
    /// Explicitly authorize one attempt with a temporary trusted-publishing token.
    /// Its expiry, issuer and crate permissions must be qualified separately.
    pub fn confirm_trusted(self, credential: &'a TrustedPublishingToken) -> PublishPermit<'a> {
        PublishPermit {
            request: self,
            credential: PublishCredential::Trusted(credential),
        }
    }
}
pub(super) enum PublishCredential<'a> {
    Api(&'a ApiToken),
    Trusted(&'a TrustedPublishingToken),
}
impl PublishCredential<'_> {
    pub(super) fn origin(&self) -> CredentialOrigin {
        match self {
            Self::Api(t) => t.origin(),
            Self::Trusted(t) => t.origin(),
        }
    }
}
/// Non-cloneable, consumed publish authority. A new permit requires fresh consent.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::publishing::PublishPermit;
/// fn replay(p: PublishPermit<'_>) { let _ = p.clone(); }
/// ```
pub struct PublishPermit<'a> {
    pub(super) request: PublishRequest<'a>,
    pub(super) credential: PublishCredential<'a>,
}
impl PublishPermit<'_> {
    /// Immutable destination; no secret bytes are returned.
    pub fn origin(&self) -> CredentialOrigin {
        self.credential.origin()
    }
    /// Exact declared body length for the trusted streaming adapter.
    pub const fn content_length(&self) -> u64 {
        self.request.content_length()
    }
}
impl core::fmt::Debug for PublishRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PublishRequest([redacted])")
    }
}
impl core::fmt::Debug for PublishPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("PublishPermit([redacted])")
    }
}
