use crate::rate_limit::{
    DelaySeconds, MAX_QUOTA_BUCKETS, QuotaBucket, QuotaBucketId, QuotaBuckets, QuotaError,
    QuotaExtension, QuotaReset,
};
use core::fmt::{self, Write};

fn bucket(id: &[u8], remaining: u64) -> Result<QuotaBucket, QuotaError> {
    QuotaBucket::new(
        QuotaBucketId::new(id)?,
        10,
        remaining,
        QuotaReset::After(DelaySeconds::new(5)),
    )
}

#[test]
fn large_bucket_accessors_borrow_instead_of_copying_the_bucket() {
    let _: fn(&QuotaBucket) -> QuotaBucketId = QuotaBucket::id;
    let _: fn(&QuotaBucket) -> u64 = QuotaBucket::limit;
    let _: fn(&QuotaBucket) -> u64 = QuotaBucket::remaining;
    let _: fn(&QuotaBucket) -> QuotaReset = QuotaBucket::reset;
    let _: fn(&QuotaBucket) -> bool = QuotaBucket::is_exhausted;
}

#[test]
fn validates_bucket_invariants_and_distinct_ids() {
    let id = QuotaBucketId::new(b"project-hourly");
    assert!(id.is_ok());
    let Ok(id) = id else { return };
    assert_eq!(
        QuotaBucket::new(id, 0, 0, QuotaReset::Unknown),
        Err(QuotaError::LimitZero)
    );
    assert_eq!(
        QuotaBucket::new(id, 10, 11, QuotaReset::Unknown),
        Err(QuotaError::RemainingExceedsLimit)
    );
    let mut buckets = QuotaBuckets::new();
    assert_eq!(
        buckets.try_push(bucket(b"same", 1).unwrap_or_else(|_| unreachable!())),
        Ok(())
    );
    assert_eq!(
        buckets.try_push(bucket(b"same", 0).unwrap_or_else(|_| unreachable!())),
        Err(QuotaError::DuplicateBucket)
    );
}

#[test]
fn enforces_bucket_capacity() {
    let ids: [&[u8]; MAX_QUOTA_BUCKETS + 1] =
        [b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"h", b"i"];
    let mut buckets = QuotaBuckets::new();
    for id in ids.iter().take(MAX_QUOTA_BUCKETS) {
        let Ok(value) = bucket(id, 1) else { return };
        assert_eq!(buckets.try_push(value), Ok(()));
    }
    let Ok(extra) = bucket(ids.get(MAX_QUOTA_BUCKETS).copied().unwrap_or_default(), 1) else {
        return;
    };
    assert_eq!(buckets.try_push(extra), Err(QuotaError::TooManyBuckets));
}

#[test]
fn preserves_bounded_extensions_with_redacted_debug() {
    let Ok(mut value) = bucket(b"partitioned", 1) else {
        return;
    };
    let extension = QuotaExtension::new(b"partition-key", b"customer-42");
    assert!(extension.is_ok());
    let Ok(extension) = extension else { return };
    assert_eq!(value.try_add_extension(extension), Ok(()));
    let retained = value.extensions().next();
    assert_eq!(
        retained.map(QuotaExtension::name),
        Some(b"partition-key".as_slice())
    );
    assert_eq!(
        retained.map(QuotaExtension::value),
        Some(b"customer-42".as_slice())
    );
    let mut debug = DebugBuffer::new();
    assert!(write!(&mut debug, "{value:?}").is_ok());
    assert!(!debug.as_str().contains("customer-42"));
    assert!(debug.as_str().contains("[redacted]"));

    assert_eq!(
        value.try_add_extension(extension),
        Err(QuotaError::DuplicateExtension)
    );
}

struct DebugBuffer {
    bytes: [u8; 512],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 512],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
