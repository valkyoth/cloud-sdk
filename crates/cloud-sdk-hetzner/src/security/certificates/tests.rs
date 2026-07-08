use super::{
    CertificateCreateMode, CertificateCreateRequest, CertificateDomainName, CertificateEndpoint,
    CertificateId, CertificateListRequest, CertificateName, CertificateSortField, CertificateType,
    certificate_pem, private_key_pem,
};
use crate::EndpointGroup;
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;
use crate::security::shared::SecurityRequestError;
use core::fmt::Write;

const CERT: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----";
const KEY: &str = "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----";

#[test]
fn certificate_paths_match_api_matrix() {
    let id = CertificateId::new(42);
    let mut output = [0u8; 64];
    if let Some(id) = id {
        assert_eq!(CertificateEndpoint::List.write_path(&mut output), Ok(13));
        assert_eq!(CertificateEndpoint::Get(id).write_path(&mut output), Ok(16));
        assert_eq!(
            CertificateEndpoint::Retry(id).write_path(&mut output),
            Ok(30)
        );
        let path = output
            .get(..30)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/certificates/42/actions/retry"));
        assert_eq!(CertificateEndpoint::Retry(id).method().as_str(), "POST");
        assert_eq!(
            CertificateEndpoint::Retry(id).api_base_url(),
            ApiBaseUrl::CloudV1
        );
        assert_eq!(
            CertificateEndpoint::Retry(id).endpoint_group(),
            EndpointGroup::CertificateActions
        );
    }
}

#[test]
fn certificate_list_query_writes_filters_pagination_and_sorting() {
    let name = CertificateName::new("web cert");
    let selector = LabelSelector::new("env=prod");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 160];
    if let (Ok(name), Ok(selector), Ok(page), Ok(per_page)) = (name, selector, page, per_page) {
        let request = CertificateListRequest::new()
            .with_name(name)
            .with_label_selector(selector)
            .with_type(CertificateType::Managed)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(CertificateSortField::Created, SortDirection::Desc);
        assert_eq!(request.write_query(&mut output), Ok(93));
        let query = output
            .get(..93)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some(
                "label_selector=env%3Dprod&name=web%20cert&page=2&per_page=25&sort=created%3Adesc&type=managed"
            )
        );
    }
}

#[test]
fn certificate_create_modes_validate_required_fields_and_redact_debug() {
    let name = CertificateName::new("web");
    let certificate = certificate_pem(CERT);
    let private_key = private_key_pem(KEY);
    if let (Ok(name), Ok(certificate), Ok(private_key)) = (name, certificate, private_key) {
        let mode = CertificateCreateMode::uploaded(certificate, private_key);
        assert_eq!(
            CertificateCreateRequest::try_new(None, Some(mode)),
            Err(SecurityRequestError::MissingRequiredField)
        );
        let request = CertificateCreateRequest::try_new(Some(name), Some(mode));
        assert!(request.is_ok());
        if let Ok(request) = request {
            let mut debug = DebugBuffer::new();
            assert!(write!(&mut debug, "{request:?}").is_ok());
            let debug = debug.as_str();
            assert!(debug.contains("[redacted]"));
            assert!(!debug.contains("PRIVATE KEY"));
            assert_eq!(request.mode().certificate_type(), CertificateType::Uploaded);
        }
    }
}

#[test]
fn managed_certificate_domains_are_validated() {
    assert_eq!(
        CertificateDomainName::new("-bad.example"),
        Err(SecurityRequestError::InvalidDomainName)
    );
    let domain = CertificateDomainName::new("*.example.com");
    if let Ok(domain) = domain {
        let domains = [domain];
        assert!(CertificateCreateMode::managed(&domains).is_ok());
    }
    assert_eq!(
        CertificateCreateMode::managed(&[]),
        Err(SecurityRequestError::EmptyDomainNames)
    );
}

#[test]
fn pem_validation_rejects_wrong_markers() {
    assert_eq!(
        certificate_pem("-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----"),
        Err(SecurityRequestError::InvalidPem)
    );
    assert_eq!(
        certificate_pem("-----END CERTIFICATE-----\nMIIB\n-----BEGIN CERTIFICATE-----"),
        Err(SecurityRequestError::InvalidPem)
    );
    assert_eq!(
        certificate_pem("-----BEGIN CERTIFICATE----------END CERTIFICATE-----"),
        Err(SecurityRequestError::InvalidPem)
    );
    assert_eq!(private_key_pem(KEY).map(|pem| pem.as_str()), Ok(KEY));
}

struct DebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0u8; 256],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
