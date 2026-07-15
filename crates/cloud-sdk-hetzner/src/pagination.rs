//! Pagination domains.

use cloud_sdk::pagination::PageMetadata;

/// Default page size documented by the Hetzner API.
pub const DEFAULT_PER_PAGE: u16 = 25;

/// Maximum page size documented by the Hetzner API unless specified otherwise.
pub const MAX_PER_PAGE: u16 = 50;

/// Pagination validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PaginationError {
    /// Page numbers are one-based.
    PageZero,
    /// Per-page values are one-based.
    PerPageZero,
    /// Per-page values are capped by [`MAX_PER_PAGE`].
    PerPageTooLarge,
    /// Pagination navigation metadata is contradictory.
    InvalidNavigation,
}

impl_static_error!(PaginationError,
    Self::PageZero => "page number must be nonzero",
    Self::PerPageZero => "page size must be nonzero",
    Self::PerPageTooLarge => "page size exceeds the provider limit",
    Self::InvalidNavigation => "pagination navigation metadata is invalid",
);

/// One-based page number.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Page(u64);

impl Page {
    /// Creates a one-based page number.
    pub const fn new(value: u64) -> Result<Self, PaginationError> {
        if value == 0 {
            return Err(PaginationError::PageZero);
        }
        Ok(Self(value))
    }

    /// Returns the page number.
    #[must_use]
    pub const fn get(self) -> u64 {
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

/// Validated metadata from a Hetzner paginated response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaginationMetadata {
    page: Page,
    per_page: PerPage,
    previous_page: Option<Page>,
    next_page: Option<Page>,
    last_page: Option<Page>,
    total_entries: Option<u64>,
    core: PageMetadata,
}

impl PaginationMetadata {
    /// Creates metadata using Hetzner's documented pagination fields.
    pub fn new(
        page: Page,
        per_page: PerPage,
        previous_page: Option<Page>,
        next_page: Option<Page>,
        last_page: Option<Page>,
        total_entries: Option<u64>,
    ) -> Result<Self, PaginationError> {
        let page_number = to_core_page(page)?;
        let previous = previous_page.map(to_core_page).transpose()?;
        let next = next_page.map(to_core_page).transpose()?;
        let last = last_page.map(to_core_page).transpose()?;
        let core = PageMetadata::new(
            page_number,
            u64::from(per_page.get()),
            previous,
            next,
            last,
            total_entries,
        )
        .map_err(|_| PaginationError::InvalidNavigation)?;
        Ok(Self {
            page,
            per_page,
            previous_page,
            next_page,
            last_page,
            total_entries,
            core,
        })
    }

    /// Returns the current page.
    #[must_use]
    pub const fn page(self) -> Page {
        self.page
    }

    /// Returns the requested maximum entries per page.
    #[must_use]
    pub const fn per_page(self) -> PerPage {
        self.per_page
    }

    /// Returns the previous page when present.
    #[must_use]
    pub const fn previous_page(self) -> Option<Page> {
        self.previous_page
    }

    /// Returns the next page when present.
    #[must_use]
    pub const fn next_page(self) -> Option<Page> {
        self.next_page
    }

    /// Returns the final page when known.
    #[must_use]
    pub const fn last_page(self) -> Option<Page> {
        self.last_page
    }

    /// Returns the total matching entries when known.
    #[must_use]
    pub const fn total_entries(self) -> Option<u64> {
        self.total_entries
    }

    /// Returns provider-neutral metadata for the core pagination cursor.
    #[must_use]
    pub const fn as_core(self) -> PageMetadata {
        self.core
    }
}

fn to_core_page(page: Page) -> Result<cloud_sdk::pagination::PageNumber, PaginationError> {
    cloud_sdk::pagination::PageNumber::new(page.get())
        .map_err(|_| PaginationError::InvalidNavigation)
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

impl_static_error!(SortError,
    Self::Empty => "sort key is empty",
    Self::InvalidByte => "sort key contains an invalid byte",
);

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
    use super::{
        Page, PaginationError, PaginationMetadata, PerPage, SortDirection, SortError, SortKey,
    };

    #[test]
    fn validates_pagination_bounds() {
        assert_eq!(Page::new(0), Err(PaginationError::PageZero));
        assert_eq!(PerPage::new(0), Err(PaginationError::PerPageZero));
        assert_eq!(PerPage::new(51), Err(PaginationError::PerPageTooLarge));
        assert_eq!(PerPage::new(50).map(PerPage::get), Ok(50));
        assert_eq!(PerPage::default().get(), 25);
    }

    #[test]
    fn validates_navigation_and_converts_to_core_metadata() {
        let metadata = PaginationMetadata::new(
            Page::new(2).unwrap_or_else(|_| unreachable!()),
            PerPage::new(25).unwrap_or_else(|_| unreachable!()),
            Page::new(1).ok(),
            Page::new(3).ok(),
            Page::new(4).ok(),
            Some(100),
        );
        assert!(metadata.is_ok());
        let Ok(metadata) = metadata else { return };
        assert_eq!(metadata.as_core().page().get(), 2);
        assert_eq!(
            metadata.as_core().next_page().map(|page| page.get()),
            Some(3)
        );
        assert_eq!(metadata.total_entries(), Some(100));

        assert_eq!(
            PaginationMetadata::new(
                Page::new(2).unwrap_or_else(|_| unreachable!()),
                PerPage::default(),
                None,
                Page::new(2).ok(),
                None,
                None,
            ),
            Err(PaginationError::InvalidNavigation)
        );
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
