//! Read-only Cloud catalog request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::pagination::{Page, PerPage, Sort, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath, EndpointPathError};

/// Error returned while building catalog request components.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogRequestError {
    /// Endpoint paths failed validation.
    InvalidPath(EndpointPathError),
    /// The endpoint does not support pagination.
    UnsupportedPagination,
    /// The endpoint does not support sorting.
    UnsupportedSorting,
    /// Caller-provided path buffer is too small.
    PathBufferTooSmall,
    /// Caller-provided query buffer is too small.
    QueryBufferTooSmall,
    /// Decimal conversion failed.
    NumberEncodingFailed,
    /// Path bytes failed UTF-8 conversion after construction.
    PathEncodingFailed,
}

impl_static_error!(CatalogRequestError,
    Self::InvalidPath(_) => "catalog endpoint path is invalid",
    Self::UnsupportedPagination => "catalog endpoint does not support pagination",
    Self::UnsupportedSorting => "catalog endpoint does not support sorting",
    Self::PathBufferTooSmall => "catalog path buffer is too small",
    Self::QueryBufferTooSmall => "catalog query buffer is too small",
    Self::NumberEncodingFailed => "catalog number encoding failed",
    Self::PathEncodingFailed => "catalog path encoding failed",
);

/// Nonzero identifier for read-only catalog resources.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CatalogId(u64);

impl CatalogId {
    /// Creates a nonzero catalog identifier.
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw identifier.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Public Hetzner image kind admitted by the v0.4 catalog API.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PublicImageKind {
    /// Provider-maintained operating system image.
    System,
    /// Provider-maintained application image.
    App,
}

impl PublicImageKind {
    /// Returns the Cloud API image type query value.
    #[must_use]
    pub const fn as_api_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::App => "app",
        }
    }
}

/// Read-only catalog list endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogListEndpoint {
    /// `GET /locations`.
    Locations,
    /// `GET /server_types`.
    ServerTypes,
    /// `GET /load_balancer_types`.
    LoadBalancerTypes,
    /// `GET /isos`.
    Isos,
    /// `GET /images` scoped to public provider image types.
    PublicImages(PublicImageKind),
}

impl CatalogListEndpoint {
    /// Returns the endpoint group from the source-locked API matrix.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        match self {
            Self::Locations => EndpointGroup::Locations,
            Self::ServerTypes => EndpointGroup::ServerTypes,
            Self::LoadBalancerTypes => EndpointGroup::LoadBalancerTypes,
            Self::Isos => EndpointGroup::Isos,
            Self::PublicImages(_) => EndpointGroup::Images,
        }
    }

    /// Returns the source-locked list path.
    #[must_use]
    pub const fn path_str(self) -> &'static str {
        match self {
            Self::Locations => "/locations",
            Self::ServerTypes => "/server_types",
            Self::LoadBalancerTypes => "/load_balancer_types",
            Self::Isos => "/isos",
            Self::PublicImages(_) => "/images",
        }
    }

    /// Returns a validated endpoint path.
    pub fn path(self) -> Result<EndpointPath<'static>, CatalogRequestError> {
        EndpointPath::new(self.path_str()).map_err(CatalogRequestError::InvalidPath)
    }

    /// Returns true when the list endpoint accepts page and per_page.
    #[must_use]
    pub const fn supports_pagination(self) -> bool {
        true
    }

    /// Returns true when the list endpoint accepts sort.
    #[must_use]
    pub const fn supports_sorting(self) -> bool {
        matches!(self, Self::Locations | Self::PublicImages(_))
    }
}

/// Read-only catalog get endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogGetEndpoint {
    /// `GET /locations/{id}`.
    Location(CatalogId),
    /// `GET /server_types/{id}`.
    ServerType(CatalogId),
    /// `GET /load_balancer_types/{id}`.
    LoadBalancerType(CatalogId),
    /// `GET /isos/{id}`.
    Iso(CatalogId),
    /// `GET /images/{id}` for an image identifier.
    ///
    /// The `PublicImage` variant name reflects the intended catalog use case:
    /// looking up a provider-maintained image by ID. It does not and cannot
    /// verify that `id` refers to a public image rather than a private snapshot
    /// or backup. That scoping is enforced server-side by the Hetzner API based
    /// on account ownership, not by this request builder.
    PublicImage(CatalogId),
}

impl CatalogGetEndpoint {
    /// Returns the endpoint group from the source-locked API matrix.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        match self {
            Self::Location(_) => EndpointGroup::Locations,
            Self::ServerType(_) => EndpointGroup::ServerTypes,
            Self::LoadBalancerType(_) => EndpointGroup::LoadBalancerTypes,
            Self::Iso(_) => EndpointGroup::Isos,
            Self::PublicImage(_) => EndpointGroup::Images,
        }
    }

    /// Writes the source-locked get path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, CatalogRequestError> {
        let mut len = 0;
        write_path_str(output, &mut len, self.path_prefix())?;
        write_path_u64(output, &mut len, self.id().get())?;
        validate_written_path(output, len)?;
        Ok(len)
    }

    fn path_prefix(self) -> &'static str {
        match self {
            Self::Location(_) => "/locations/",
            Self::ServerType(_) => "/server_types/",
            Self::LoadBalancerType(_) => "/load_balancer_types/",
            Self::Iso(_) => "/isos/",
            Self::PublicImage(_) => "/images/",
        }
    }

    fn id(self) -> CatalogId {
        match self {
            Self::Location(id)
            | Self::ServerType(id)
            | Self::LoadBalancerType(id)
            | Self::Iso(id)
            | Self::PublicImage(id) => id,
        }
    }
}

/// Read-only catalog singleton endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CatalogSingletonEndpoint {
    /// `GET /pricing`.
    Pricing,
}

impl CatalogSingletonEndpoint {
    /// Returns the endpoint group from the source-locked API matrix.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        match self {
            Self::Pricing => EndpointGroup::Pricing,
        }
    }

    /// Returns a validated endpoint path.
    pub fn path(self) -> Result<EndpointPath<'static>, CatalogRequestError> {
        match self {
            Self::Pricing => {
                EndpointPath::new("/pricing").map_err(CatalogRequestError::InvalidPath)
            }
        }
    }
}

/// Read-only catalog list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CatalogListRequest<'a> {
    endpoint: CatalogListEndpoint,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<Sort<'a>>,
}

impl<'a> CatalogListRequest<'a> {
    /// Creates a list request for the endpoint.
    #[must_use]
    pub const fn new(endpoint: CatalogListEndpoint) -> Self {
        Self {
            endpoint,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Get
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the list endpoint.
    #[must_use]
    pub const fn endpoint(self) -> CatalogListEndpoint {
        self.endpoint
    }

    /// Sets the page value.
    pub fn with_page(mut self, page: Page) -> Result<Self, CatalogRequestError> {
        if !self.endpoint.supports_pagination() {
            return Err(CatalogRequestError::UnsupportedPagination);
        }
        self.page = Some(page);
        Ok(self)
    }

    /// Sets the per_page value.
    pub fn with_per_page(mut self, per_page: PerPage) -> Result<Self, CatalogRequestError> {
        if !self.endpoint.supports_pagination() {
            return Err(CatalogRequestError::UnsupportedPagination);
        }
        self.per_page = Some(per_page);
        Ok(self)
    }

    /// Sets the sort value.
    pub fn with_sort(mut self, sort: Sort<'a>) -> Result<Self, CatalogRequestError> {
        if !self.endpoint.supports_sorting() {
            return Err(CatalogRequestError::UnsupportedSorting);
        }
        self.sort = Some(sort);
        Ok(self)
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, CatalogRequestError> {
        let mut len = 0;
        let mut first = true;
        if let CatalogListEndpoint::PublicImages(kind) = self.endpoint {
            write_pair(output, &mut len, &mut first, "type", kind.as_api_str())?;
        }
        if let Some(page) = self.page {
            write_pair_u64(output, &mut len, &mut first, "page", page.get())?;
        }
        if let Some(per_page) = self.per_page {
            write_pair_u64(
                output,
                &mut len,
                &mut first,
                "per_page",
                u64::from(per_page.get()),
            )?;
        }
        if let Some(sort) = self.sort {
            write_sort_pair(output, &mut len, &mut first, sort)?;
        }
        Ok(len)
    }
}

fn write_pair(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: &str,
) -> Result<(), CatalogRequestError> {
    write_separator(output, len, first)?;
    write_str(output, len, key)?;
    write_query_byte(output, len, b'=')?;
    write_str(output, len, value)
}

fn write_pair_u64(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: u64,
) -> Result<(), CatalogRequestError> {
    write_separator(output, len, first)?;
    write_str(output, len, key)?;
    write_query_byte(output, len, b'=')?;
    write_u64(output, len, value)
}

fn write_sort_pair(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    sort: Sort<'_>,
) -> Result<(), CatalogRequestError> {
    write_separator(output, len, first)?;
    write_str(output, len, "sort=")?;
    write_str(output, len, sort.key().as_str())?;
    write_str(output, len, "%3A")?;
    write_str(output, len, sort_direction_str(sort.direction()))
}

fn write_separator(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
) -> Result<(), CatalogRequestError> {
    if *first {
        *first = false;
        return Ok(());
    }
    write_query_byte(output, len, b'&')
}

fn write_str(output: &mut [u8], len: &mut usize, value: &str) -> Result<(), CatalogRequestError> {
    for byte in value.bytes() {
        write_query_byte(output, len, byte)?;
    }
    Ok(())
}

fn write_u64(output: &mut [u8], len: &mut usize, value: u64) -> Result<(), CatalogRequestError> {
    write_u64_with_error(output, len, value, CatalogRequestError::QueryBufferTooSmall)
}

fn write_path_str(
    output: &mut [u8],
    len: &mut usize,
    value: &str,
) -> Result<(), CatalogRequestError> {
    for byte in value.bytes() {
        write_path_byte(output, len, byte)?;
    }
    Ok(())
}

fn write_path_u64(
    output: &mut [u8],
    len: &mut usize,
    value: u64,
) -> Result<(), CatalogRequestError> {
    write_u64_with_error(output, len, value, CatalogRequestError::PathBufferTooSmall)
}

fn write_u64_with_error(
    output: &mut [u8],
    len: &mut usize,
    mut value: u64,
    buffer_error: CatalogRequestError,
) -> Result<(), CatalogRequestError> {
    if value == 0 {
        return write_byte(output, len, b'0', buffer_error);
    }

    let mut digits = [0u8; 20];
    let mut cursor = digits.len();
    while value != 0 {
        cursor = match cursor.checked_sub(1) {
            Some(next) => next,
            None => return Err(CatalogRequestError::NumberEncodingFailed),
        };
        let remainder = value % 10;
        let digit =
            u8::try_from(remainder).map_err(|_| CatalogRequestError::NumberEncodingFailed)?;
        let byte = b'0'
            .checked_add(digit)
            .ok_or(CatalogRequestError::NumberEncodingFailed)?;
        let slot = digits
            .get_mut(cursor)
            .ok_or(CatalogRequestError::NumberEncodingFailed)?;
        *slot = byte;
        value /= 10;
    }

    let encoded = digits
        .get(cursor..)
        .ok_or(CatalogRequestError::NumberEncodingFailed)?;
    for byte in encoded {
        write_byte(output, len, *byte, buffer_error)?;
    }
    Ok(())
}

fn write_path_byte(
    output: &mut [u8],
    len: &mut usize,
    byte: u8,
) -> Result<(), CatalogRequestError> {
    write_byte(output, len, byte, CatalogRequestError::PathBufferTooSmall)
}

fn write_query_byte(
    output: &mut [u8],
    len: &mut usize,
    byte: u8,
) -> Result<(), CatalogRequestError> {
    write_byte(output, len, byte, CatalogRequestError::QueryBufferTooSmall)
}

fn write_byte(
    output: &mut [u8],
    len: &mut usize,
    byte: u8,
    buffer_error: CatalogRequestError,
) -> Result<(), CatalogRequestError> {
    let slot = output.get_mut(*len).ok_or(buffer_error)?;
    *slot = byte;
    *len = len.checked_add(1).ok_or(buffer_error)?;
    Ok(())
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), CatalogRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(CatalogRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| CatalogRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(CatalogRequestError::InvalidPath)?;
    Ok(())
}

const fn sort_direction_str(direction: SortDirection) -> &'static str {
    match direction {
        SortDirection::Asc => "asc",
        SortDirection::Desc => "desc",
    }
}

#[cfg(test)]
mod tests;
