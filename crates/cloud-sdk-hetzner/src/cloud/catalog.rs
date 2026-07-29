//! Read-only Cloud catalog request domains.

use cloud_sdk::transport::MAX_REQUEST_TARGET_BYTES;
use cloud_sdk::{Method, buffer};
use cloud_sdk_sanitization::sanitize_bytes;

use crate::EndpointGroup;
use crate::pagination::{Page, PerPage, Sort, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath, EndpointPathError, MAX_ENDPOINT_PATH_BYTES};

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
        let len = buffer::encode_snapshot_bounded(
            self,
            output,
            MAX_ENDPOINT_PATH_BYTES,
            CatalogRequestError::PathBufferTooSmall,
            |endpoint, encoder| {
                encoder.string(endpoint.path_prefix())?;
                encoder.u64(endpoint.id().get())
            },
        )?;
        validate_or_clear_path(output, len)?;
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
        buffer::encode_snapshot_bounded(
            self,
            output,
            MAX_REQUEST_TARGET_BYTES,
            CatalogRequestError::QueryBufferTooSmall,
            |request, encoder| {
                let mut first = true;
                if let CatalogListEndpoint::PublicImages(kind) = request.endpoint {
                    encoder.query_pair(&mut first, "type", kind.as_api_str())?;
                }
                if let Some(page) = request.page {
                    encoder.query_u64(&mut first, "page", page.get())?;
                }
                if let Some(per_page) = request.per_page {
                    encoder.query_u64(&mut first, "per_page", u64::from(per_page.get()))?;
                }
                if let Some(sort) = request.sort {
                    encoder.query_separator(&mut first)?;
                    encoder.string("sort=")?;
                    encoder.percent_encoded(sort.key().as_str())?;
                    encoder.string("%3A")?;
                    encoder.percent_encoded(sort_direction_str(sort.direction()))?;
                }
                Ok(())
            },
        )
    }
}

fn validate_or_clear_path(output: &mut [u8], len: usize) -> Result<(), CatalogRequestError> {
    if let Err(error) = validate_written_path(output, len) {
        if let Some(path) = output.get_mut(..len) {
            sanitize_bytes(path);
        }
        return Err(error);
    }
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
