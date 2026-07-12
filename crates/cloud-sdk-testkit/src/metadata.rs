//! Provider-neutral response metadata fixtures.

/// Fixture metadata validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureMetadataError {
    /// Pagination page and page-size values must be nonzero.
    PaginationZero,
    /// Current page must not exceed the last page.
    PageAfterLast,
    /// Remaining requests must not exceed the rate limit.
    RemainingExceedsLimit,
    /// Rate limits must be nonzero.
    RateLimitZero,
    /// Action progress must be in `0..=100`.
    InvalidProgress,
}

/// Pagination metadata for a deterministic response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationFixture {
    page: u64,
    per_page: u64,
    total_entries: u64,
    last_page: u64,
}

impl PaginationFixture {
    /// Creates coherent pagination metadata.
    pub const fn new(
        page: u64,
        per_page: u64,
        total_entries: u64,
        last_page: u64,
    ) -> Result<Self, FixtureMetadataError> {
        if page == 0 || per_page == 0 || last_page == 0 {
            return Err(FixtureMetadataError::PaginationZero);
        }
        if page > last_page {
            return Err(FixtureMetadataError::PageAfterLast);
        }
        Ok(Self {
            page,
            per_page,
            total_entries,
            last_page,
        })
    }

    /// Returns the current page.
    #[must_use]
    pub const fn page(self) -> u64 {
        self.page
    }

    /// Returns the requested page size.
    #[must_use]
    pub const fn per_page(self) -> u64 {
        self.per_page
    }

    /// Returns the total entry count.
    #[must_use]
    pub const fn total_entries(self) -> u64 {
        self.total_entries
    }

    /// Returns the last page number.
    #[must_use]
    pub const fn last_page(self) -> u64 {
        self.last_page
    }
}

/// Provider-neutral action lifecycle fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ActionState {
    /// Action remains in progress.
    Running,
    /// Action completed successfully.
    Success,
    /// Action completed with an error.
    Error,
}

/// Action metadata for polling tests.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ActionFixture {
    state: ActionState,
    progress: u8,
}

impl ActionFixture {
    /// Creates bounded action metadata.
    pub const fn new(state: ActionState, progress: u8) -> Result<Self, FixtureMetadataError> {
        if progress > 100 {
            return Err(FixtureMetadataError::InvalidProgress);
        }
        Ok(Self { state, progress })
    }

    /// Returns the lifecycle state.
    #[must_use]
    pub const fn state(self) -> ActionState {
        self.state
    }

    /// Returns progress in `0..=100`.
    #[must_use]
    pub const fn progress(self) -> u8 {
        self.progress
    }
}

/// Rate-limit metadata fixture.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RateLimitFixture {
    limit: u64,
    remaining: u64,
    reset_at: Option<u64>,
}

impl RateLimitFixture {
    /// Creates coherent rate-limit metadata.
    pub const fn new(
        limit: u64,
        remaining: u64,
        reset_at: Option<u64>,
    ) -> Result<Self, FixtureMetadataError> {
        if limit == 0 {
            return Err(FixtureMetadataError::RateLimitZero);
        }
        if remaining > limit {
            return Err(FixtureMetadataError::RemainingExceedsLimit);
        }
        Ok(Self {
            limit,
            remaining,
            reset_at,
        })
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

    /// Returns an optional provider-specific reset timestamp.
    #[must_use]
    pub const fn reset_at(self) -> Option<u64> {
        self.reset_at
    }
}
