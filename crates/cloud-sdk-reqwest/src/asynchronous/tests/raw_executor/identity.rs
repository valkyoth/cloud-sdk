use super::*;

#[test]
fn raw_async_streams_directly_into_the_caller_buffer() {
    run_async_test(async {
        let server = spawn(
            "200 OK",
            &[("Content-Type", "application/json")],
            b"{}",
            Duration::ZERO,
        );
        let Ok(server) = server else {
            unreachable!("security fixture construction failed")
        };
        let Some(client) = build_raw_loopback(&server.endpoint) else {
            unreachable!("security fixture construction failed");
        };
        let Ok(target) = cloud_sdk::transport::RequestTarget::new("/servers") else {
            unreachable!("security fixture construction failed");
        };
        let policy = policy(2).unwrap_or_else(|| unreachable!());
        let mut body = [0xa5_u8; 16];
        let mut header_storage = [0xa5_u8; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut header_storage);
        assert!(
            client
                .execute_checked(
                    TransportRequest::new(Method::Get, target),
                    policy,
                    response.writer(),
                )
                .await
                .is_ok()
        );
        assert!(
            response
                .with_response(|value| value.body() == b"{}")
                .unwrap_or(false)
        );
        assert_eq!(
            cloud_sdk::transport::BoundUserAgent::configured_user_agent(&client),
            b"cloud-sdk-raw-test/0.40"
        );
        let recorded = server
            .request
            .recv_timeout(Duration::from_secs(2))
            .unwrap_or_else(|_| unreachable!("loopback request"));
        assert!(
            std::string::String::from_utf8_lossy(&recorded.bytes)
                .to_ascii_lowercase()
                .contains("user-agent: cloud-sdk-raw-test/0.40\r\n")
        );
    });
}
