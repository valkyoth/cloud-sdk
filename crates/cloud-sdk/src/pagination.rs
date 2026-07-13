//! Explicit provider-neutral pagination state.

use crate::rate_limit::RateLimit;

/// Pagination validation or transition error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaginationError {
    /// Page numbers are one-based.
    PageZero,
    /// Per-page values are one-based.
    PerPageZero,
    /// The caller-selected page limit must be nonzero.
    PageLimitZero,
    /// The previous-page link is not immediately before the current page.
    InvalidPreviousPage,
    /// The next-page link is not immediately after the current page.
    InvalidNextPage,
    /// The last page contradicts the current page or continuation state.
    InvalidLastPage,
    /// The response page differs from the page the cursor requested.
    UnexpectedPage,
    /// A non-terminal empty page advertised another page.
    EmptyPageWithNextPage,
    /// The decoded entry count contradicts page or total-entry metadata.
    InvalidEntryCount,
    /// Advancing would exceed the caller-selected page limit.
    PageLimitExceeded,
    /// The cursor already reached its terminal page.
    Complete,
}

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

/// Nonzero limit on pages admitted by one cursor.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PageLimit(u32);

impl PageLimit {
    /// Creates a nonzero page limit.
    pub const fn new(value: u32) -> Result<Self, PaginationError> {
        if value == 0 {
            return Err(PaginationError::PageLimitZero);
        }
        Ok(Self(value))
    }

    /// Returns the maximum number of pages.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Validated metadata from one paginated provider response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageMetadata {
    page: PageNumber,
    per_page: u64,
    previous_page: Option<PageNumber>,
    next_page: Option<PageNumber>,
    last_page: Option<PageNumber>,
    total_entries: Option<u64>,
}

impl PageMetadata {
    /// Creates coherent page navigation metadata.
    pub const fn new(
        page: PageNumber,
        per_page: u64,
        previous_page: Option<PageNumber>,
        next_page: Option<PageNumber>,
        last_page: Option<PageNumber>,
        total_entries: Option<u64>,
    ) -> Result<Self, PaginationError> {
        if per_page == 0 {
            return Err(PaginationError::PerPageZero);
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
            if last.0 < page.0 {
                return Err(PaginationError::InvalidLastPage);
            }
            if (page.0 < last.0) != next_page.is_some() {
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
            per_page,
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

    /// Returns the requested maximum entries per page.
    #[must_use]
    pub const fn per_page(self) -> u64 {
        self.per_page
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

/// Accepted page boundary returned to a caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PageBoundary {
    metadata: PageMetadata,
    entries: usize,
    rate_limit: Option<RateLimit>,
}

impl PageBoundary {
    /// Returns the validated provider metadata.
    #[must_use]
    pub const fn metadata(self) -> PageMetadata {
        self.metadata
    }

    /// Returns the entries decoded from this page.
    #[must_use]
    pub const fn entries(self) -> usize {
        self.entries
    }

    /// Returns rate-limit metadata from this response when supplied.
    #[must_use]
    pub const fn rate_limit(self) -> Option<RateLimit> {
        self.rate_limit
    }

    /// Reports whether this page ended iteration.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        self.metadata.next_page.is_none()
    }
}

/// Bounded explicit cursor for caller-driven pagination.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationCursor {
    next_page: Option<PageNumber>,
    pages_seen: u32,
    limit: PageLimit,
}

impl PaginationCursor {
    /// Starts a cursor at a caller-selected page with a hard page limit.
    #[must_use]
    pub const fn new(first_page: PageNumber, limit: PageLimit) -> Self {
        Self {
            next_page: Some(first_page),
            pages_seen: 0,
            limit,
        }
    }

    /// Returns the page the caller must request next.
    pub const fn next_page(self) -> Result<PageNumber, PaginationError> {
        match self.next_page {
            Some(page) => Ok(page),
            None => Err(PaginationError::Complete),
        }
    }

    /// Returns the number of accepted pages.
    #[must_use]
    pub const fn pages_seen(self) -> u32 {
        self.pages_seen
    }

    /// Validates metadata and exact decoded entry count, then records the page.
    pub fn observe(
        &mut self,
        metadata: PageMetadata,
        entries: usize,
        rate_limit: Option<RateLimit>,
    ) -> Result<PageBoundary, PaginationError> {
        let expected = match self.next_page {
            Some(page) => page,
            None => return Err(PaginationError::Complete),
        };
        if metadata.page.0 != expected.0 {
            return Err(PaginationError::UnexpectedPage);
        }
        let entry_count = u64::try_from(entries).map_err(|_| PaginationError::InvalidEntryCount)?;
        if entry_count > metadata.per_page {
            return Err(PaginationError::InvalidEntryCount);
        }
        if let Some(total) = metadata.total_entries {
            let page_offset = metadata
                .page
                .0
                .checked_sub(1)
                .and_then(|page| page.checked_mul(metadata.per_page))
                .ok_or(PaginationError::InvalidEntryCount)?;
            let expected_entries = total.saturating_sub(page_offset).min(metadata.per_page);
            let expected_continuation = total
                > page_offset
                    .checked_add(entry_count)
                    .ok_or(PaginationError::InvalidEntryCount)?;
            if entry_count != expected_entries
                || expected_continuation != metadata.next_page.is_some()
            {
                return Err(PaginationError::InvalidEntryCount);
            }
        }
        if entries == 0 && metadata.next_page.is_some() {
            return Err(PaginationError::EmptyPageWithNextPage);
        }
        let pages_seen = match self.pages_seen.checked_add(1) {
            Some(value) => value,
            None => return Err(PaginationError::PageLimitExceeded),
        };
        if metadata.next_page.is_some() && pages_seen >= self.limit.0 {
            return Err(PaginationError::PageLimitExceeded);
        }
        self.pages_seen = pages_seen;
        self.next_page = metadata.next_page;
        Ok(PageBoundary {
            metadata,
            entries,
            rate_limit,
        })
    }
}

#[cfg(test)]
mod tests;
