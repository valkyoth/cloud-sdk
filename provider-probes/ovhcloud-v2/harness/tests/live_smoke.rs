//! Explicitly enabled, ignored, read-only OVHcloud execution smoke.

#![cfg(feature = "live-smoke")]

#[path = "live/config.rs"]
mod config;
mod support;

use std::error::Error;
use std::time::Duration;

use cloud_sdk::ProviderMarker;
use cloud_sdk::ServiceMarker;
use cloud_sdk::transport::EndpointPolicy;
use cloud_sdk_reqwest::blocking::{
    BearerCredential, BearerCredentialScope, BlockingClientBuilder, HttpsEndpoint, RequestTimeouts,
    UserAgent,
};

use support::{ApiV2, OPERATIONS, OvhcloudProbe, prepared, request_headers};

#[test]
#[ignore = "requires explicit read-only mode and a least-privilege OVHcloud token file"]
fn least_privilege_policy_collection_smoke() -> Result<(), Box<dyn Error>> {
    let operation = OPERATIONS
        .iter()
        .find(|operation| operation.id == "iam/policy")
        .ok_or("source-locked live operation is unavailable")?;
    let identity = support::endpoint();
    let endpoint = HttpsEndpoint::new_with_policy(
        "https://eu.api.ovh.com/v2",
        EndpointPolicy::fixed(identity),
    )?;
    let token = config::load_read_only_token()?;
    let scope = BearerCredentialScope::new(OvhcloudProbe::ID, ApiV2::ID, endpoint.clone());
    let credential = BearerCredential::new(token, scope);
    let user_agent = UserAgent::new("cloud-sdk-ovhcloud-probe/0.61")?;
    let timeouts = RequestTimeouts::new(Duration::from_secs(30), Duration::from_secs(10))?;
    let client = BlockingClientBuilder::new(endpoint, credential, user_agent, timeouts).build()?;
    let (headers, count) = request_headers(operation.paginated);
    let entries = headers
        .get(..count)
        .ok_or("pagination header count exceeds fixture storage")?;
    let request = prepared(*operation, entries);
    let mut body = [0_u8; 65_536];
    let mut response_headers = [0_u8; 8192];
    let response = request.execute_blocking(&client, &mut body, &mut response_headers)?;
    if !response.with_borrowed(|checked| !checked.body().is_empty()) {
        return Err("OVHcloud returned an empty successful response".into());
    }
    Ok(())
}
