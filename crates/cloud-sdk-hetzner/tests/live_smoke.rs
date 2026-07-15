//! Opt-in live smoke coverage for the public Hetzner Cloud catalog.

use core::fmt;

#[path = "live_smoke/catalog.rs"]
mod catalog;
#[path = "live_smoke/config.rs"]
mod config;

use std::time::Duration;

use cloud_sdk_hetzner::request::CLOUD_API_BASE_URL;
use cloud_sdk_reqwest::blocking::{
    BlockingClientBuilder, BuildError, EndpointError, HttpsEndpoint, RequestTimeouts, TimeoutError,
    UserAgent, UserAgentError,
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
    let endpoint =
        HttpsEndpoint::new_custom(CLOUD_API_BASE_URL).map_err(LiveSmokeError::Endpoint)?;
    let user_agent =
        UserAgent::new("cloud-sdk-live-smoke/0.19.0").map_err(LiveSmokeError::UserAgent)?;
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(10))
        .map_err(LiveSmokeError::Timeout)?;
    let mut client = BlockingClientBuilder::new(endpoint, token, user_agent, timeouts)
        .build()
        .map_err(LiveSmokeError::Client)?;

    for probe in PROBES {
        probe.run(&mut client)?;
        println!("live smoke: {} passed", probe.name());
    }
    Ok(())
}

#[test]
fn live_error_diagnostics_do_not_contain_secrets_or_resource_ids() {
    let error = LiveSmokeError::Configuration(LiveConfigurationError::TokenRejected);
    let diagnostic = format!("{error:?}");
    assert!(!diagnostic.contains("secret-token"));
    assert!(!diagnostic.contains("12345678"));
}
