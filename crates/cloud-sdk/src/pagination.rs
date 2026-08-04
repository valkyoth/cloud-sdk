//! Explicit provider-neutral pagination strategy state.

mod budget;
mod driver;
mod link;
mod local_async;
mod numbered;
mod offset;
mod opaque;

pub use budget::{
    MAX_SNAPSHOT_ID_BYTES, PaginationBudget, PaginationLimits, PaginationProgress, SnapshotId,
    SnapshotPolicy,
};
pub use driver::{PageStrategy, PagerControl, PagerDriver, PagerDriverError, PagerStep};
pub use link::{ProviderLinkBinding, ProviderLinkExecutionError, ValidatedProviderLink};
pub use numbered::{
    NumberedPageBoundary, NumberedPageMetadata, NumberedPageObservation, NumberedPagination,
    PageNumber,
};
pub use offset::{OffsetPageBoundary, OffsetPageMetadata, OffsetPageObservation, OffsetPagination};
pub use opaque::{
    CursorDigest, CursorHistory, MAX_OPAQUE_STATE_BYTES, PaginationCursor, PaginationMarker,
};

/// Pagination validation or transition error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaginationError {
    /// A required numeric bound was zero.
    ZeroLimit,
    /// An opaque state bound exceeds the SDK hard limit.
    StateLimitTooLarge,
    /// A page number was zero.
    PageZero,
    /// A page size was zero.
    PageSizeZero,
    /// Previous-page metadata was not immediately before the current page.
    InvalidPreviousPage,
    /// Next-page metadata was not immediately after the current page.
    InvalidNextPage,
    /// Last-page metadata contradicted the continuation state.
    InvalidLastPage,
    /// A response did not match the requested page or offset.
    UnexpectedPosition,
    /// A nonterminal response contained no entries.
    EmptyPageWithContinuation,
    /// Entry count or total metadata was incoherent.
    InvalidEntryCount,
    /// Provider page size changed during one traversal.
    PageSizeChanged,
    /// Stable provider traversal metadata changed.
    TraversalChanged,
    /// A continuation would exceed the request budget.
    RequestBudgetExceeded,
    /// Accepted entries would exceed the item budget.
    ItemBudgetExceeded,
    /// A required provider snapshot identifier was omitted.
    SnapshotRequired,
    /// A provider snapshot identifier was empty.
    SnapshotIdEmpty,
    /// A provider snapshot identifier exceeded its exact-byte bound.
    SnapshotIdTooLong,
    /// A provider snapshot identifier was supplied when forbidden.
    SnapshotForbidden,
    /// Snapshot presence or value changed during one traversal.
    SnapshotChanged,
    /// The traversal has no continuation.
    Complete,
    /// Opaque continuation state was omitted or empty.
    MissingState,
    /// Opaque continuation state exceeds its caller-selected limit.
    StateTooLong,
    /// Caller-owned destination storage is too small.
    OutputTooSmall,
    /// Cursor history has no remaining entry or byte budget.
    HistoryBudgetExceeded,
    /// A cursor repeated an earlier exact value.
    CursorCycle,
    /// One digest identified two distinct cursor values.
    CursorDigestCollision,
    /// One cursor value was supplied with different digests.
    CursorDigestChanged,
    /// A provider link was not valid UTF-8 or request-target syntax.
    InvalidProviderLink,
    /// A provider link changed the bound network scheme.
    ProviderLinkSchemeChanged,
    /// A provider link changed the bound authority.
    ProviderLinkAuthorityChanged,
    /// A provider link contained user information.
    ProviderLinkUserinfo,
    /// A provider link contained a fragment.
    ProviderLinkFragment,
    /// A provider link changed the bound operation path.
    ProviderLinkPathChanged,
    /// A provider link was used with another HTTP method.
    ProviderLinkMethodChanged,
    /// A provider link was used with another operation identifier.
    ProviderLinkOperationChanged,
}

impl_static_error!(PaginationError,
    Self::ZeroLimit => "pagination limits must be nonzero",
    Self::StateLimitTooLarge => "opaque pagination state limit exceeds the hard limit",
    Self::PageZero => "page number must be nonzero",
    Self::PageSizeZero => "page size must be nonzero",
    Self::InvalidPreviousPage => "previous-page metadata is invalid",
    Self::InvalidNextPage => "next-page metadata is invalid",
    Self::InvalidLastPage => "last-page metadata is invalid",
    Self::UnexpectedPosition => "response position differs from the requested position",
    Self::EmptyPageWithContinuation => "nonterminal response page is empty",
    Self::InvalidEntryCount => "response entry count is invalid",
    Self::PageSizeChanged => "response page size changed during traversal",
    Self::TraversalChanged => "pagination metadata changed during traversal",
    Self::RequestBudgetExceeded => "pagination request budget was exceeded",
    Self::ItemBudgetExceeded => "pagination item budget was exceeded",
    Self::SnapshotRequired => "provider snapshot identifier is required",
    Self::SnapshotIdEmpty => "provider snapshot identifier is empty",
    Self::SnapshotIdTooLong => "provider snapshot identifier exceeds the length limit",
    Self::SnapshotForbidden => "provider snapshot identifier is forbidden",
    Self::SnapshotChanged => "provider snapshot identifier changed during traversal",
    Self::Complete => "pagination traversal is complete",
    Self::MissingState => "opaque pagination state is missing",
    Self::StateTooLong => "opaque pagination state exceeds the selected limit",
    Self::OutputTooSmall => "pagination state output is too small",
    Self::HistoryBudgetExceeded => "cursor history budget was exceeded",
    Self::CursorCycle => "pagination cursor cycle was detected",
    Self::CursorDigestCollision => "pagination cursor digest collision was detected",
    Self::CursorDigestChanged => "pagination cursor digest changed for the same state",
    Self::InvalidProviderLink => "provider pagination link is invalid",
    Self::ProviderLinkSchemeChanged => "provider pagination link changed scheme",
    Self::ProviderLinkAuthorityChanged => "provider pagination link changed authority",
    Self::ProviderLinkUserinfo => "provider pagination link contains user information",
    Self::ProviderLinkFragment => "provider pagination link contains a fragment",
    Self::ProviderLinkPathChanged => "provider pagination link changed operation path",
    Self::ProviderLinkMethodChanged => "provider pagination link changed HTTP method",
    Self::ProviderLinkOperationChanged => "provider pagination link changed operation",
);

#[cfg(test)]
mod tests;
