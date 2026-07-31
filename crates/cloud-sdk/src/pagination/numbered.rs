use crate::rate_limit::RateLimit;

use super::{PaginationBudget, PaginationError, PaginationProgress, SnapshotId};

/// One-based provider-neutral page number.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PageNumber(u64);

impl PageNumber {
    /// Creates a one-based page number.
    pub const fn new(value: u64) -> Result<Self, PaginationError> {
        if value == 0 {
            return Err(PaginationError::PageZero);
        }
        Ok(Self(value))
    }

    /// Returns the raw page number.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Validated metadata from one numbered-page response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberedPageMetadata {
    page: PageNumber,
    page_size: u64,
    previous_page: Option<PageNumber>,
    next_page: Option<PageNumber>,
    last_page: Option<PageNumber>,
    total_entries: Option<u64>,
}

impl NumberedPageMetadata {
    /// Creates coherent numbered navigation metadata.
    pub const fn new(
        page: PageNumber,
        page_size: u64,
        previous_page: Option<PageNumber>,
        next_page: Option<PageNumber>,
        last_page: Option<PageNumber>,
        total_entries: Option<u64>,
    ) -> Result<Self, PaginationError> {
        if page_size == 0 {
            return Err(PaginationError::PageSizeZero);
        }
        if let Some(previous) = previous_page {
            let Some(expected) = page.0.checked_sub(1) else {
                return Err(PaginationError::InvalidPreviousPage);
            };
            if previous.0 != expected {
                return Err(PaginationError::InvalidPreviousPage);
            }
        }
        if let Some(next) = next_page {
            let Some(expected) = page.0.checked_add(1) else {
                return Err(PaginationError::InvalidNextPage);
            };
            if next.0 != expected {
                return Err(PaginationError::InvalidNextPage);
            }
        }
        if let Some(last) = last_page {
            if last.0 < page.0 || (page.0 < last.0) != next_page.is_some() {
                return Err(PaginationError::InvalidLastPage);
            }
            if let Some(next) = next_page
                && next.0 > last.0
            {
                return Err(PaginationError::InvalidLastPage);
            }
        }
        Ok(Self {
            page,
            page_size,
            previous_page,
            next_page,
            last_page,
            total_entries,
        })
    }

    /// Returns the current page.
    #[must_use]
    pub const fn page(self) -> PageNumber {
        self.page
    }

    /// Returns the response page size.
    #[must_use]
    pub const fn page_size(self) -> u64 {
        self.page_size
    }

    /// Returns the previous page when advertised.
    #[must_use]
    pub const fn previous_page(self) -> Option<PageNumber> {
        self.previous_page
    }

    /// Returns the next page when advertised.
    #[must_use]
    pub const fn next_page(self) -> Option<PageNumber> {
        self.next_page
    }

    /// Returns the final page when known.
    #[must_use]
    pub const fn last_page(self) -> Option<PageNumber> {
        self.last_page
    }

    /// Returns the total matching entries when known.
    #[must_use]
    pub const fn total_entries(self) -> Option<u64> {
        self.total_entries
    }
}

/// Accepted numbered-page boundary.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumberedPageBoundary {
    metadata: NumberedPageMetadata,
    entries: usize,
    rate_limit: Option<RateLimit>,
    progress: PaginationProgress,
}

impl NumberedPageBoundary {
    /// Returns validated provider metadata.
    #[must_use]
    pub const fn metadata(self) -> NumberedPageMetadata {
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

    /// Returns traversal counters after this page.
    #[must_use]
    pub const fn progress(self) -> PaginationProgress {
        self.progress
    }

    /// Reports whether this page ended iteration.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        self.metadata.next_page.is_none()
    }
}

/// Stateful numbered-page strategy with locked traversal metadata.
///
/// This type is intentionally neither `Copy` nor `Clone`.
pub struct NumberedPagination {
    next_page: Option<PageNumber>,
    expected_page_size: u64,
    expected_total_entries: Option<u64>,
    expected_last_page: Option<PageNumber>,
    metadata_initialized: bool,
    budget: PaginationBudget,
}

impl NumberedPagination {
    /// Starts a numbered traversal.
    pub fn new(
        first_page: PageNumber,
        expected_page_size: u64,
        budget: PaginationBudget,
    ) -> Result<Self, PaginationError> {
        if expected_page_size == 0 {
            return Err(PaginationError::PageSizeZero);
        }
        Ok(Self {
            next_page: Some(first_page),
            expected_page_size,
            expected_total_entries: None,
            expected_last_page: None,
            metadata_initialized: false,
            budget,
        })
    }

    /// Returns the page the caller must request next.
    pub const fn next_page(&self) -> Result<PageNumber, PaginationError> {
        match self.next_page {
            Some(page) => Ok(page),
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
        metadata: NumberedPageMetadata,
        entries: usize,
        rate_limit: Option<RateLimit>,
        snapshot: Option<SnapshotId>,
    ) -> Result<NumberedPageBoundary, PaginationError> {
        let expected = self.next_page.ok_or(PaginationError::Complete)?;
        if metadata.page != expected {
            return Err(PaginationError::UnexpectedPosition);
        }
        if metadata.page_size != self.expected_page_size {
            return Err(PaginationError::PageSizeChanged);
        }
        if self.metadata_initialized
            && (metadata.total_entries != self.expected_total_entries
                || metadata.last_page != self.expected_last_page)
        {
            return Err(PaginationError::TraversalChanged);
        }
        validate_entry_count(metadata, entries)?;
        let has_continuation = metadata.next_page.is_some();
        if entries == 0 && has_continuation {
            return Err(PaginationError::EmptyPageWithContinuation);
        }
        let progress = self.budget.admit(entries, has_continuation, snapshot)?;
        self.next_page = metadata.next_page;
        if !self.metadata_initialized {
            self.expected_total_entries = metadata.total_entries;
            self.expected_last_page = metadata.last_page;
            self.metadata_initialized = true;
        }
        Ok(NumberedPageBoundary {
            metadata,
            entries,
            rate_limit,
            progress,
        })
    }
}

fn validate_entry_count(
    metadata: NumberedPageMetadata,
    entries: usize,
) -> Result<(), PaginationError> {
    let entry_count = u64::try_from(entries).map_err(|_| PaginationError::InvalidEntryCount)?;
    if entry_count > metadata.page_size {
        return Err(PaginationError::InvalidEntryCount);
    }
    if let Some(total) = metadata.total_entries {
        let offset = metadata
            .page
            .0
            .checked_sub(1)
            .and_then(|page| page.checked_mul(metadata.page_size))
            .ok_or(PaginationError::InvalidEntryCount)?;
        let expected = total.saturating_sub(offset).min(metadata.page_size);
        let continuation = total
            > offset
                .checked_add(entry_count)
                .ok_or(PaginationError::InvalidEntryCount)?;
        if entry_count != expected || continuation != metadata.next_page.is_some() {
            return Err(PaginationError::InvalidEntryCount);
        }
    }
    Ok(())
}

impl core::fmt::Debug for NumberedPagination {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NumberedPagination")
            .field("next_page", &self.next_page)
            .field("expected_page_size", &self.expected_page_size)
            .field("traversal_metadata", &"[redacted]")
            .field("budget", &self.budget)
            .finish()
    }
}
