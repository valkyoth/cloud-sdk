//! Volume request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::actions::ActionId;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

use super::shared::{
    CloudLabels, CloudName, CloudQueryWriter, CloudRequestError, CloudResourceId, CloudText,
    write_id_path, write_static_path,
};

/// Minimum Volume size in GB admitted by the source-locked API.
pub const MIN_VOLUME_SIZE_GB: u32 = 10;
/// Maximum Volume size in GB admitted by the source-locked API.
pub const MAX_VOLUME_SIZE_GB: u32 = 10_240;

/// Volume identifier.
pub type VolumeId = CloudResourceId;
/// Volume server identifier.
pub type VolumeServerId = CloudResourceId;
/// Volume request name.
pub type VolumeName<'a> = CloudName<'a>;
/// Volume location name.
pub type VolumeLocation<'a> = CloudName<'a>;
/// Volume filesystem format marker.
pub type VolumeFormat<'a> = CloudText<'a>;
/// Volume request labels.
pub type VolumeLabels<'a> = CloudLabels<'a>;
/// Volume request error.
pub type VolumeRequestError = CloudRequestError;

/// Volume endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] =
    &[EndpointGroup::Volumes, EndpointGroup::VolumeActions];

/// Volume size in GB.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeSizeGb(u32);

impl VolumeSizeGb {
    /// Creates a bounded Volume size.
    pub const fn new(value: u32) -> Option<Self> {
        if value < MIN_VOLUME_SIZE_GB || value > MAX_VOLUME_SIZE_GB {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw size in GB.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// Volume status filter admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeStatus {
    /// Volume is available.
    Available,
    /// Volume is being created.
    Creating,
}

impl VolumeStatus {
    const fn as_api_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Creating => "creating",
        }
    }
}

/// Volume sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Explicit Volume create placement.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeCreatePlacement<'a> {
    /// Create and attach to this server.
    Server {
        /// Server identifier.
        server: VolumeServerId,
        /// Whether the API should auto-mount after attach.
        automount: bool,
    },
    /// Create unattached in this location.
    Location(VolumeLocation<'a>),
}

/// Volume CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeEndpoint {
    /// `GET /volumes`.
    List,
    /// `POST /volumes`.
    Create,
    /// `GET /volumes/{id}`.
    Get(VolumeId),
    /// `PUT /volumes/{id}`.
    Update(VolumeId),
    /// `DELETE /volumes/{id}`.
    Delete(VolumeId),
}

impl VolumeEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) => Method::Get,
            Self::Create => Method::Post,
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
        EndpointGroup::Volumes
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, VolumeRequestError> {
        match self {
            Self::List | Self::Create => write_static_path(output, "/volumes"),
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/volumes/", id, "")
            }
        }
    }
}

/// Volume list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeListRequest<'a> {
    status: Option<VolumeStatus>,
    name: Option<VolumeName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(VolumeSortField, SortDirection)>,
}

impl<'a> VolumeListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            status: None,
            name: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets status filtering.
    #[must_use]
    pub const fn with_status(mut self, status: VolumeStatus) -> Self {
        self.status = Some(status);
        self
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: VolumeName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
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
    pub const fn with_sort(mut self, field: VolumeSortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, VolumeRequestError> {
        let mut writer = CloudQueryWriter::new(output);
        if let Some(selector) = self.label_selector {
            writer.pair("label_selector", selector.as_str())?;
        }
        if let Some(name) = self.name {
            writer.pair("name", name.as_str())?;
        }
        if let Some(page) = self.page {
            writer.u64_pair("page", page.get())?;
        }
        if let Some(per_page) = self.per_page {
            writer.u64_pair("per_page", u64::from(per_page.get()))?;
        }
        if let Some((field, direction)) = self.sort {
            writer.pair("sort", volume_sort_value(field, direction))?;
        }
        if let Some(status) = self.status {
            writer.pair("status", status.as_api_str())?;
        }
        Ok(writer.len())
    }
}

impl Default for VolumeListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Volume create request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeCreateRequest<'a> {
    size: VolumeSizeGb,
    name: VolumeName<'a>,
    placement: VolumeCreatePlacement<'a>,
    format: Option<VolumeFormat<'a>>,
    labels: Option<VolumeLabels<'a>>,
}

impl<'a> VolumeCreateRequest<'a> {
    /// Creates a validated create request with explicit server or location placement.
    pub fn try_new(
        size: Option<VolumeSizeGb>,
        name: Option<VolumeName<'a>>,
        placement: Option<VolumeCreatePlacement<'a>>,
    ) -> Result<Self, VolumeRequestError> {
        Ok(Self {
            size: size.ok_or(VolumeRequestError::MissingRequiredField)?,
            name: name.ok_or(VolumeRequestError::MissingRequiredField)?,
            placement: placement.ok_or(VolumeRequestError::MissingRequiredField)?,
            format: None,
            labels: None,
        })
    }

    /// Sets filesystem formatting at creation.
    #[must_use]
    pub const fn with_format(mut self, format: VolumeFormat<'a>) -> Self {
        self.format = Some(format);
        self
    }

    /// Sets labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: VolumeLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the requested size.
    #[must_use]
    pub const fn size(self) -> VolumeSizeGb {
        self.size
    }

    /// Returns the explicit placement.
    #[must_use]
    pub const fn placement(self) -> VolumeCreatePlacement<'a> {
        self.placement
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> VolumeEndpoint {
        VolumeEndpoint::Create
    }
}

/// Volume update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeUpdateRequest<'a> {
    id: VolumeId,
    name: Option<VolumeName<'a>>,
    labels: Option<VolumeLabels<'a>>,
}

impl<'a> VolumeUpdateRequest<'a> {
    /// Creates an update request.
    #[must_use]
    pub const fn new(id: VolumeId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: VolumeName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: VolumeLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> VolumeEndpoint {
        VolumeEndpoint::Update(self.id)
    }
}

/// Volume action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VolumeActionEndpoint {
    /// `GET /volumes/actions`.
    ListAll,
    /// `GET /volumes/actions/{id}`.
    Get(ActionId),
    /// `GET /volumes/{id}/actions`.
    ListForVolume(VolumeId),
    /// `POST /volumes/{id}/actions/attach`.
    Attach(VolumeId),
    /// `POST /volumes/{id}/actions/change_protection`.
    ChangeProtection(VolumeId),
    /// `POST /volumes/{id}/actions/detach`.
    Detach(VolumeId),
    /// `POST /volumes/{id}/actions/resize`.
    Resize(VolumeId),
}

impl VolumeActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::ListAll | Self::Get(_) | Self::ListForVolume(_) => Method::Get,
            Self::Attach(_) | Self::ChangeProtection(_) | Self::Detach(_) | Self::Resize(_) => {
                Method::Post
            }
        }
    }

    /// Returns the endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::VolumeActions
    }

    /// Returns the base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, VolumeRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/volumes/actions"),
            Self::Get(id) => {
                let id = VolumeId::new(id.get()).ok_or(VolumeRequestError::InvalidType)?;
                write_id_path(output, "/volumes/actions/", id, "")
            }
            Self::ListForVolume(id) => write_id_path(output, "/volumes/", id, "/actions"),
            Self::Attach(id) => write_id_path(output, "/volumes/", id, "/actions/attach"),
            Self::ChangeProtection(id) => {
                write_id_path(output, "/volumes/", id, "/actions/change_protection")
            }
            Self::Detach(id) => write_id_path(output, "/volumes/", id, "/actions/detach"),
            Self::Resize(id) => write_id_path(output, "/volumes/", id, "/actions/resize"),
        }
    }
}

/// Volume attach action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeAttachRequest {
    server: VolumeServerId,
    automount: bool,
}

impl VolumeAttachRequest {
    /// Creates an attach request.
    pub fn try_new(
        server: Option<VolumeServerId>,
        automount: bool,
    ) -> Result<Self, VolumeRequestError> {
        Ok(Self {
            server: server.ok_or(VolumeRequestError::MissingRequiredField)?,
            automount,
        })
    }

    /// Returns the server ID.
    #[must_use]
    pub const fn server(self) -> VolumeServerId {
        self.server
    }

    /// Returns whether auto-mount is requested.
    #[must_use]
    pub const fn automount(self) -> bool {
        self.automount
    }
}

/// Volume resize action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeResizeRequest {
    size: VolumeSizeGb,
}

impl VolumeResizeRequest {
    /// Creates a resize request.
    pub fn try_new(size: Option<VolumeSizeGb>) -> Result<Self, VolumeRequestError> {
        Ok(Self {
            size: size.ok_or(VolumeRequestError::MissingRequiredField)?,
        })
    }

    /// Returns the target size.
    #[must_use]
    pub const fn size(self) -> VolumeSizeGb {
        self.size
    }
}

/// Volume protection action request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VolumeProtectionRequest {
    delete: bool,
}

impl VolumeProtectionRequest {
    /// Creates a protection request.
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

const fn volume_sort_value(field: VolumeSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (VolumeSortField::Id, SortDirection::Asc) => "id:asc",
        (VolumeSortField::Id, SortDirection::Desc) => "id:desc",
        (VolumeSortField::Name, SortDirection::Asc) => "name:asc",
        (VolumeSortField::Name, SortDirection::Desc) => "name:desc",
        (VolumeSortField::Created, SortDirection::Asc) => "created:asc",
        (VolumeSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
