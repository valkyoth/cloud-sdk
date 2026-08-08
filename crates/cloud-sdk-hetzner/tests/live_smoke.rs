//! Opt-in live smoke coverage for the public Hetzner Cloud catalog.

use core::fmt;

#[path = "live_smoke/catalog.rs"]
mod catalog;
#[path = "live_smoke/config.rs"]
mod config;

use std::time::Duration;

use cloud_sdk::operation::PreparationStorage;
use cloud_sdk_hetzner::association::AssociatedOperation;
use cloud_sdk_hetzner::association::operations::ListZones;
use cloud_sdk_hetzner::association::operations::{ListCertificates, ListSshKeys};
use cloud_sdk_hetzner::dns::zones::{ZoneEndpoint, ZoneListRequest};
use cloud_sdk_hetzner::official_endpoint_policy;
use cloud_sdk_hetzner::pagination::{Page, PerPage};
use cloud_sdk_hetzner::request::{ApiBaseUrl, CLOUD_API_BASE_URL};
use cloud_sdk_hetzner::security::certificates::{CertificateEndpoint, CertificateListRequest};
use cloud_sdk_hetzner::security::ssh_keys::{SshKeyEndpoint, SshKeyListRequest};
use cloud_sdk_hetzner::serde::{
    DnsResource, HetznerSuccess, SecurityResourceKind, decode_associated_checked_response,
};
use cloud_sdk_hetzner::{
    CLOUD_SERVICE_ID, DNS_SERVICE_ID, HETZNER_PROVIDER_ID, SECURITY_SERVICE_ID,
};
use cloud_sdk_reqwest::blocking::{
    BearerCredential, BearerCredentialScope, BlockingClientBuilder, BuildError, EndpointError,
    HttpsEndpoint, RequestTimeouts, TimeoutError, UserAgent, UserAgentError,
};

use catalog::{PROBES, ProbeFailure};
use config::{LiveConfigurationError, load_read_only_token};

enum LiveSmokeError {
    Configuration(LiveConfigurationError),
    Endpoint(EndpointError),
    UserAgent(UserAgentError),
    Timeout(TimeoutError),
    Client(BuildError),
    Probe(ProbeFailure),
    DnsProbe(DnsProbeStage),
    SecurityProbe(SecurityProbeStage),
}

#[derive(Clone, Copy, Debug)]
enum DnsProbeStage {
    Pagination,
    Association,
    Preparation,
    Transport,
    Decode,
    Shape,
}

#[derive(Clone, Copy, Debug)]
enum SecurityProbeStage {
    Pagination,
    Association,
    Preparation,
    Transport,
    Decode,
    Shape,
}

impl fmt::Debug for LiveSmokeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Configuration(error) => {
                formatter.debug_tuple("Configuration").field(error).finish()
            }
            Self::Endpoint(error) => formatter.debug_tuple("Endpoint").field(error).finish(),
            Self::UserAgent(error) => formatter.debug_tuple("UserAgent").field(error).finish(),
            Self::Timeout(error) => formatter.debug_tuple("Timeout").field(error).finish(),
            Self::Client(error) => formatter.debug_tuple("Client").field(error).finish(),
            Self::Probe(error) => formatter.debug_tuple("Probe").field(error).finish(),
            Self::DnsProbe(stage) => formatter.debug_tuple("DnsProbe").field(stage).finish(),
            Self::SecurityProbe(stage) => {
                formatter.debug_tuple("SecurityProbe").field(stage).finish()
            }
        }
    }
}

impl From<LiveConfigurationError> for LiveSmokeError {
    fn from(error: LiveConfigurationError) -> Self {
        Self::Configuration(error)
    }
}

impl From<ProbeFailure> for LiveSmokeError {
    fn from(error: ProbeFailure) -> Self {
        Self::Probe(error)
    }
}

#[test]
#[ignore = "requires explicit opt-in and a private read-only Hetzner token file"]
fn read_only_catalog_smoke() -> Result<(), LiveSmokeError> {
    let token = load_read_only_token()?;
    let policy = official_endpoint_policy(ApiBaseUrl::CloudV1)
        .map_err(|_| LiveSmokeError::Endpoint(EndpointError::PolicyRejected))?;
    let endpoint = HttpsEndpoint::new_with_policy(CLOUD_API_BASE_URL, policy)
        .map_err(LiveSmokeError::Endpoint)?;
    let credential_scope =
        BearerCredentialScope::new(HETZNER_PROVIDER_ID, CLOUD_SERVICE_ID, endpoint.clone());
    let credential = BearerCredential::new(token, credential_scope);
    let user_agent =
        UserAgent::new("cloud-sdk-live-smoke/0.65.0").map_err(LiveSmokeError::UserAgent)?;
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(10))
        .map_err(LiveSmokeError::Timeout)?;
    let client = BlockingClientBuilder::new(endpoint, credential, user_agent, timeouts)
        .build()
        .map_err(LiveSmokeError::Client)?;

    for probe in PROBES {
        probe.run(&client)?;
        println!("live smoke: {} passed", probe.name());
    }
    Ok(())
}

#[test]
#[ignore = "requires explicit opt-in and a private read-only Hetzner token file"]
fn read_only_dns_model_smoke() -> Result<(), LiveSmokeError> {
    let token = load_read_only_token()?;
    let policy = official_endpoint_policy(ApiBaseUrl::CloudV1)
        .map_err(|_| LiveSmokeError::Endpoint(EndpointError::PolicyRejected))?;
    let endpoint = HttpsEndpoint::new_with_policy(CLOUD_API_BASE_URL, policy)
        .map_err(LiveSmokeError::Endpoint)?;
    let credential_scope =
        BearerCredentialScope::new(HETZNER_PROVIDER_ID, DNS_SERVICE_ID, endpoint.clone());
    let credential = BearerCredential::new(token, credential_scope);
    let user_agent =
        UserAgent::new("cloud-sdk-dns-live-smoke/0.65.0").map_err(LiveSmokeError::UserAgent)?;
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(10))
        .map_err(LiveSmokeError::Timeout)?;
    let client = BlockingClientBuilder::new(endpoint, credential, user_agent, timeouts)
        .build()
        .map_err(LiveSmokeError::Client)?;

    let page = Page::new(1).map_err(|_| LiveSmokeError::DnsProbe(DnsProbeStage::Pagination))?;
    let per_page =
        PerPage::new(1).map_err(|_| LiveSmokeError::DnsProbe(DnsProbeStage::Pagination))?;
    let query = ZoneListRequest::new().with_page(page, per_page);
    let operation = AssociatedOperation::<ListZones, _, _>::query(ZoneEndpoint::List, query)
        .map_err(|_| LiveSmokeError::DnsProbe(DnsProbeStage::Association))?;
    let mut target = [0_u8; 64];
    let mut request_body = [0_u8; 1];
    let prepared = operation
        .prepare_typed(PreparationStorage::new(&mut target, &mut request_body))
        .map_err(|_| LiveSmokeError::DnsProbe(DnsProbeStage::Preparation))?;
    let mut body = vec![0_u8; 1_048_576];
    let mut headers = [0_u8; 8_192];
    let response = prepared
        .execute_blocking(&client, &mut body, &mut headers)
        .map_err(|_| LiveSmokeError::DnsProbe(DnsProbeStage::Transport))?;
    let decoded = decode_associated_checked_response(response)
        .map_err(|_| LiveSmokeError::DnsProbe(DnsProbeStage::Decode))?;
    let HetznerSuccess::DnsResources { resources, .. } = decoded.success() else {
        return Err(LiveSmokeError::DnsProbe(DnsProbeStage::Shape));
    };
    if resources
        .iter()
        .any(|resource| !matches!(resource, DnsResource::Zone(_)))
    {
        return Err(LiveSmokeError::DnsProbe(DnsProbeStage::Shape));
    }
    Ok(())
}

#[test]
#[ignore = "requires explicit opt-in and a private read-only Hetzner token file"]
fn read_only_security_model_smoke() -> Result<(), LiveSmokeError> {
    let token = load_read_only_token()?;
    let policy = official_endpoint_policy(ApiBaseUrl::CloudV1)
        .map_err(|_| LiveSmokeError::Endpoint(EndpointError::PolicyRejected))?;
    let endpoint = HttpsEndpoint::new_with_policy(CLOUD_API_BASE_URL, policy)
        .map_err(LiveSmokeError::Endpoint)?;
    let credential_scope =
        BearerCredentialScope::new(HETZNER_PROVIDER_ID, SECURITY_SERVICE_ID, endpoint.clone());
    let credential = BearerCredential::new(token, credential_scope);
    let user_agent = UserAgent::new("cloud-sdk-security-live-smoke/0.66.0")
        .map_err(LiveSmokeError::UserAgent)?;
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(10))
        .map_err(LiveSmokeError::Timeout)?;
    let client = BlockingClientBuilder::new(endpoint, credential, user_agent, timeouts)
        .build()
        .map_err(LiveSmokeError::Client)?;
    let page =
        Page::new(1).map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Pagination))?;
    let per_page = PerPage::new(1)
        .map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Pagination))?;

    macro_rules! run_probe {
        ($operation:expr, $expected:expr) => {{
            let mut target = [0_u8; 128];
            let mut request_body = [0_u8; 1];
            let prepared = $operation
                .prepare_typed(PreparationStorage::new(&mut target, &mut request_body))
                .map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Preparation))?;
            let mut body = vec![0_u8; 8_388_608];
            let mut headers = [0_u8; 8_192];
            let response = prepared
                .execute_blocking(&client, &mut body, &mut headers)
                .map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Transport))?;
            let decoded = decode_associated_checked_response(response)
                .map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Decode))?;
            let HetznerSuccess::SecurityResources { resources, .. } = decoded.success() else {
                return Err(LiveSmokeError::SecurityProbe(SecurityProbeStage::Shape));
            };
            if resources
                .iter()
                .any(|resource| resource.kind() != $expected)
            {
                return Err(LiveSmokeError::SecurityProbe(SecurityProbeStage::Shape));
            }
        }};
    }

    let certificate_query = CertificateListRequest::new()
        .with_page(page)
        .with_per_page(per_page);
    let certificate = AssociatedOperation::<ListCertificates, _, _>::query(
        CertificateEndpoint::List,
        certificate_query,
    )
    .map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Association))?;
    run_probe!(certificate, SecurityResourceKind::Certificate);

    let ssh_query = SshKeyListRequest::new()
        .with_page(page)
        .with_per_page(per_page);
    let ssh = AssociatedOperation::<ListSshKeys, _, _>::query(SshKeyEndpoint::List, ssh_query)
        .map_err(|_| LiveSmokeError::SecurityProbe(SecurityProbeStage::Association))?;
    run_probe!(ssh, SecurityResourceKind::SshKey);
    Ok(())
}

#[test]
fn live_error_diagnostics_do_not_contain_secrets_or_resource_ids() {
    let error = LiveSmokeError::Configuration(LiveConfigurationError::TokenRejected);
    let diagnostic = format!("{error:?}");
    assert!(!diagnostic.contains("secret-token"));
    assert!(!diagnostic.contains("12345678"));
}
