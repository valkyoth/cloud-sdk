//! Decode a synthetic response through its exact prepared operation policy.

use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{ResponseContentType, StatusCode, TransportResponse};
use cloud_sdk_hetzner::cloud::servers::{ServerEndpoint, ServerId};
use cloud_sdk_hetzner::serde::{HetznerSuccess, decode_response};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let id = ServerId::new(42).ok_or("invalid server ID")?;
    let operation = ServerEndpoint::Get(id);
    let mut target = [0_u8; 64];
    let mut body = [];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body))?;

    let response_body = br#"{"server":{"id":42,"name":"web-1","status":"running"}}"#;
    let response = TransportResponse::new(StatusCode::OK, response_body)
        .with_content_type(ResponseContentType::new("application/json")?);
    let decoded = decode_response(prepared, response)?;

    let HetznerSuccess::Resource(server) = decoded.success() else {
        return Err("unexpected response family".into());
    };
    assert_eq!(server.name(), Some("web-1"));
    Ok(())
}
