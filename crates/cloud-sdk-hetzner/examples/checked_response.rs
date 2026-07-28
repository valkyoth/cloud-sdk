//! Decode a synthetic response through its exact prepared operation policy.

use cloud_sdk::operation::{PreparationStorage, PrepareOperation};
use cloud_sdk::transport::{
    ResponseBuffer, ResponseContentType, ResponseMetadata, ResponseStorageSanitizer, StatusCode,
};
use cloud_sdk_hetzner::cloud::servers::{ServerEndpoint, ServerId};
use cloud_sdk_hetzner::serde::{HetznerSuccess, decode_response};

fn main() -> Result<(), Box<dyn core::error::Error>> {
    let id = ServerId::new(42).ok_or("invalid server ID")?;
    let operation = ServerEndpoint::Get(id);
    let mut target = [0_u8; 64];
    let mut body = [];
    let prepared = operation.prepare(PreparationStorage::new(&mut target, &mut body))?;

    let response_body = br#"{"server":{"id":42,"name":"web-1","status":"running"}}"#;
    let mut response_storage = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut response_storage, 128, &Sanitizer);
    let output = response
        .writer()
        .body_mut()?
        .get_mut(..response_body.len())
        .ok_or("response buffer is too small")?;
    output.copy_from_slice(response_body);
    response.writer().commit(
        StatusCode::OK,
        response_body.len(),
        ResponseMetadata::EMPTY.with_content_type(ResponseContentType::new("application/json")?),
    )?;
    let decoded = decode_response(prepared, response)?;

    let HetznerSuccess::Resource(server) = decoded.success() else {
        return Err("unexpected response family".into());
    };
    assert_eq!(server.name(), Some("web-1"));
    Ok(())
}

struct Sanitizer;

impl ResponseStorageSanitizer for Sanitizer {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        cloud_sdk_sanitization::sanitize_bytes(response_storage);
    }
}
