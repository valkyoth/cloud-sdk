//! Certificate action endpoint and list-query models.

use cloud_sdk::{Method, buffer};

use super::{CertificateId, SecurityRequestError};
use crate::EndpointGroup;
use crate::actions::{ActionId, ActionStatus};
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};
use crate::security::shared::{write_id_path, write_query_pair, write_query_u64};

/// Maximum repeated action IDs admitted by one certificate action query.
pub const MAX_CERTIFICATE_ACTION_IDS: usize = 128;
/// Maximum repeated status filters admitted by one certificate action query.
pub const MAX_CERTIFICATE_ACTION_STATUSES: usize = 3;
/// Maximum repeated sort values admitted by one certificate action query.
pub const MAX_CERTIFICATE_ACTION_SORTS: usize = 5;

/// Certificate action sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateActionSortField {
    /// Sort by action ID.
    Id,
    /// Sort by action command.
    Command,
    /// Sort by action status.
    Status,
    /// Sort by start timestamp.
    Started,
    /// Sort by finish timestamp.
    Finished,
}

/// Non-deprecated certificate action endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateActionEndpoint {
    /// `GET /certificates/actions`.
    ListAll,
    /// `GET /certificates/actions/{id}`.
    Get(ActionId),
    /// `GET /certificates/{id}/actions`.
    ListForCertificate(CertificateId),
}

impl CertificateActionEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        Method::Get
    }

    /// Returns the Cloud API base URL family.
    #[must_use]
    pub const fn api_base_url(self) -> ApiBaseUrl {
        ApiBaseUrl::CloudV1
    }

    /// Returns the source-locked endpoint group.
    #[must_use]
    pub const fn endpoint_group(self) -> EndpointGroup {
        EndpointGroup::CertificateActions
    }

    /// Returns the static global-list path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, SecurityRequestError>> {
        match self {
            Self::ListAll => Some(
                EndpointPath::new("/certificates/actions")
                    .map_err(SecurityRequestError::InvalidPath),
            ),
            Self::Get(_) | Self::ListForCertificate(_) => None,
        }
    }

    /// Writes the endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        match self {
            Self::ListAll => write_static_path(output, "/certificates/actions"),
            Self::Get(id) => write_action_id_path(output, "/certificates/actions/", id),
            Self::ListForCertificate(id) => write_id_path(output, "/certificates/", id, "/actions"),
        }
    }
}

/// Query for `GET /certificates/actions`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateActionListRequest<'a> {
    action_ids: &'a [ActionId],
    statuses: &'a [ActionStatus],
    sorts: &'a [(CertificateActionSortField, SortDirection)],
    page: Option<Page>,
    per_page: Option<PerPage>,
}

impl<'a> CertificateActionListRequest<'a> {
    /// Creates an empty global certificate action query.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            action_ids: &[],
            statuses: &[],
            sorts: &[],
            page: None,
            per_page: None,
        }
    }

    /// Sets bounded repeated action-ID filters.
    pub fn with_action_ids(
        mut self,
        action_ids: &'a [ActionId],
    ) -> Result<Self, SecurityRequestError> {
        validate_len(
            action_ids.len(),
            MAX_CERTIFICATE_ACTION_IDS,
            SecurityRequestError::TooManyActionIds,
        )?;
        self.action_ids = action_ids;
        Ok(self)
    }

    /// Sets bounded repeated action-status filters.
    pub fn with_statuses(
        mut self,
        statuses: &'a [ActionStatus],
    ) -> Result<Self, SecurityRequestError> {
        validate_len(
            statuses.len(),
            MAX_CERTIFICATE_ACTION_STATUSES,
            SecurityRequestError::TooManyActionStatuses,
        )?;
        self.statuses = statuses;
        Ok(self)
    }

    /// Sets bounded repeated sort values.
    pub fn with_sorts(
        mut self,
        sorts: &'a [(CertificateActionSortField, SortDirection)],
    ) -> Result<Self, SecurityRequestError> {
        validate_len(
            sorts.len(),
            MAX_CERTIFICATE_ACTION_SORTS,
            SecurityRequestError::TooManyActionSorts,
        )?;
        self.sorts = sorts;
        Ok(self)
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per-page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Returns the global certificate action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> CertificateActionEndpoint {
        CertificateActionEndpoint::ListAll
    }

    /// Writes the query into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        write_list_query(
            output,
            self.action_ids,
            self.page,
            self.per_page,
            self.sorts,
            self.statuses,
        )
    }
}

impl Default for CertificateActionListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Query for `GET /certificates/{id}/actions`.
///
/// Hetzner returns an empty action list for uploaded certificates; only managed
/// certificates can have actions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateActionListForCertificateRequest<'a> {
    certificate_id: CertificateId,
    statuses: &'a [ActionStatus],
    sorts: &'a [(CertificateActionSortField, SortDirection)],
    page: Option<Page>,
    per_page: Option<PerPage>,
}

impl<'a> CertificateActionListForCertificateRequest<'a> {
    /// Creates an empty query bound to one certificate.
    #[must_use]
    pub const fn new(certificate_id: CertificateId) -> Self {
        Self {
            certificate_id,
            statuses: &[],
            sorts: &[],
            page: None,
            per_page: None,
        }
    }

    /// Sets bounded repeated action-status filters.
    pub fn with_statuses(
        mut self,
        statuses: &'a [ActionStatus],
    ) -> Result<Self, SecurityRequestError> {
        validate_len(
            statuses.len(),
            MAX_CERTIFICATE_ACTION_STATUSES,
            SecurityRequestError::TooManyActionStatuses,
        )?;
        self.statuses = statuses;
        Ok(self)
    }

    /// Sets bounded repeated sort values.
    pub fn with_sorts(
        mut self,
        sorts: &'a [(CertificateActionSortField, SortDirection)],
    ) -> Result<Self, SecurityRequestError> {
        validate_len(
            sorts.len(),
            MAX_CERTIFICATE_ACTION_SORTS,
            SecurityRequestError::TooManyActionSorts,
        )?;
        self.sorts = sorts;
        Ok(self)
    }

    /// Sets the page value.
    #[must_use]
    pub const fn with_page(mut self, page: Page) -> Self {
        self.page = Some(page);
        self
    }

    /// Sets the per-page value.
    #[must_use]
    pub const fn with_per_page(mut self, per_page: PerPage) -> Self {
        self.per_page = Some(per_page);
        self
    }

    /// Returns the certificate-local action endpoint.
    #[must_use]
    pub const fn endpoint(self) -> CertificateActionEndpoint {
        CertificateActionEndpoint::ListForCertificate(self.certificate_id)
    }

    /// Writes the query into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        write_list_query(
            output,
            &[],
            self.page,
            self.per_page,
            self.sorts,
            self.statuses,
        )
    }
}

fn write_list_query(
    output: &mut [u8],
    action_ids: &[ActionId],
    page: Option<Page>,
    per_page: Option<PerPage>,
    sorts: &[(CertificateActionSortField, SortDirection)],
    statuses: &[ActionStatus],
) -> Result<usize, SecurityRequestError> {
    let mut len = 0;
    let mut first = true;
    for id in action_ids {
        write_query_u64(output, &mut len, &mut first, "id", id.get())?;
    }
    if let Some(page) = page {
        write_query_u64(output, &mut len, &mut first, "page", page.get())?;
    }
    if let Some(per_page) = per_page {
        write_query_u64(
            output,
            &mut len,
            &mut first,
            "per_page",
            u64::from(per_page.get()),
        )?;
    }
    for (field, direction) in sorts {
        write_query_pair(
            output,
            &mut len,
            &mut first,
            "sort",
            sort_value(*field, *direction),
        )?;
    }
    for status in statuses {
        write_query_pair(output, &mut len, &mut first, "status", status.as_api_str())?;
    }
    Ok(len)
}

fn write_static_path(output: &mut [u8], path: &str) -> Result<usize, SecurityRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        path,
        SecurityRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

fn write_action_id_path(
    output: &mut [u8],
    prefix: &str,
    id: ActionId,
) -> Result<usize, SecurityRequestError> {
    let mut len = 0;
    buffer::write_str(
        output,
        &mut len,
        prefix,
        SecurityRequestError::PathBufferTooSmall,
    )?;
    buffer::write_u64(
        output,
        &mut len,
        id.get(),
        SecurityRequestError::PathBufferTooSmall,
    )?;
    validate_written_path(output, len)?;
    Ok(len)
}

fn validate_written_path(output: &[u8], len: usize) -> Result<(), SecurityRequestError> {
    let bytes = output
        .get(..len)
        .ok_or(SecurityRequestError::PathBufferTooSmall)?;
    let path = core::str::from_utf8(bytes).map_err(|_| SecurityRequestError::PathEncodingFailed)?;
    EndpointPath::new(path).map_err(SecurityRequestError::InvalidPath)?;
    Ok(())
}

fn validate_len(
    actual: usize,
    maximum: usize,
    error: SecurityRequestError,
) -> Result<(), SecurityRequestError> {
    if actual > maximum {
        return Err(error);
    }
    Ok(())
}

const fn sort_value(field: CertificateActionSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (CertificateActionSortField::Id, SortDirection::Asc) => "id:asc",
        (CertificateActionSortField::Id, SortDirection::Desc) => "id:desc",
        (CertificateActionSortField::Command, SortDirection::Asc) => "command:asc",
        (CertificateActionSortField::Command, SortDirection::Desc) => "command:desc",
        (CertificateActionSortField::Status, SortDirection::Asc) => "status:asc",
        (CertificateActionSortField::Status, SortDirection::Desc) => "status:desc",
        (CertificateActionSortField::Started, SortDirection::Asc) => "started:asc",
        (CertificateActionSortField::Started, SortDirection::Desc) => "started:desc",
        (CertificateActionSortField::Finished, SortDirection::Asc) => "finished:asc",
        (CertificateActionSortField::Finished, SortDirection::Desc) => "finished:desc",
    }
}

#[cfg(test)]
mod tests;
