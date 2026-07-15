//! SSH key endpoint request domains.

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};
use crate::security::shared::{
    MAX_SSH_FINGERPRINT_BYTES, static_path, write_id_path, write_query_pair, write_query_u64,
};

/// SSH key endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[EndpointGroup::SshKeys];

/// SSH key identifier.
pub use crate::security::shared::SecurityId as SshKeyId;

/// SSH key resource name.
pub use crate::security::shared::SecurityName as SshKeyName;

/// SSH key request labels.
pub use crate::security::shared::SecurityLabels;

/// SSH key request error.
pub use crate::security::shared::SecurityRequestError;

/// SSH public key request value.
pub use crate::security::shared::SshPublicKey;

/// SSH key sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SshKeySortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
}

/// SSH key CRUD endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SshKeyEndpoint {
    /// `GET /ssh_keys`.
    List,
    /// `POST /ssh_keys`.
    Create,
    /// `GET /ssh_keys/{id}`.
    Get(SshKeyId),
    /// `PUT /ssh_keys/{id}`.
    Update(SshKeyId),
    /// `DELETE /ssh_keys/{id}`.
    Delete(SshKeyId),
}

impl SshKeyEndpoint {
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

    /// Returns the endpoint group from the source-locked API matrix.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::SshKeys
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, SecurityRequestError>> {
        match self {
            Self::List | Self::Create => Some(static_path("/ssh_keys")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) => None,
        }
    }

    /// Writes an endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        match self {
            Self::List | Self::Create => {
                let path = static_path("/ssh_keys")?;
                let bytes = path.as_str().as_bytes();
                if output.len() < bytes.len() {
                    return Err(SecurityRequestError::PathBufferTooSmall);
                }
                let target = output
                    .get_mut(..bytes.len())
                    .ok_or(SecurityRequestError::PathBufferTooSmall)?;
                target.copy_from_slice(bytes);
                Ok(bytes.len())
            }
            Self::Get(id) | Self::Update(id) | Self::Delete(id) => {
                write_id_path(output, "/ssh_keys/", id, "")
            }
        }
    }
}

/// SSH key list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SshKeyListRequest<'a> {
    name: Option<SshKeyName<'a>>,
    fingerprint: Option<&'a str>,
    label_selector: Option<LabelSelector<'a>>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(SshKeySortField, SortDirection)>,
}

impl<'a> SshKeyListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            fingerprint: None,
            label_selector: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: SshKeyName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets exact fingerprint filtering.
    pub fn with_fingerprint(mut self, fingerprint: &'a str) -> Result<Self, SecurityRequestError> {
        if fingerprint.is_empty()
            || fingerprint.len() > MAX_SSH_FINGERPRINT_BYTES
            || fingerprint
                .bytes()
                .any(|byte| !(byte.is_ascii_hexdigit() || byte == b':'))
        {
            return Err(SecurityRequestError::InvalidSshFingerprint);
        }
        self.fingerprint = Some(fingerprint);
        Ok(self)
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
    pub const fn with_sort(mut self, field: SshKeySortField, direction: SortDirection) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        let mut len = 0;
        let mut first = true;
        if let Some(fingerprint) = self.fingerprint {
            write_query_pair(output, &mut len, &mut first, "fingerprint", fingerprint)?;
        }
        if let Some(selector) = self.label_selector {
            write_query_pair(
                output,
                &mut len,
                &mut first,
                "label_selector",
                selector.as_str(),
            )?;
        }
        if let Some(name) = self.name {
            write_query_pair(output, &mut len, &mut first, "name", name.as_str())?;
        }
        if let Some(page) = self.page {
            write_query_u64(output, &mut len, &mut first, "page", page.get())?;
        }
        if let Some(per_page) = self.per_page {
            write_query_u64(
                output,
                &mut len,
                &mut first,
                "per_page",
                u64::from(per_page.get()),
            )?;
        }
        if let Some((field, direction)) = self.sort {
            write_query_pair(
                output,
                &mut len,
                &mut first,
                "sort",
                sort_value(field, direction),
            )?;
        }
        Ok(len)
    }
}

impl Default for SshKeyListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// SSH key create request fields.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SshKeyCreateRequest<'a> {
    name: SshKeyName<'a>,
    public_key: SshPublicKey<'a>,
    labels: Option<SecurityLabels<'a>>,
}

impl<'a> SshKeyCreateRequest<'a> {
    /// Creates a validated create request.
    #[must_use]
    pub const fn new(name: SshKeyName<'a>, public_key: SshPublicKey<'a>) -> Self {
        Self {
            name,
            public_key,
            labels: None,
        }
    }

    /// Adds validated labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: SecurityLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> SshKeyEndpoint {
        SshKeyEndpoint::Create
    }

    /// Returns the resource name.
    #[must_use]
    pub const fn name(self) -> SshKeyName<'a> {
        self.name
    }

    /// Returns the SSH public key.
    #[must_use]
    pub const fn public_key(self) -> SshPublicKey<'a> {
        self.public_key
    }

    pub(crate) const fn labels(self) -> Option<SecurityLabels<'a>> {
        self.labels
    }
}

impl core::fmt::Debug for SshKeyCreateRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SshKeyCreateRequest")
            .field("name", &self.name)
            .field("public_key", &"[redacted]")
            .field("labels", &self.labels)
            .finish()
    }
}

/// SSH key update request fields.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct SshKeyUpdateRequest<'a> {
    id: SshKeyId,
    name: Option<SshKeyName<'a>>,
    labels: Option<SecurityLabels<'a>>,
}

impl<'a> SshKeyUpdateRequest<'a> {
    /// Creates a validated update request.
    #[must_use]
    pub const fn new(id: SshKeyId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets the replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: SshKeyName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets replacement labels.
    #[must_use]
    pub const fn with_labels(mut self, labels: SecurityLabels<'a>) -> Self {
        self.labels = Some(labels);
        self
    }

    /// Returns the endpoint.
    #[must_use]
    pub const fn endpoint(self) -> SshKeyEndpoint {
        SshKeyEndpoint::Update(self.id)
    }

    pub(crate) const fn prepared_parts(
        self,
    ) -> (Option<SshKeyName<'a>>, Option<SecurityLabels<'a>>) {
        (self.name, self.labels)
    }
}

impl core::fmt::Debug for SshKeyUpdateRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("SshKeyUpdateRequest")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("labels", &self.labels)
            .finish()
    }
}

const fn sort_value(field: SshKeySortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (SshKeySortField::Id, SortDirection::Asc) => "id:asc",
        (SshKeySortField::Id, SortDirection::Desc) => "id:desc",
        (SshKeySortField::Name, SortDirection::Asc) => "name:asc",
        (SshKeySortField::Name, SortDirection::Desc) => "name:desc",
    }
}

#[cfg(test)]
mod tests;
