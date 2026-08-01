use core::fmt;

use super::{DelaySeconds, QuotaExtension, WallClockTimestamp};

/// Maximum provider quota buckets retained from one response.
pub const MAX_QUOTA_BUCKETS: usize = 8;
/// Maximum informational extensions retained per quota bucket.
pub const MAX_QUOTA_EXTENSIONS_PER_BUCKET: usize = 4;
/// Maximum exact-byte provider quota bucket identity length.
pub const MAX_QUOTA_BUCKET_ID_BYTES: usize = 64;

/// Provider-defined quota bucket identity in fixed-capacity storage.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct QuotaBucketId {
    bytes: [u8; MAX_QUOTA_BUCKET_ID_BYTES],
    len: u8,
}

impl QuotaBucketId {
    /// Copies one stable ASCII bucket identity.
    pub fn new(value: &[u8]) -> Result<Self, QuotaError> {
        if value.is_empty() {
            return Err(QuotaError::BucketIdEmpty);
        }
        if value.len() > MAX_QUOTA_BUCKET_ID_BYTES {
            return Err(QuotaError::BucketIdTooLong);
        }
        if !value
            .iter()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-_.:/".contains(byte))
        {
            return Err(QuotaError::InvalidBucketId);
        }
        let mut result = Self {
            bytes: [0; MAX_QUOTA_BUCKET_ID_BYTES],
            len: 0,
        };
        let target = result
            .bytes
            .get_mut(..value.len())
            .ok_or(QuotaError::BucketIdTooLong)?;
        target.copy_from_slice(value);
        result.len = u8::try_from(value.len()).map_err(|_| QuotaError::BucketIdTooLong)?;
        Ok(result)
    }

    /// Returns the exact provider bucket identity.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.get(..usize::from(self.len)).unwrap_or_default()
    }
}

impl fmt::Debug for QuotaBucketId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_tuple("QuotaBucketId")
            .field(&self.as_bytes())
            .finish()
    }
}

/// Reset semantics for one provider quota bucket.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaReset {
    /// Relative delay from the caller-supplied observation time.
    After(DelaySeconds),
    /// Absolute Unix wall-clock reset timestamp.
    At(WallClockTimestamp),
    /// Provider exposed the bucket without actionable reset metadata.
    Unknown,
}

/// One coherent provider quota bucket.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuotaBucket {
    id: QuotaBucketId,
    limit: u64,
    remaining: u64,
    reset: QuotaReset,
    extensions: [Option<QuotaExtension>; MAX_QUOTA_EXTENSIONS_PER_BUCKET],
    extension_len: u8,
}

impl QuotaBucket {
    /// Creates a bucket with no informational extensions.
    pub const fn new(
        id: QuotaBucketId,
        limit: u64,
        remaining: u64,
        reset: QuotaReset,
    ) -> Result<Self, QuotaError> {
        if limit == 0 {
            return Err(QuotaError::LimitZero);
        }
        if remaining > limit {
            return Err(QuotaError::RemainingExceedsLimit);
        }
        Ok(Self {
            id,
            limit,
            remaining,
            reset,
            extensions: [None; MAX_QUOTA_EXTENSIONS_PER_BUCKET],
            extension_len: 0,
        })
    }

    /// Appends one unique informational extension.
    pub fn try_add_extension(&mut self, extension: QuotaExtension) -> Result<(), QuotaError> {
        if self
            .extensions()
            .any(|current| current.name() == extension.name())
        {
            return Err(QuotaError::DuplicateExtension);
        }
        let index = usize::from(self.extension_len);
        let slot = self
            .extensions
            .get_mut(index)
            .ok_or(QuotaError::TooManyExtensions)?;
        *slot = Some(extension);
        self.extension_len = self
            .extension_len
            .checked_add(1)
            .ok_or(QuotaError::TooManyExtensions)?;
        Ok(())
    }

    /// Returns the bucket identity.
    #[must_use]
    pub const fn id(self) -> QuotaBucketId {
        self.id
    }
    /// Returns the request limit.
    #[must_use]
    pub const fn limit(self) -> u64 {
        self.limit
    }
    /// Returns the remaining request count.
    #[must_use]
    pub const fn remaining(self) -> u64 {
        self.remaining
    }
    /// Returns the reset semantics.
    #[must_use]
    pub const fn reset(self) -> QuotaReset {
        self.reset
    }
    /// Reports whether this bucket is exhausted.
    #[must_use]
    pub const fn is_exhausted(self) -> bool {
        self.remaining == 0
    }
    /// Iterates retained informational extensions.
    pub fn extensions(&self) -> impl Iterator<Item = &QuotaExtension> {
        self.extensions.iter().filter_map(Option::as_ref)
    }
}

/// Fixed-capacity set of distinct provider quota buckets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QuotaBuckets {
    buckets: [Option<QuotaBucket>; MAX_QUOTA_BUCKETS],
    len: u8,
}

impl QuotaBuckets {
    /// Creates an empty quota set.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            buckets: [None; MAX_QUOTA_BUCKETS],
            len: 0,
        }
    }

    /// Appends one bucket atomically.
    pub fn try_push(&mut self, bucket: QuotaBucket) -> Result<(), QuotaError> {
        if self.iter().any(|current| current.id == bucket.id) {
            return Err(QuotaError::DuplicateBucket);
        }
        let index = usize::from(self.len);
        let slot = self
            .buckets
            .get_mut(index)
            .ok_or(QuotaError::TooManyBuckets)?;
        *slot = Some(bucket);
        self.len = self.len.checked_add(1).ok_or(QuotaError::TooManyBuckets)?;
        Ok(())
    }

    /// Returns the number of retained buckets.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len as usize
    }
    /// Reports whether no quota bucket was supplied.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }
    /// Iterates retained buckets in provider order.
    pub fn iter(&self) -> impl Iterator<Item = &QuotaBucket> {
        self.buckets.iter().filter_map(Option::as_ref)
    }
}

impl Default for QuotaBuckets {
    fn default() -> Self {
        Self::new()
    }
}

/// Invalid quota bucket or bounded collection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QuotaError {
    /// A bucket identity was empty.
    BucketIdEmpty,
    /// A bucket identity exceeded its fixed bound.
    BucketIdTooLong,
    /// A bucket identity contained unsupported bytes.
    InvalidBucketId,
    /// A bucket limit was zero.
    LimitZero,
    /// Remaining requests exceeded the bucket limit.
    RemainingExceedsLimit,
    /// More than the bounded number of buckets was supplied.
    TooManyBuckets,
    /// A bucket identity appeared more than once.
    DuplicateBucket,
    /// More than the bounded number of extensions was supplied.
    TooManyExtensions,
    /// An extension name appeared more than once in a bucket.
    DuplicateExtension,
}

impl_static_error!(QuotaError,
    Self::BucketIdEmpty => "quota bucket identity is empty",
    Self::BucketIdTooLong => "quota bucket identity exceeds its length limit",
    Self::InvalidBucketId => "quota bucket identity contains an invalid byte",
    Self::LimitZero => "quota bucket limit must be nonzero",
    Self::RemainingExceedsLimit => "quota bucket remaining count exceeds its limit",
    Self::TooManyBuckets => "quota bucket count exceeds its fixed capacity",
    Self::DuplicateBucket => "quota bucket identity is duplicated",
    Self::TooManyExtensions => "quota extension count exceeds its fixed capacity",
    Self::DuplicateExtension => "quota extension name is duplicated",
);
