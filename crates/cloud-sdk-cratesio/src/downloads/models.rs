use crate::{discovery::DiscoveryValue, query::Page};

/// Complete checked usage data. Values and field names remain protected.
#[derive(Debug)]
pub struct DownloadCounts(pub(super) DiscoveryValue);
impl DownloadCounts {
    /// Enumerate time buckets, aggregate counts and optionally included versions.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.0
    }
}
/// Explicit numbered continuation, never an automatic crawl.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReverseContinuation {
    /// The observed total has been reached; totals are not snapshot isolation.
    End,
    /// Repeat the same request with this page and unchanged page size.
    Next(Page),
    /// More matches exist beyond the SDK's numbered-page ceiling.
    LimitReached,
}
/// Reverse dependency records correlated with included dependent versions.
#[derive(Debug)]
pub struct ReverseDependencies {
    pub(super) fields: DiscoveryValue,
    /// Explicit continuation state.
    pub next: ReverseContinuation,
}
impl ReverseDependencies {
    /// Complete bounded dependency, version and total metadata.
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.fields
    }
}
/// Inert archive location; inspection grants no fetch or credential authority.
#[derive(Debug)]
pub struct DownloadLocation(pub(super) DiscoveryValue);
impl DownloadLocation {
    /// Inspect untrusted location text without fetching it.
    pub fn with_url<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, super::DownloadError> {
        self.0.required("url")?.with_text(inspect)
    }
}
/// Success bound to the selected download or usage operation.
#[derive(Debug)]
pub enum DownloadResponse {
    /// JSON archive location; no redirect is followed.
    Location(DownloadLocation),
    /// Crate or version time buckets.
    Counts(DownloadCounts),
    /// Bounded numbered reverse dependency page.
    ReverseDependencies(ReverseDependencies),
}
