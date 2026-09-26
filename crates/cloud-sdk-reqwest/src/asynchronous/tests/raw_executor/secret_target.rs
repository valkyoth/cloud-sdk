use super::*;
use cloud_sdk::transport::{RequestTarget, drive_local_raw};

#[test]
fn secret_target_wire_is_exact_and_does_not_gain_authorization() {
    run_async_test(async {
        for local in [false, true] {
            for suffix in ["?", "?x=%2F%20%25"] {
                let server = spawn_raw_response(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
                ).unwrap_or_else(|error| unreachable!("loopback: {error}"));
                let client = build_raw_loopback(&server.endpoint)
                    .unwrap_or_else(|| unreachable!("client fixture"));
                let path = std::format!("/confirm/fixture-{}{suffix}", std::process::id());
                let target = RequestTarget::new(&path).unwrap_or_else(|_| unreachable!("target"));
                let mut body = [0xa5; 16];
                let mut headers = [0xa5; 128];
                let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
                let request = TransportRequest::new(Method::Put, target);
                assert!(!std::format!("{request:?}").contains(&path));
                let policy = policy(2).unwrap_or_else(|| unreachable!("policy"));
                if local {
                    assert!(
                        drive_local_raw(&client, request, policy, response.writer())
                            .await
                            .is_ok()
                    );
                } else {
                    assert!(
                        client
                            .execute_checked(request, policy, response.writer())
                            .await
                            .is_ok()
                    );
                }
                let wire = server
                    .request
                    .recv_timeout(Duration::from_secs(2))
                    .unwrap_or_else(|error| unreachable!("wire: {error}"));
                let wire = std::string::String::from_utf8(wire.bytes)
                    .unwrap_or_else(|_| unreachable!("ASCII wire"));
                assert!(wire.starts_with(&std::format!("PUT /v1{path} HTTP/1.1\r\n")));
                assert!(!wire.to_ascii_lowercase().contains("authorization:"));
                drop(response);
                assert_eq!(body, [0; 16]);
                assert_eq!(headers, [0; 128]);
            }
        }
    });
}
