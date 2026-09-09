//! Explicit bounded pagination; no requests, sleeps or retries are implicit.

use crate::{
    endpoint::OfficialCratesIoEndpoint,
    query::{ApiPath, Page, Query, QueryError, Seek},
};
use cloud_sdk::pagination::{PaginationBudget, PaginationLimits, SnapshotPolicy};
use core::fmt;
mod compare;
mod link;
pub use link::PageLink;

/// Payload-free continuation rejection.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaginationError {
    /// Invalid, duplicated or conflicting continuation state.
    Invalid,
    /// A link tried to change the origin, path or non-pagination query.
    Binding,
    /// Page depth, request, item, state or output limit exceeded.
    Limit,
    /// A numbered or seek link did not move in its declared direction.
    Progress,
}
impl fmt::Display for PaginationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Invalid => "invalid crates.io continuation",
            Self::Binding => "crates.io continuation changed its request binding",
            Self::Limit => "crates.io pagination limit exceeded",
            Self::Progress => "crates.io continuation made invalid progress",
        })
    }
}
impl core::error::Error for PaginationError {}
impl From<QueryError> for PaginationError {
    fn from(_: QueryError) -> Self {
        Self::Invalid
    }
}

/// Expected direction of a response meta link.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Direction {
    /// Following page.
    Next,
    /// Preceding page.
    Previous,
}

/// Typed page identity, with mutually exclusive numbered and opaque variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cursor<'a> {
    /// Canonical bounded numbered page.
    Page(Page),
    /// Provider-issued opaque seek token.
    Seek(Seek<'a>),
}

/// Validated complete response links, including validation of the unused direction.
#[derive(Debug)]
pub struct MetaLinks<'a> {
    next: Option<PageLink<'a>>,
    previous: Option<PageLink<'a>>,
}
impl<'a> MetaLinks<'a> {
    /// Validates both optional links against the same complete current request.
    pub fn new(
        endpoint: OfficialCratesIoEndpoint,
        path: ApiPath<'a>,
        query: Query<'a>,
        next: Option<&'a str>,
        previous: Option<&'a str>,
    ) -> Result<Self, PaginationError> {
        let next = next
            .map(|v| PageLink::new(endpoint, path, query, v, Direction::Next))
            .transpose()?;
        let previous = previous
            .map(|v| PageLink::new(endpoint, path, query, v, Direction::Previous))
            .transpose()?;
        if !query.operation().paginated() || !path.supports(query.operation()) {
            return Err(PaginationError::Binding);
        }
        Ok(Self { next, previous })
    }
    /// Lends the validated next link.
    #[must_use]
    pub const fn next(&self) -> Option<&PageLink<'a>> {
        self.next.as_ref()
    }
    /// Lends the validated previous link.
    #[must_use]
    pub const fn previous(&self) -> Option<&PageLink<'a>> {
        self.previous.as_ref()
    }
}

/// Legacy Cargo `more` response interpretation without synthesizing seek state.
pub fn legacy_next(current: Page, more: bool) -> Result<Option<Page>, PaginationError> {
    if !more {
        return Ok(None);
    }
    let value = current.get().checked_add(1).ok_or(PaginationError::Limit)?;
    Page::new(value)
        .map(Some)
        .map_err(|_| PaginationError::Limit)
}

/// A non-cloneable traversal counter using the neutral transactional budget.
///
/// Call once per validated response before following either meta links or
/// legacy `more`. Every actual attempt must separately use the official rate
/// gate; these decoded-response limits are not transport authorization.
#[derive(Debug)]
pub struct Traversal {
    budget: PaginationBudget,
    per_page: u32,
}
impl Traversal {
    /// Fixes traversal request/item/state ceilings and response page size.
    #[must_use]
    pub const fn new(limits: PaginationLimits, per_page: crate::query::PerPage) -> Self {
        Self {
            budget: PaginationBudget::new(limits, SnapshotPolicy::Forbidden),
            per_page: per_page.get(),
        }
    }
    /// Admits response counts before a caller follows the next continuation.
    pub fn admit(
        &mut self,
        entries: usize,
        next: Option<&PageLink<'_>>,
    ) -> Result<(), PaginationError> {
        if let Some(next) = next
            && next.state_bytes() > self.budget.limits().max_state_bytes()
        {
            return Err(PaginationError::Limit);
        }
        self.admit_count(entries, next.is_some())
    }
    /// Admits a legacy numbered response and derives the bounded next page.
    pub fn admit_legacy(
        &mut self,
        entries: usize,
        current: Page,
        more: bool,
    ) -> Result<Option<Page>, PaginationError> {
        let next = legacy_next(current, more)?;
        self.admit_count(entries, next.is_some())?;
        Ok(next)
    }
    fn admit_count(&mut self, entries: usize, more: bool) -> Result<(), PaginationError> {
        if u64::try_from(entries).map_err(|_| PaginationError::Limit)? > u64::from(self.per_page) {
            return Err(PaginationError::Limit);
        }
        self.budget
            .admit(entries, more, None)
            .map_err(|_| PaginationError::Limit)?;
        Ok(())
    }
}

#[cfg(test)]
mod execution_tests;
#[cfg(test)]
mod tests;
