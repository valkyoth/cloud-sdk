//! Source-scoped query values and transactional caller-buffer encoding.

use core::fmt;
mod encode;
mod parameters;
mod path;
mod values;
pub use parameters::{Parameter, Query};
pub use path::{ApiPath, FixedSegment, PathSegment};
pub use values::{Include, IncludeSet, Page, PerPage, QueryOperation, SearchQuery, Seek, Sort};

/// SDK ceiling on decoded search text and opaque seek state.
pub const MAX_QUERY_VALUE_BYTES: usize = 1024;
/// Hard bound on input parameter groups and each array's entries.
pub const MAX_QUERY_PARAMETERS: usize = 16;
/// SDK aggregate encoded query/target ceiling, below the neutral target cap.
pub const MAX_TARGET_BYTES: usize = 4096;
/// Source-locked maximum number of entries per response page.
pub const MAX_PER_PAGE: u32 = 100;
/// Conservative SDK numbered-page ceiling; prefer seek and bulk alternatives.
pub const MAX_PAGE: u32 = 10;

/// Payload-free request component error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryError {
    /// Invalid component or unknown enum value.
    Syntax,
    /// SDK or source-owned resource bound exceeded.
    Limit,
    /// Duplicate parameter or repeated array element.
    Duplicate,
    /// Parameter, include or sort is not valid for the selected operation.
    Operation,
    /// Mutually exclusive inputs or inputs silently shadowed upstream.
    Conflict,
    /// Output cannot contain the complete encoded request.
    Output,
}
impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Syntax => "invalid crates.io request component",
            Self::Limit => "crates.io request component exceeds its bound",
            Self::Duplicate => "duplicate crates.io request component",
            Self::Operation => "crates.io component is not supported by this operation",
            Self::Conflict => "conflicting crates.io request components",
            Self::Output => "crates.io request output buffer is too small",
        })
    }
}
impl core::error::Error for QueryError {}

#[cfg(test)]
mod contract_tests;
#[cfg(test)]
mod tests;
