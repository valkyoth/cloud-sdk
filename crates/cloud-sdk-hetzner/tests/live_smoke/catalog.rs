use core::{fmt, str};

use cloud_sdk::Method;
use cloud_sdk::transport::{
    BlockingTransport, RequestTarget, RequestTargetError, StatusCode, TransportRequest,
};
use cloud_sdk_hetzner::cloud::catalog::{
    CatalogListEndpoint, CatalogListRequest, CatalogRequestError, CatalogSingletonEndpoint,
    PublicImageKind,
};
use cloud_sdk_hetzner::pagination::{PaginationError, PerPage};
use cloud_sdk_hetzner::serde::PaginationEnvelope;
use cloud_sdk_reqwest::blocking::{BlockingClient, TransportError};
use cloud_sdk_sanitization::SecretBuffer;
use serde_json::Value;

const MAX_TARGET_BYTES: usize = 256;
const MAX_RESPONSE_BYTES: usize = 1_048_576;

pub(super) const PROBES: [CatalogProbe; 6] = [
    CatalogProbe::list("locations", "locations", CatalogListEndpoint::Locations),
    CatalogProbe::list(
        "server-types",
        "server_types",
        CatalogListEndpoint::ServerTypes,
    ),
    CatalogProbe::list(
        "load-balancer-types",
        "load_balancer_types",
        CatalogListEndpoint::LoadBalancerTypes,
    ),
    CatalogProbe::list("isos", "isos", CatalogListEndpoint::Isos),
    CatalogProbe::list(
        "public-images",
        "images",
        CatalogListEndpoint::PublicImages(PublicImageKind::System),
    ),
    CatalogProbe::singleton("pricing", "pricing", CatalogSingletonEndpoint::Pricing),
];

#[derive(Clone, Copy, Debug)]
enum ProbeRequest {
    List(CatalogListEndpoint),
    Singleton(CatalogSingletonEndpoint),
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CatalogProbe {
    name: &'static str,
    response_key: &'static str,
    request: ProbeRequest,
}

impl CatalogProbe {
    const fn list(
        name: &'static str,
        response_key: &'static str,
        endpoint: CatalogListEndpoint,
    ) -> Self {
        Self {
            name,
            response_key,
            request: ProbeRequest::List(endpoint),
        }
    }

    const fn singleton(
        name: &'static str,
        response_key: &'static str,
        endpoint: CatalogSingletonEndpoint,
    ) -> Self {
        Self {
            name,
            response_key,
            request: ProbeRequest::Singleton(endpoint),
        }
    }

    pub(super) const fn name(self) -> &'static str {
        self.name
    }

    pub(super) fn run(self, client: &mut BlockingClient) -> Result<(), ProbeFailure> {
        let mut target_bytes = [0_u8; MAX_TARGET_BYTES];
        let target_len = self
            .write_target(&mut target_bytes)
            .map_err(|kind| ProbeFailure::new(self.name, kind))?;
        let target_bytes = target_bytes
            .get(..target_len)
            .ok_or_else(|| ProbeFailure::new(self.name, ProbeError::TargetEncoding))?;
        let target_text = str::from_utf8(target_bytes)
            .map_err(|_| ProbeFailure::new(self.name, ProbeError::TargetEncoding))?;
        let target = RequestTarget::new(target_text)
            .map_err(|error| ProbeFailure::new(self.name, ProbeError::Target(error)))?;
        let request = TransportRequest::new(Method::Get, target);

        let mut response_storage = vec![0_u8; MAX_RESPONSE_BYTES];
        let mut guarded = SecretBuffer::new(response_storage.as_mut_slice());
        let response = client
            .send(request, guarded.as_mut_slice())
            .map_err(|error| ProbeFailure::new(self.name, ProbeError::Transport(error)))?;
        if response.status() != StatusCode::OK {
            return Err(ProbeFailure::new(
                self.name,
                ProbeError::UnexpectedStatus(response.status().get()),
            ));
        }
        self.validate_body(response.body())
            .map_err(|kind| ProbeFailure::new(self.name, kind))
    }

    fn write_target(self, output: &mut [u8]) -> Result<usize, ProbeError> {
        match self.request {
            ProbeRequest::List(endpoint) => {
                let per_page = PerPage::new(1).map_err(ProbeError::Pagination)?;
                let request = CatalogListRequest::new(endpoint)
                    .with_per_page(per_page)
                    .map_err(ProbeError::Catalog)?;
                if request.method() != Method::Get {
                    return Err(ProbeError::NonReadOnlyMethod);
                }
                let path = endpoint.path().map_err(ProbeError::Catalog)?;
                let mut len = append(output, 0, path.as_str().as_bytes())?;
                len = append(output, len, b"?")?;
                let query_output = output.get_mut(len..).ok_or(ProbeError::TargetTooSmall)?;
                let query_len = request
                    .write_query(query_output)
                    .map_err(ProbeError::Catalog)?;
                len.checked_add(query_len).ok_or(ProbeError::TargetTooSmall)
            }
            ProbeRequest::Singleton(endpoint) => {
                let path = endpoint.path().map_err(ProbeError::Catalog)?;
                append(output, 0, path.as_str().as_bytes())
            }
        }
    }

    fn validate_body(self, body: &[u8]) -> Result<(), ProbeError> {
        let value: Value = serde_json::from_slice(body).map_err(|_| ProbeError::InvalidJson)?;
        let object = value.as_object().ok_or(ProbeError::InvalidEnvelope)?;
        let resource = object
            .get(self.response_key)
            .ok_or(ProbeError::MissingResource)?;
        match self.request {
            ProbeRequest::List(_) => {
                if !resource.is_array() {
                    return Err(ProbeError::InvalidResourceShape);
                }
                let pagination: PaginationEnvelope =
                    serde_json::from_slice(body).map_err(|_| ProbeError::InvalidPagination)?;
                if pagination.pagination().page().get() != 1
                    || pagination.pagination().per_page().get() != 1
                {
                    return Err(ProbeError::InvalidPagination);
                }
            }
            ProbeRequest::Singleton(_) if !resource.is_object() => {
                return Err(ProbeError::InvalidResourceShape);
            }
            ProbeRequest::Singleton(_) => {}
        }
        Ok(())
    }
}

fn append(output: &mut [u8], len: usize, value: &[u8]) -> Result<usize, ProbeError> {
    let end = len
        .checked_add(value.len())
        .ok_or(ProbeError::TargetTooSmall)?;
    let destination = output.get_mut(len..end).ok_or(ProbeError::TargetTooSmall)?;
    destination.copy_from_slice(value);
    Ok(end)
}

#[derive(Clone, Copy)]
enum ProbeError {
    Catalog(CatalogRequestError),
    Pagination(PaginationError),
    Target(RequestTargetError),
    Transport(TransportError),
    TargetTooSmall,
    TargetEncoding,
    NonReadOnlyMethod,
    UnexpectedStatus(u16),
    InvalidJson,
    InvalidEnvelope,
    MissingResource,
    InvalidResourceShape,
    InvalidPagination,
}

impl fmt::Debug for ProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Catalog(error) => formatter.debug_tuple("Catalog").field(error).finish(),
            Self::Pagination(error) => formatter.debug_tuple("Pagination").field(error).finish(),
            Self::Target(error) => formatter.debug_tuple("Target").field(error).finish(),
            Self::Transport(error) => formatter.debug_tuple("Transport").field(error).finish(),
            Self::UnexpectedStatus(status) => formatter
                .debug_tuple("UnexpectedStatus")
                .field(status)
                .finish(),
            Self::TargetTooSmall => formatter.write_str("TargetTooSmall"),
            Self::TargetEncoding => formatter.write_str("TargetEncoding"),
            Self::NonReadOnlyMethod => formatter.write_str("NonReadOnlyMethod"),
            Self::InvalidJson => formatter.write_str("InvalidJson"),
            Self::InvalidEnvelope => formatter.write_str("InvalidEnvelope"),
            Self::MissingResource => formatter.write_str("MissingResource"),
            Self::InvalidResourceShape => formatter.write_str("InvalidResourceShape"),
            Self::InvalidPagination => formatter.write_str("InvalidPagination"),
        }
    }
}

#[derive(Clone, Copy)]
pub(super) struct ProbeFailure {
    probe: &'static str,
    kind: ProbeError,
}

impl fmt::Debug for ProbeFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ProbeFailure")
            .field("probe", &self.probe)
            .field("kind", &self.kind)
            .finish()
    }
}

impl ProbeFailure {
    const fn new(probe: &'static str, kind: ProbeError) -> Self {
        Self { probe, kind }
    }
}

#[cfg(test)]
mod tests {
    use super::{MAX_TARGET_BYTES, PROBES, ProbeError};

    #[test]
    fn every_probe_builds_a_bounded_read_only_target() {
        let expected = [
            "/locations?per_page=1",
            "/server_types?per_page=1",
            "/load_balancer_types?per_page=1",
            "/isos?per_page=1",
            "/images?type=system&per_page=1",
            "/pricing",
        ];
        for (probe, expected_target) in PROBES.into_iter().zip(expected) {
            let mut output = [0_u8; MAX_TARGET_BYTES];
            let built = probe.write_target(&mut output);
            assert!(built.is_ok());
            let Ok(len) = built else { continue };
            assert_eq!(output.get(..len), Some(expected_target.as_bytes()));
        }
    }

    #[test]
    fn target_writes_fail_closed_without_partial_length_claims() {
        for probe in PROBES {
            assert!(matches!(
                probe.write_target(&mut [0_u8; 4]),
                Err(ProbeError::TargetTooSmall | ProbeError::Catalog(_))
            ));
        }
    }

    #[test]
    fn response_validation_accepts_only_expected_shapes_and_pagination() {
        let Some(locations) = PROBES.first().copied() else {
            return;
        };
        assert!(locations
            .validate_body(br#"{"locations":[],"meta":{"pagination":{"page":1,"per_page":1,"previous_page":null,"next_page":null,"last_page":1,"total_entries":0}}}"#)
            .is_ok());
        assert!(matches!(
            locations.validate_body(br#"{"servers":[]}"#),
            Err(ProbeError::MissingResource)
        ));
        assert!(matches!(
            locations.validate_body(br#"{"locations":{},"meta":{"pagination":{"page":1,"per_page":1,"previous_page":null,"next_page":null,"last_page":1,"total_entries":0}}}"#),
            Err(ProbeError::InvalidResourceShape)
        ));
    }

    #[test]
    fn probe_failures_never_embed_response_content() {
        let failure = super::ProbeFailure::new("locations", ProbeError::InvalidJson);
        let diagnostic = format!("{failure:?}");
        assert!(diagnostic.contains("locations"));
        assert!(!diagnostic.contains("secret-token"));
        assert!(!diagnostic.contains("12345678"));
    }
}
