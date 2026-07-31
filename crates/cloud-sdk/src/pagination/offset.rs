use crate::rate_limit::RateLimit;

use super::{PaginationBudget, PaginationError, PaginationProgress, SnapshotId};

/// Validated metadata from one offset-based response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OffsetPageMetadata {
    offset: u64,
    page_size: u64,
    next_offset: Option<u64>,
    total_entries: Option<u64>,
}

impl OffsetPageMetadata {
    /// Creates offset metadata without inferring continuation.
    pub const fn new(
        offset: u64,
        page_size: u64,
        next_offset: Option<u64>,
        total_entries: Option<u64>,
    ) -> Result<Self, PaginationError> {
        if page_size == 0 {
            return Err(PaginationError::PageSizeZero);
        }
        if let Some(next) = next_offset
            && next <= offset
        {
            return Err(PaginationError::InvalidNextPage);
        }
        Ok(Self {
            offset,
            page_size,
            next_offset,
            total_entries,
        })
    }

    /// Returns the current offset.
    #[must_use]
    pub const fn offset(self) -> u64 {
        self.offset
    }

    /// Returns the requested page size.
    #[must_use]
    pub const fn page_size(self) -> u64 {
        self.page_size
    }

    /// Returns the next offset when advertised.
    #[must_use]
    pub const fn next_offset(self) -> Option<u64> {
        self.next_offset
    }

    /// Returns total entries when known.
    #[must_use]
    pub const fn total_entries(self) -> Option<u64> {
        self.total_entries
    }
}

/// Accepted offset-page boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OffsetPageBoundary {
    metadata: OffsetPageMetadata,
    entries: usize,
    rate_limit: Option<RateLimit>,
    progress: PaginationProgress,
}

impl OffsetPageBoundary {
    /// Returns validated metadata.
    #[must_use]
    pub const fn metadata(self) -> OffsetPageMetadata {
        self.metadata
    }

    /// Returns decoded entry count.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }

    /// Returns rate-limit metadata when supplied.
    #[must_use]
    pub const fn rate_limit(self) -> Option<RateLimit> {
        self.rate_limit
    }

    /// Returns traversal counters after this response.
    #[must_use]
    pub const fn progress(self) -> PaginationProgress {
        self.progress
    }

    /// Reports whether no continuation remains.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        self.metadata.next_offset.is_none()
    }
}

/// Stateful offset strategy with fixed page size and total metadata.
///
/// This type is intentionally neither `Copy` nor `Clone`.
pub struct OffsetPagination {
    next_offset: Option<u64>,
    expected_page_size: u64,
    expected_total_entries: Option<u64>,
    metadata_initialized: bool,
    budget: PaginationBudget,
}

impl OffsetPagination {
    /// Starts an offset traversal.
    pub fn new(
        first_offset: u64,
        expected_page_size: u64,
        budget: PaginationBudget,
    ) -> Result<Self, PaginationError> {
        if expected_page_size == 0 {
            return Err(PaginationError::PageSizeZero);
        }
        Ok(Self {
            next_offset: Some(first_offset),
            expected_page_size,
            expected_total_entries: None,
            metadata_initialized: false,
            budget,
        })
    }

    /// Returns the offset the caller must request next.
    pub const fn next_offset(&self) -> Result<u64, PaginationError> {
        match self.next_offset {
            Some(offset) => Ok(offset),
            None => Err(PaginationError::Complete),
        }
    }

    /// Returns accepted counters.
    #[must_use]
    pub const fn progress(&self) -> PaginationProgress {
        self.budget.progress()
    }

    /// Validates one response transactionally, then advances the strategy.
    pub fn observe(
        &mut self,
        metadata: OffsetPageMetadata,
        entries: usize,
        rate_limit: Option<RateLimit>,
        snapshot: Option<SnapshotId<'_>>,
    ) -> Result<OffsetPageBoundary, PaginationError> {
        let expected = self.next_offset.ok_or(PaginationError::Complete)?;
        if metadata.offset != expected {
            return Err(PaginationError::UnexpectedPosition);
        }
        if metadata.page_size != self.expected_page_size {
            return Err(PaginationError::PageSizeChanged);
        }
        if self.metadata_initialized && metadata.total_entries != self.expected_total_entries {
            return Err(PaginationError::TraversalChanged);
        }
        let entry_count = u64::try_from(entries).map_err(|_| PaginationError::InvalidEntryCount)?;
        if entry_count > metadata.page_size {
            return Err(PaginationError::InvalidEntryCount);
        }
        let expected_next = metadata
            .offset
            .checked_add(entry_count)
            .ok_or(PaginationError::InvalidEntryCount)?;
        if let Some(next) = metadata.next_offset
            && (entries == 0 || next != expected_next)
        {
            return Err(PaginationError::InvalidNextPage);
        }
        if let Some(total) = metadata.total_entries {
            let continuation = total > expected_next;
            if expected_next > total || continuation != metadata.next_offset.is_some() {
                return Err(PaginationError::InvalidEntryCount);
            }
        }
        let has_continuation = metadata.next_offset.is_some();
        if entries == 0 && has_continuation {
            return Err(PaginationError::EmptyPageWithContinuation);
        }
        let progress = self.budget.admit(entries, has_continuation, snapshot)?;
        self.next_offset = metadata.next_offset;
        if !self.metadata_initialized {
            self.expected_total_entries = metadata.total_entries;
            self.metadata_initialized = true;
        }
        Ok(OffsetPageBoundary {
            metadata,
            entries,
            rate_limit,
            progress,
        })
    }
}

impl core::fmt::Debug for OffsetPagination {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("OffsetPagination")
            .field("next_offset", &self.next_offset)
            .field("expected_page_size", &self.expected_page_size)
            .field("traversal_metadata", &"[redacted]")
            .field("budget", &self.budget)
            .finish()
    }
}
