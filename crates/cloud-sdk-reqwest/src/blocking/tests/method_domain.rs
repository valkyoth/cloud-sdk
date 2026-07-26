use std::string::String;
use std::time::Duration;

use cloud_sdk::Method;
use cloud_sdk::transport::{BlockingTransport, RequestTarget, TransportRequest};

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
        let server = spawn("200 OK", &[], b"", Duration::ZERO);
        let Ok(server) = server else { return };
        let Some(client) = build_loopback(&server.endpoint) else {
            return;
        };
        let Ok(target) = RequestTarget::new("/method-check") else {
            return;
        };
        let mut output = [0_u8; 1];
        let response = client.send(TransportRequest::new(method, target), &mut output);
        assert!(response.is_ok());

        let recorded = server.request.recv_timeout(Duration::from_secs(2));
        assert!(recorded.is_ok());
        if let Ok(recorded) = recorded {
            let wire = String::from_utf8_lossy(&recorded.bytes);
            assert!(wire.starts_with(method.as_str()));
            assert!(wire[method.as_str().len()..].starts_with(" /v1/method-check HTTP/1.1\r\n"));
        }
    }
}
