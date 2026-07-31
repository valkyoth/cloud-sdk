use cloud_sdk_sanitization::sanitize_value;

use super::{MAX_OPAQUE_STATE_BYTES, PaginationError};

/// Hard traversal limits selected before the first request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationLimits {
    max_requests: u32,
    max_items: u64,
    max_state_bytes: usize,
}

impl PaginationLimits {
    /// Creates nonzero request, item, and opaque-state limits.
    pub const fn new(
        max_requests: u32,
        max_items: u64,
        max_state_bytes: usize,
    ) -> Result<Self, PaginationError> {
        if max_requests == 0 || max_items == 0 || max_state_bytes == 0 {
            return Err(PaginationError::ZeroLimit);
        }
        if max_state_bytes > MAX_OPAQUE_STATE_BYTES {
            return Err(PaginationError::StateLimitTooLarge);
        }
        Ok(Self {
            max_requests,
            max_items,
            max_state_bytes,
        })
    }

    /// Returns the maximum admitted requests.
    #[must_use]
    pub const fn max_requests(self) -> u32 {
        self.max_requests
    }

    /// Returns the maximum admitted items.
    #[must_use]
    pub const fn max_items(self) -> u64 {
        self.max_items
    }

    /// Returns the maximum opaque state length.
    #[must_use]
    pub const fn max_state_bytes(self) -> usize {
        self.max_state_bytes
    }
}

/// Provider snapshot identifier supplied by a provider decoder.
#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct SnapshotId(u64);

impl SnapshotId {
    /// Creates an opaque nonzero snapshot identifier.
    pub const fn new(value: u64) -> Result<Self, PaginationError> {
        if value == 0 {
            return Err(PaginationError::ZeroLimit);
        }
        Ok(Self(value))
    }

    /// Returns the provider-decoder identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

impl core::fmt::Debug for SnapshotId {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("SnapshotId([redacted])")
    }
}

/// Snapshot presence policy fixed for one traversal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnapshotPolicy {
    /// Every response must provide one stable identifier.
    Required,
    /// Presence is provider-dependent but cannot change after the first page.
    Optional,
    /// Responses must not provide snapshot identifiers.
    Forbidden,
}

/// Public nonsensitive traversal counters.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationProgress {
    requests: u32,
    items: u64,
}

impl PaginationProgress {
    /// Returns accepted response count.
    #[must_use]
    pub const fn requests(self) -> u32 {
        self.requests
    }

    /// Returns accepted item count.
    #[must_use]
    pub const fn items(self) -> u64 {
        self.items
    }
}

/// Transactional request, item, and snapshot budget.
///
/// This state is intentionally neither `Copy` nor `Clone`.
///
/// ```compile_fail
/// use cloud_sdk::pagination::PaginationBudget;
/// fn require_copy<T: Copy>() {}
/// require_copy::<PaginationBudget>();
/// ```
pub struct PaginationBudget {
    limits: PaginationLimits,
    snapshot_policy: SnapshotPolicy,
    snapshot: Option<SnapshotId>,
    snapshot_initialized: bool,
    requests: u32,
    items: u64,
}

impl PaginationBudget {
    /// Starts an empty traversal budget.
    #[must_use]
    pub const fn new(limits: PaginationLimits, snapshot_policy: SnapshotPolicy) -> Self {
        Self {
            limits,
            snapshot_policy,
            snapshot: None,
            snapshot_initialized: false,
            requests: 0,
            items: 0,
        }
    }

    /// Returns the immutable limits.
    #[must_use]
    pub const fn limits(&self) -> PaginationLimits {
        self.limits
    }

    /// Returns accepted counters.
    #[must_use]
    pub const fn progress(&self) -> PaginationProgress {
        PaginationProgress {
            requests: self.requests,
            items: self.items,
        }
    }

    /// Transactionally admits one decoded response into the hard budget.
    ///
    /// Cursor, marker, and provider-link traversals call this once for every
    /// response before following a continuation. On failure, counters and
    /// snapshot state remain unchanged.
    pub fn admit(
        &mut self,
        entries: usize,
        has_continuation: bool,
        snapshot: Option<SnapshotId>,
    ) -> Result<PaginationProgress, PaginationError> {
        self.validate_snapshot(snapshot)?;
        let requests = self
            .requests
            .checked_add(1)
            .ok_or(PaginationError::RequestBudgetExceeded)?;
        if requests > self.limits.max_requests
            || has_continuation && requests == self.limits.max_requests
        {
            return Err(PaginationError::RequestBudgetExceeded);
        }
        let entries = u64::try_from(entries).map_err(|_| PaginationError::ItemBudgetExceeded)?;
        let items = self
            .items
            .checked_add(entries)
            .ok_or(PaginationError::ItemBudgetExceeded)?;
        if items > self.limits.max_items {
            return Err(PaginationError::ItemBudgetExceeded);
        }
        self.requests = requests;
        self.items = items;
        if !self.snapshot_initialized {
            self.snapshot = snapshot;
            self.snapshot_initialized = true;
        }
        Ok(self.progress())
    }

    fn validate_snapshot(&self, snapshot: Option<SnapshotId>) -> Result<(), PaginationError> {
        match self.snapshot_policy {
            SnapshotPolicy::Required if snapshot.is_none() => {
                Err(PaginationError::SnapshotRequired)
            }
            SnapshotPolicy::Forbidden if snapshot.is_some() => {
                Err(PaginationError::SnapshotForbidden)
            }
            _ if self.snapshot_initialized && snapshot != self.snapshot => {
                Err(PaginationError::SnapshotChanged)
            }
            _ => Ok(()),
        }
    }
}

impl Drop for PaginationBudget {
    fn drop(&mut self) {
        if let Some(snapshot) = &mut self.snapshot {
            sanitize_value(&mut snapshot.0);
        }
        self.snapshot = None;
        sanitize_value(&mut self.snapshot_initialized);
        sanitize_value(&mut self.requests);
        sanitize_value(&mut self.items);
    }
}

impl core::fmt::Debug for PaginationBudget {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PaginationBudget")
            .field("limits", &self.limits)
            .field("snapshot", &"[redacted]")
            .field("progress", &self.progress())
            .finish()
    }
}
