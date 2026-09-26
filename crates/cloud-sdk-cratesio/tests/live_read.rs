//! Explicit operator-only live reads. Normal workspace tests perform no I/O.
#[path = "live_read/config.rs"]
mod config;
#[path = "live_read/dispatch_tests.rs"]
mod dispatch_tests;
#[path = "live_read/guard.rs"]
mod guard;
#[path = "live_read/tests.rs"]
mod tests;

use cloud_sdk_cratesio::{
    bundled::{RequestTimeouts, production_blocking},
    catalog::CatalogRequest,
    client::{RegistryBuffers, RegistryClient},
    discovery::DiscoveryRequest,
    query::{Parameter, PerPage},
};
use std::{io::IsTerminal, time::Duration};

#[test]
#[ignore = "operator opt-in; fixed read-only production endpoints; never in CI"]
fn live_read() -> Result<(), &'static str> {
    let selected = std::env::var("CLOUD_SDK_CRATESIO_LIVE").map_err(|_| "live opt-in required")?;
    let ack = std::env::var("CLOUD_SDK_CRATESIO_TOKEN_ACK").ok();
    let ci = [
        "CI",
        "GITHUB_ACTIONS",
        "GITLAB_CI",
        "BUILD_BUILDID",
        "JENKINS_URL",
    ]
    .iter()
    .any(|key| std::env::var_os(key).is_some());
    let mode = config::mode(&selected, ci, ack.as_deref())?;
    let agent = std::env::var("CLOUD_SDK_CRATESIO_USER_AGENT")
        .map_err(|_| "identifying user agent required")?;
    let identity = config::identity(&agent)?;
    let token = if mode == config::Mode::Token {
        if std::io::stdin().is_terminal() {
            return Err("credential must arrive through redirected stdin");
        }
        Some(config::read_token(&mut std::io::stdin().lock())?)
    } else {
        None
    };
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(5))
        .map_err(|_| "timeout policy rejected")?;
    let transport = guard::ReadOnly(
        production_blocking(identity, timeouts).map_err(|_| "transport construction failed")?,
    );
    let client = RegistryClient::production(&transport, identity, 262_144)
        .map_err(|_| "client construction failed")?;
    let mut credential = [0; 2048];
    let mut response = vec![0; 262_144];
    let mut headers = [0; 2048];
    client
        .execute(
            DiscoveryRequest::site_metadata(),
            RegistryBuffers {
                credential: &mut credential,
                body: &mut [],
                response: &mut response,
                headers: &mut headers,
            },
        )
        .map_err(|_| "metadata live read failed")?;
    // Operator harness scheduling, not an SDK retry. The shared SDK gate still
    // enforces provider delays and stops this run rather than retrying a failure.
    std::thread::sleep(Duration::from_secs(1));
    let page = PerPage::new(1).map_err(|_| "page policy rejected")?;
    let buffers = RegistryBuffers {
        credential: &mut credential,
        body: &mut [],
        response: &mut response,
        headers: &mut headers,
    };
    match token {
        Some(token) => {
            let parameters = [Parameter::Following, Parameter::PerPage(page)];
            let request =
                CatalogRequest::list(&parameters).map_err(|_| "following request rejected")?;
            client
                .catalog_with_token(request, &token, buffers)
                .map_err(|error| {
                    use cloud_sdk_cratesio::{
                        discovery::DiscoveryExecutionError as E, wire::CratesIoWireError,
                    };
                    match error {
                        E::Schedule(_) => "following read not admitted by scheduler",
                        E::Transport(_) => "following read transport failed",
                        E::Model(_) => "following read model validation failed",
                        E::Staging => "following read staging failed",
                        E::Wire(CratesIoWireError::Provider(provider)) => {
                            eprintln!(
                                "following read provider status: {}",
                                provider.status().get()
                            );
                            "following read rejected by provider"
                        }
                        E::Wire(_) => "following read wire validation failed",
                    }
                })?;
        }
        None => {
            let parameters = [Parameter::PerPage(page)];
            let request =
                DiscoveryRequest::keywords(&parameters).map_err(|_| "keyword request rejected")?;
            client
                .execute(request, buffers)
                .map_err(|_| "keyword live read failed")?;
        }
    }
    assert!(
        credential
            .iter()
            .chain(&response)
            .chain(&headers)
            .all(|byte| *byte == 0)
    );
    Ok(())
}
