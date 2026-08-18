use std::time::Duration;

use cloud_sdk::authentication::{BlockingAuthenticatedTransport, BoundCredentialTransport};
use cloud_sdk::client::{ClientWorkspace, ClientWorkspacePool};
use cloud_sdk::transport::{BoundTransport, DeliveryClassified};
use cloud_sdk_hetzner::client::{RobotClient, RobotClientResponse};
use cloud_sdk_hetzner::robot::RobotServerListRequest;
use cloud_sdk_hetzner::{ROBOT_API_BASE_URL, official_robot_endpoint_policy};
use cloud_sdk_reqwest::blocking::{
    BlockingBasicClientBuilder, HttpsEndpoint, RequestTimeouts, UserAgent,
};

use super::robot_config::{RobotLiveConfigurationError, load_read_only_robot_credential};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RobotLiveSmokeError {
    Configuration(RobotLiveConfigurationError),
    Endpoint,
    UserAgent,
    Timeout,
    Transport,
    Client,
    WorkspacePool,
    WorkspaceLease,
    Execution,
    ProviderFailure,
}

pub(super) fn run_read_only_server_probe() -> Result<(), RobotLiveSmokeError> {
    let policy = official_robot_endpoint_policy().map_err(|_| RobotLiveSmokeError::Endpoint)?;
    let endpoint = HttpsEndpoint::new_with_policy(ROBOT_API_BASE_URL, policy)
        .map_err(|_| RobotLiveSmokeError::Endpoint)?;
    let credential = load_read_only_robot_credential(endpoint.clone())
        .map_err(RobotLiveSmokeError::Configuration)?;
    let user_agent = UserAgent::new("cloud-sdk-robot-live-smoke/0.95.0")
        .map_err(|_| RobotLiveSmokeError::UserAgent)?;
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(10))
        .map_err(|_| RobotLiveSmokeError::Timeout)?;
    let transport = BlockingBasicClientBuilder::new(endpoint, credential, user_agent, timeouts)
        .build()
        .map_err(|_| RobotLiveSmokeError::Transport)?;
    let client = RobotClient::official(transport).map_err(|_| RobotLiveSmokeError::Client)?;

    run_server_probe_with_client(&client)
}

fn run_server_probe_with_client<T>(client: &RobotClient<T>) -> Result<(), RobotLiveSmokeError>
where
    T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
    T::Error: DeliveryClassified,
{
    let request = RobotServerListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let mut response_body = vec![0_u8; 8_388_608];
    let mut response_headers = [0_u8; 8_192];
    let pool = ClientWorkspacePool::<1>::new().map_err(|_| RobotLiveSmokeError::WorkspacePool)?;
    let lease = pool
        .try_acquire(ClientWorkspace::new(
            &mut target,
            &mut request_body,
            response_body.as_mut_slice(),
            &mut response_headers,
        ))
        .map_err(|_| RobotLiveSmokeError::WorkspaceLease)?;
    match client
        .execute_blocking(&request, lease)
        .map_err(|_| RobotLiveSmokeError::Execution)?
    {
        RobotClientResponse::Success(_) => Ok(()),
        RobotClientResponse::Failure(_) => Err(RobotLiveSmokeError::ProviderFailure),
    }
}

#[cfg(test)]
mod tests {
    use cloud_sdk::Method;
    use cloud_sdk::transport::{
        EndpointIdentity, EndpointScheme, MediaType, RequestHeader, RequestHeaders, RequestTarget,
    };
    use cloud_sdk_testkit::{
        ExpectedRequest, FixtureBody, MockExchange, MockTransport, ResponseFixture,
    };

    use super::{RobotClient, run_server_probe_with_client};

    #[test]
    fn robot_live_probe_has_exact_read_only_wire_contract() {
        let target = RequestTarget::new("/server")
            .unwrap_or_else(|_| unreachable!("fixed Robot target became invalid"));
        let headers = [RequestHeader::accept(MediaType::JSON)];
        let headers = RequestHeaders::new(&headers)
            .unwrap_or_else(|_| unreachable!("fixed Robot headers became invalid"));
        let expected = ExpectedRequest::new(Method::Get, target).with_headers(headers);
        let body = FixtureBody::new(b"[]")
            .unwrap_or_else(|_| unreachable!("fixed Robot response became invalid"));
        let response = ResponseFixture::success(body).with_content_type("application/json");
        let exchanges = [MockExchange::new(expected, response)];
        let endpoint =
            EndpointIdentity::new(EndpointScheme::Https, "robot-ws.your-server.de", 443, "/")
                .unwrap_or_else(|_| unreachable!("official Robot endpoint became invalid"));
        let client = RobotClient::official(MockTransport::new(&exchanges).with_endpoint(endpoint))
            .unwrap_or_else(|_| unreachable!("official Robot client construction failed"));

        assert_eq!(run_server_probe_with_client(&client), Ok(()));
        assert!(client.transport().is_complete());
    }
}
