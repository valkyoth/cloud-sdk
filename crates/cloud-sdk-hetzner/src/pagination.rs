//! Pagination domains.

/// Default page size used when callers do not override pagination.
pub const DEFAULT_PER_PAGE: u16 = 50;

/// Maximum page size admitted by the SDK policy until source-locked.
pub const MAX_PER_PAGE: u16 = 100;
