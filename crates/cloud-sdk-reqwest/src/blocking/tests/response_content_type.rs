use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, TransportRequest};

use super::build_loopback;
use crate::shared::TransportError;
use crate::test_server::spawn;

#[test]
fn malformed_or_duplicate_response_content_type_fails_closed() {
    let Ok(target) = RequestTarget::new("/servers") else {
        return;
    };
    for headers in [
        &[("Content-Type", "application/json; charset")][..],
        &[
            ("Content-Type", "application/json"),
            ("Content-Type", "text/plain"),
        ][..],
    ] {
        let server = spawn("200 OK", headers, b"secret", Duration::ZERO);
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let mut output = [0xa5_u8; 8];
        assert_eq!(
            client.send(TransportRequest::new(Method::Get, target), &mut output),
            Err(TransportError::InvalidResponseContentType)
        );
        assert_eq!(output, [0_u8; 8]);
    }
}
