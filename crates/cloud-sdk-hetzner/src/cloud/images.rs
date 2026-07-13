//! Image and ISO endpoint domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};

use super::shared::{
    CloudLabels, CloudName, CloudQueryWriter, CloudRequestError, CloudResourceId, CloudText,
    static_path, write_id_path, write_static_path,
};

/// Image and ISO endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Images,
    EndpointGroup::ImageActions,
    EndpointGroup::Isos,
];

/// Image identifier.
pub type ImageId = CloudResourceId;
/// Image request error.
pub type ImageRequestError = CloudRequestError;
/// Image description value.
pub type ImageDescription<'a> = CloudText<'a>;
/// Image request labels.
pub type ImageLabels<'a> = CloudLabels<'a>;
/// Image resource name.
pub type ImageName<'a> = CloudName<'a>;

/// Image type filter admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageTypeFilter {
    /// Provider-maintained system image.
    System,
    /// Provider-maintained app image.
    App,
    /// User snapshot image.
    Snapshot,
    /// Backup image.
    Backup,
}

impl ImageTypeFilter {
    const fn as_api_str(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::App => "app",
            Self::Snapshot => "snapshot",
            Self::Backup => "backup",
        }
    }
}

/// Image sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Image CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageEndpoint {
    /// `GET /images`.
    List,
    /// `GET /images/{id}`.
    Get(ImageId),
    /// `PUT /images/{id}`.
    Update(ImageId),
    /// `DELETE /images/{id}`.
    Delete(ImageId),
}

impl ImageEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) => Method::Get,
            Self::Update(_) => Method::Put,
            Self::Delete(_) => Method::Delete,
        }
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::Images
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, ImageRequestError>> {
        match self {
            Self::List => Some(static_path("/images")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ImageRequestError> {
        match self {
            Self::List => write_static_path(output, "/images"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/images/", id, "")
            }
        }
    }
}

/// Image list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageListRequest {
    image_type: Option<ImageTypeFilter>,
    bound_to: Option<ImageId>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(ImageSortField, SortDirection)>,
}

impl ImageListRequest {
    /// Creates an empty image list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            image_type: None,
            bound_to: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets image type filtering.
    #[must_use]
    pub const fn with_type(mut self, image_type: ImageTypeFilter) -> Self {
        self.image_type = Some(image_type);
        self
    }

    /// Sets server binding filtering.
    #[must_use]
    pub const fn with_bound_to(mut self, id: ImageId) -> Self {
        self.bound_to = Some(id);
        self
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per_page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Sets source-locked sorting.
    #[must_use]
    pub const fn with_sort(mut self, field: ImageSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, ImageRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(id) = self.bound_to {
            writer.u64_pair("bound_to", id.get())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", page.get())?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", image_sort_value(field, direction))?;
        }
        if let Some(image_type) = self.image_type {
            writer.pair("type", image_type.as_api_str())?;
        }
        Ok(writer.len())
    }
}

impl Default for ImageListRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Image update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageUpdateRequest<'a> {
    id: ImageId,
    description: Option<ImageDescription<'a>>,
    labels: Option<ImageLabels<'a>>,
}

impl<'a> ImageUpdateRequest<'a> {
    /// Creates an image update request.
    #[must_use]
    pub const fn new(id: ImageId) -> Self {
        Self {
            id,
            description: None,
            labels: None,
        }
    }

    /// Sets replacement description.
    #[must_use]
    pub const fn with_description(mut self, description: ImageDescription<'a>) -> Self {
        self.description = Some(description);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: ImageLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> ImageEndpoint {
        ImageEndpoint::Update(self.id)
    }
}

/// Image action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageActionEndpoint {
    /// `GET /images/actions`.
    ListAll,
    /// `GET /images/actions/{id}`.
    Get(ActionId),
    /// `GET /images/{id}/actions`.
    ListForImage(ImageId),
    /// `POST /images/{id}/actions/change_protection`.
    ChangeProtection(ImageId),
}

impl ImageActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForImage(_) => Method::Get,
            Self::ChangeProtection(_) => Method::Post,
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::ImageActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, ImageRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/images/actions"),
            Self::Get(id) => {
                let id = ImageId::new(id.get()).ok_or(ImageRequestError::InvalidType)?;
                write_id_path(output, "/images/actions/", id, "")
            }
            Self::ListForImage(id) => write_id_path(output, "/images/", id, "/actions"),
            Self::ChangeProtection(id) => {
                write_id_path(output, "/images/", id, "/actions/change_protection")
            }
        }
    }
}

/// Image protection action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ImageProtectionRequest {
    delete: bool,
}

impl ImageProtectionRequest {
    /// Creates an image protection request.
    #[must_use]
    pub const fn new(delete: bool) -> Self {
        Self { delete }
    }

    /// Returns the delete protection value.
    #[must_use]
    pub const fn delete(self) -> bool {
        self.delete
    }
}

const fn image_sort_value(field: ImageSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (ImageSortField::Id, SortDirection::Asc) => "id:asc",
        (ImageSortField::Id, SortDirection::Desc) => "id:desc",
        (ImageSortField::Name, SortDirection::Asc) => "name:asc",
        (ImageSortField::Name, SortDirection::Desc) => "name:desc",
        (ImageSortField::Created, SortDirection::Asc) => "created:asc",
        (ImageSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
