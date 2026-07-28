use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{RequestTarget, TransportRequest};

use crate::test_server::spawn;

use super::build_loopback;

#[test]
fn blocking_client_sends_complete_method_domain_exactly() {
    for method in [
        Method::Patch,
        Method::Head,
        Method::Options,
        Method::extension("PURGE").unwrap_or_else(|_| unreachable!()),
    ] {
        let server = spawn("200 OK", &[], b"", Duration::ZERO).ok();
        assert!(server.is_some(), "method-domain loopback server must start");
        let Some(server) = server else {
            unreachable!("successful server assertion guarantees a loopback server")
        };
        let client = build_loopback(&server.endpoint);
        assert!(client.is_some(), "method-domain loopback client must build");
        let Some(client) = client else {
            unreachable!("successful client assertion guarantees a loopback client")
        };
        let target = RequestTarget::new("/method-check");
        assert!(target.is_ok(), "static method-domain target must be valid");
        let Ok(target) = target else {
            unreachable!("successful target assertion guarantees a request target")
        };
        let mut output = [0_u8; 1];
        let response =
            super::send_test(&client, TransportRequest::new(method, target), &mut output);
        assert!(response.is_ok());

        let recorded = server.request.recv_timeout(Duration::from_secs(2));
        assert!(recorded.is_ok());
        let Ok(recorded) = recorded else {
            unreachable!("successful receive assertion guarantees a recorded request")
        };
        let wire = String::from_utf8_lossy(&recorded.bytes);
        assert!(wire.starts_with(method.as_str()));
        assert!(wire[method.as_str().len()..].starts_with(" /v1/method-check HTTP/1.1\r\n"));
    }
}
