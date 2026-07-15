//! Certificate endpoint request domains.

mod actions;
mod private_key;

use cloud_sdk::Method;

use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::{ApiBaseUrl, EndpointPath};
use crate::security::shared::{
    PemValue, static_path, write_id_path, write_query_pair, write_query_u64,
};

pub use actions::{
    CertificateActionEndpoint, CertificateActionListForCertificateRequest,
    CertificateActionListRequest, CertificateActionSortField, MAX_CERTIFICATE_ACTION_IDS,
    MAX_CERTIFICATE_ACTION_SORTS, MAX_CERTIFICATE_ACTION_STATUSES,
};
pub use private_key::{PrivateKeyPem, private_key_pem};

/// Certificate endpoint groups.
pub const ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Certificates,
    EndpointGroup::CertificateActions,
];

/// Certificate identifier.
pub use crate::security::shared::SecurityId as CertificateId;

/// Certificate resource name.
pub use crate::security::shared::SecurityName as CertificateName;

/// Uploaded certificate PEM value.
pub type CertificatePem<'a> = PemValue<'a>;

/// Managed certificate domain name.
pub use crate::security::shared::CertificateDomainName;

/// Certificate request labels.
pub use crate::security::shared::SecurityLabels;

/// Certificate request error.
pub use crate::security::shared::SecurityRequestError;

/// Certificate sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Certificate type filter and create-mode selector.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateType {
    /// Uploaded PEM certificate.
    Uploaded,
    /// Managed Let's Encrypt certificate.
    Managed,
}

impl CertificateType {
    const fn as_api_str(self) -> &'static str {
        match self {
            Self::Uploaded => "uploaded",
            Self::Managed => "managed",
        }
    }
}

/// Certificate CRUD and retry endpoint.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CertificateEndpoint {
    /// `GET /certificates`.
    List,
    /// `POST /certificates`.
    Create,
    /// `GET /certificates/{id}`.
    Get(CertificateId),
    /// `PUT /certificates/{id}`.
    Update(CertificateId),
    /// `DELETE /certificates/{id}`.
    Delete(CertificateId),
    /// `POST /certificates/{id}/actions/retry`.
    Retry(CertificateId),
}

impl CertificateEndpoint {
    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        match self {
            Self::List | Self::Get(_) => Method::Get,
            Self::Create | Self::Retry(_) => Method::Post,
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
        match self {
            Self::Retry(_) => EndpointGroup::CertificateActions,
            Self::List | Self::Create | Self::Get(_) | Self::Update(_) | Self::Delete(_) => {
                EndpointGroup::Certificates
            }
        }
    }

    /// Returns a static endpoint path when no ID is required.
    pub fn static_path(self) -> Option<Result<EndpointPath<'static>, SecurityRequestError>> {
        match self {
            Self::List | Self::Create => Some(static_path("/certificates")),
            Self::Get(_) | Self::Update(_) | Self::Delete(_) | Self::Retry(_) => None,
        }
    }

    /// Writes an endpoint path into a caller-owned buffer.
    pub fn write_path(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        match self {
            Self::List | Self::Create => {
                let path = static_path("/certificates")?;
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
                write_id_path(output, "/certificates/", id, "")
            }
            Self::Retry(id) => write_id_path(output, "/certificates/", id, "/actions/retry"),
        }
    }
}

/// Certificate list request.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateListRequest<'a> {
    name: Option<CertificateName<'a>>,
    label_selector: Option<LabelSelector<'a>>,
    certificate_type: Option<CertificateType>,
    page: Option<Page>,
    per_page: Option<PerPage>,
    sort: Option<(CertificateSortField, SortDirection)>,
}

impl<'a> CertificateListRequest<'a> {
    /// Creates an empty list request.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            name: None,
            label_selector: None,
            certificate_type: None,
            page: None,
            per_page: None,
            sort: None,
        }
    }

    /// Sets exact name filtering.
    #[must_use]
    pub const fn with_name(mut self, name: CertificateName<'a>) -> Self {
        self.name = Some(name);
        self
    }

    /// Sets label-selector filtering.
    #[must_use]
    pub const fn with_label_selector(mut self, selector: LabelSelector<'a>) -> Self {
        self.label_selector = Some(selector);
        self
    }

    /// Sets certificate type filtering.
    #[must_use]
    pub const fn with_type(mut self, certificate_type: CertificateType) -> Self {
        self.certificate_type = Some(certificate_type);
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
    pub const fn with_sort(
        mut self,
        field: CertificateSortField,
        direction: SortDirection,
    ) -> Self {
        self.sort = Some((field, direction));
        self
    }

    /// Writes the query string into a caller-owned buffer.
    pub fn write_query(self, output: &mut [u8]) -> Result<usize, SecurityRequestError> {
        let mut len = 0;
        let mut first = true;
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
        if let Some(certificate_type) = self.certificate_type {
            write_query_pair(
                output,
                &mut len,
                &mut first,
                "type",
                certificate_type.as_api_str(),
            )?;
        }
        Ok(len)
    }
}

impl Default for CertificateListRequest<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Certificate create mode.
#[derive(Clone, Copy)]
pub enum CertificateCreateMode<'a> {
    /// Uploaded PEM certificate and private key.
    Uploaded {
        /// Certificate and chain in PEM format.
        certificate: CertificatePem<'a>,
        /// Private key in PEM format.
        private_key: PrivateKeyPem<'a>,
    },
    /// Managed Let's Encrypt certificate domains.
    Managed {
        /// Domains and subdomains for the managed certificate.
        domain_names: &'a [CertificateDomainName<'a>],
    },
}

impl<'a> CertificateCreateMode<'a> {
    /// Creates the uploaded certificate mode.
    ///
    /// This only records values that already passed PEM marker validation. It
    /// does not prove that the private key matches the certificate; that would
    /// require ASN.1/crypto validation outside this no_std request-domain
    /// layer.
    pub fn uploaded(certificate: CertificatePem<'a>, private_key: PrivateKeyPem<'a>) -> Self {
        Self::Uploaded {
            certificate,
            private_key,
        }
    }

    /// Creates the managed certificate mode.
    pub fn managed(
        domain_names: &'a [CertificateDomainName<'a>],
    ) -> Result<Self, SecurityRequestError> {
        if domain_names.is_empty() {
            return Err(SecurityRequestError::EmptyDomainNames);
        }
        Ok(Self::Managed { domain_names })
    }

    /// Returns the API type value for this create mode.
    #[must_use]
    pub const fn certificate_type(self) -> CertificateType {
        match self {
            Self::Uploaded { .. } => CertificateType::Uploaded,
            Self::Managed { .. } => CertificateType::Managed,
        }
    }
}

impl core::fmt::Debug for CertificateCreateMode<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Uploaded { .. } => formatter
                .debug_struct("Uploaded")
                .field("certificate", &"[redacted]")
                .field("private_key", &"[redacted]")
                .finish(),
            Self::Managed { domain_names } => formatter
                .debug_struct("Managed")
                .field("domain_names", domain_names)
                .finish(),
        }
    }
}

/// Certificate create request fields.
#[derive(Clone, Copy)]
pub struct CertificateCreateRequest<'a> {
    name: CertificateName<'a>,
    mode: CertificateCreateMode<'a>,
    labels: Option<SecurityLabels<'a>>,
}

impl<'a> CertificateCreateRequest<'a> {
    /// Creates a validated create request.
    #[must_use]
    pub const fn new(name: CertificateName<'a>, mode: CertificateCreateMode<'a>) -> Self {
        Self {
            name,
            mode,
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
    pub const fn endpoint(self) -> CertificateEndpoint {
        CertificateEndpoint::Create
    }

    /// Returns the create mode.
    #[must_use]
    pub const fn mode(self) -> CertificateCreateMode<'a> {
        self.mode
    }

    pub(crate) const fn prepared_parts(self) -> (CertificateName<'a>, Option<SecurityLabels<'a>>) {
        (self.name, self.labels)
    }
}

impl core::fmt::Debug for CertificateCreateRequest<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CertificateCreateRequest")
            .field("name", &self.name)
            .field("mode", &self.mode)
            .field("labels", &self.labels)
            .finish()
    }
}

/// Certificate update request fields.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CertificateUpdateRequest<'a> {
    id: CertificateId,
    name: Option<CertificateName<'a>>,
    labels: Option<SecurityLabels<'a>>,
}

impl<'a> CertificateUpdateRequest<'a> {
    /// Creates a validated update request.
    #[must_use]
    pub const fn new(id: CertificateId) -> Self {
        Self {
            id,
            name: None,
            labels: None,
        }
    }

    /// Sets the replacement name.
    #[must_use]
    pub const fn with_name(mut self, name: CertificateName<'a>) -> Self {
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
    pub const fn endpoint(self) -> CertificateEndpoint {
        CertificateEndpoint::Update(self.id)
    }

    pub(crate) const fn prepared_parts(
        self,
    ) -> (Option<CertificateName<'a>>, Option<SecurityLabels<'a>>) {
        (self.name, self.labels)
    }
}

/// Creates a validated certificate PEM value.
pub fn certificate_pem(value: &str) -> Result<CertificatePem<'_>, SecurityRequestError> {
    PemValue::new(
        value,
        "-----BEGIN CERTIFICATE-----",
        "-----END CERTIFICATE-----",
    )
}

const fn sort_value(field: CertificateSortField, direction: SortDirection) -> &'static str {
    match (field, direction) {
        (CertificateSortField::Id, SortDirection::Asc) => "id:asc",
        (CertificateSortField::Id, SortDirection::Desc) => "id:desc",
        (CertificateSortField::Name, SortDirection::Asc) => "name:asc",
        (CertificateSortField::Name, SortDirection::Desc) => "name:desc",
        (CertificateSortField::Created, SortDirection::Asc) => "created:asc",
        (CertificateSortField::Created, SortDirection::Desc) => "created:desc",
    }
}

#[cfg(test)]
mod tests;
