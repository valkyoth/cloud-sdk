//! Pagination domains.

/// Default page size used when callers do not override pagination.
pub const DEFAULT_PER_PAGE: u16 = 50;

/// Maximum page size admitted by the SDK policy until source-locked.
pub const MAX_PER_PAGE: u16 = 100;

/// Pagination validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaginationError {
    /// Page numbers are one-based.
    PageZero,
    /// Per-page values are one-based.
    PerPageZero,
    /// Per-page values are capped by [`MAX_PER_PAGE`].
    PerPageTooLarge,
}

/// One-based page number.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Page(u32);

impl Page {
    /// Creates a one-based page number.
    pub const fn new(value: u32) -> Result<Self, PaginationError> {
        if value == 0 {
            return Err(PaginationError::PageZero);
        }
        Ok(Self(value))
    }

    /// Returns the page number.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Per-page result count.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PerPage(u16);

impl PerPage {
    /// Creates a validated per-page value.
    pub const fn new(value: u16) -> Result<Self, PaginationError> {
        if value == 0 {
            return Err(PaginationError::PerPageZero);
        }
        if value > MAX_PER_PAGE {
            return Err(PaginationError::PerPageTooLarge);
        }
        Ok(Self(value))
    }

    /// Returns the per-page value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

impl Default for PerPage {
    fn default() -> Self {
        Self(DEFAULT_PER_PAGE)
    }
}

/// Sort direction.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SortDirection {
    /// Ascending order.
    Asc,
    /// Descending order.
    Desc,
}

/// Sort key validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SortError {
    /// Sort keys must not be empty.
    Empty,
    /// Sort keys must contain only simple field-name bytes.
    InvalidByte,
}

/// Borrowed sort key.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SortKey<'a> {
    value: &'a str,
}

impl<'a> SortKey<'a> {
    /// Creates a validated sort key.
    pub fn new(value: &'a str) -> Result<Self, SortError> {
        if value.is_empty() {
            return Err(SortError::Empty);
        }
        for byte in value.bytes() {
            if !matches!(byte, b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'_' | b'.') {
                return Err(SortError::InvalidByte);
            }
        }
        Ok(Self { value })
    }

    /// Returns the sort key.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

/// Sort parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Sort<'a> {
    key: SortKey<'a>,
    direction: SortDirection,
}

impl<'a> Sort<'a> {
    /// Creates a sort parameter.
    #[must_use]
    pub const fn new(key: SortKey<'a>, direction: SortDirection) -> Self {
        Self { key, direction }
    }

    /// Returns the key.
    #[must_use]
    pub const fn key(self) -> SortKey<'a> {
        self.key
    }

    /// Returns the direction.
    #[must_use]
    pub const fn direction(self) -> SortDirection {
        self.direction
    }
}

#[cfg(test)]
mod tests {
    use super::{Page, PaginationError, PerPage, SortDirection, SortError, SortKey};

    #[test]
    fn validates_pagination_bounds() {
        assert_eq!(Page::new(0), Err(PaginationError::PageZero));
        assert_eq!(PerPage::new(0), Err(PaginationError::PerPageZero));
        assert_eq!(PerPage::new(101), Err(PaginationError::PerPageTooLarge));
        assert_eq!(PerPage::new(100).map(PerPage::get), Ok(100));
    }

    #[test]
    fn validates_sort_keys() {
        assert_eq!(SortKey::new(""), Err(SortError::Empty));
        assert_eq!(SortKey::new("bad-key"), Err(SortError::InvalidByte));
        assert_eq!(
            SortKey::new("created.desc").map(SortKey::as_str),
            Ok("created.desc")
        );
        assert_eq!(SortDirection::Asc, SortDirection::Asc);
    }
}
