use super::*;
use cloud_sdk::transport::{EndpointIdentity, EndpointScheme};

fn listener() -> (std::net::TcpListener, std::string::String) {
    let listener =
        std::net::TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| unreachable!("bind: {e}"));
    listener
        .set_nonblocking(true)
        .unwrap_or_else(|e| unreachable!("nonblocking: {e}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|e| unreachable!("address: {e}"));
    (listener, std::format!("http://{address}/v1"))
}

#[test]
fn upload_rejects_wrong_destination_without_reading_and_clears_storage() {
    run_async_test(async {
        let (listener, endpoint) = listener();
        let client = build_raw_loopback(&endpoint).unwrap_or_else(|| unreachable!());
        let mut source = Source {
            bytes: b"abc",
            reads: 0,
            pending: false,
        };
        let mut scratch = [0xa5; 2];
        let mut body = [0xa5; 16];
        let mut headers = [0xa5; 128];
        let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
        let expected = EndpointIdentity::new(EndpointScheme::Https, "wrong.invalid", 443, "/")
            .unwrap_or_else(|_| unreachable!());
        let value = std::format!("fixture-{}", std::process::id());
        let upload = RawUpload::new(
            expected,
            HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
            &mut source,
            stream_policy(3),
            &mut scratch,
        );
        let result = client
            .execute_upload(
                request(&[]),
                policy(2).unwrap_or_else(|| unreachable!()),
                upload,
                response.writer(),
            )
            .await;
        assert_eq!(
            result,
            Err(cloud_sdk::transport::TransportFailure::not_sent(
                RawHttpError::TargetRejected
            ))
        );
        assert_eq!(source.reads, 0);
        assert_eq!(scratch, [0; 2]);
        assert!(
            listener
                .accept()
                .is_err_and(|e| e.kind() == std::io::ErrorKind::WouldBlock)
        );
        drop(response);
        assert_eq!(body, [0; 16]);
        assert_eq!(headers, [0; 128]);
    });
}

#[test]
fn cancelled_and_unpolled_uploads_clear_scratch_and_response() {
    run_async_test(async {
        for local in [false, true] {
            for polled in [false, true] {
                let (_listener, endpoint) = listener();
                let client = build_raw_loopback(&endpoint).unwrap_or_else(|| unreachable!());
                let mut source = Source {
                    bytes: b"abc",
                    reads: 0,
                    pending: true,
                };
                let mut scratch = [0xa5; 2];
                let mut body = [0xa5; 16];
                let mut headers = [0xa5; 128];
                let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
                let value = std::format!("fixture-{}", std::process::id());
                let request_headers =
                    [
                        RequestHeader::new("content-type", "application/octet-stream")
                            .unwrap_or_else(|_| unreachable!()),
                    ];
                let upload = RawUpload::new(
                    client
                        .endpoint_identity()
                        .unwrap_or_else(|_| unreachable!()),
                    HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
                    &mut source,
                    stream_policy(3),
                    &mut scratch,
                );
                if local {
                    cancel(
                        client.execute_upload_local(
                            request(&request_headers),
                            policy(2).unwrap_or_else(|| unreachable!()),
                            upload,
                            response.writer(),
                        ),
                        polled,
                    )
                    .await;
                } else {
                    cancel(
                        send(client.execute_upload(
                            request(&request_headers),
                            policy(2).unwrap_or_else(|| unreachable!()),
                            upload,
                            response.writer(),
                        )),
                        polled,
                    )
                    .await;
                }
                assert_eq!(source.reads, usize::from(polled));
                assert_eq!(scratch, [0; 2]);
                drop(response);
                assert_eq!(body, [0; 16]);
                assert_eq!(headers, [0; 128]);
            }
        }
    });
}
async fn cancel<F: Future>(future: F, polled: bool) {
    if polled {
        assert!(
            tokio::time::timeout(Duration::from_millis(20), future)
                .await
                .is_err()
        );
    } else {
        drop(future);
    }
}

#[test]
fn upload_rejects_short_long_and_incoherent_sources_without_committing() {
    run_async_test(async {
        for local in [false, true] {
            for bytes in [b"ab".as_slice(), b"abcd"] {
                let (_listener, endpoint) = listener();
                let client = build_raw_loopback(&endpoint).unwrap_or_else(|| unreachable!());
                let mut source = Source {
                    bytes,
                    reads: 0,
                    pending: false,
                };
                let mut scratch = [0xa5; 2];
                let mut body = [0xa5; 16];
                let mut headers = [0xa5; 128];
                let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
                let value = std::format!("fixture-{}", std::process::id());
                let request_headers =
                    [
                        RequestHeader::new("content-type", "application/octet-stream")
                            .unwrap_or_else(|_| unreachable!()),
                    ];
                let upload = RawUpload::new(
                    client
                        .endpoint_identity()
                        .unwrap_or_else(|_| unreachable!()),
                    HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
                    &mut source,
                    stream_policy(3),
                    &mut scratch,
                );
                let result = if local {
                    client
                        .execute_upload_local(
                            request(&request_headers),
                            policy(2).unwrap_or_else(|| unreachable!()),
                            upload,
                            response.writer(),
                        )
                        .await
                } else {
                    client
                        .execute_upload(
                            request(&request_headers),
                            policy(2).unwrap_or_else(|| unreachable!()),
                            upload,
                            response.writer(),
                        )
                        .await
                };
                assert_eq!(
                    result,
                    Err(cloud_sdk::transport::TransportFailure::possibly_sent(
                        RawHttpError::UploadFailed
                    ))
                );
                assert_eq!(scratch, [0; 2]);
                assert!(response.writer().begin_attempt().is_ok());
                drop(response);
                assert_eq!(body, [0; 16]);
                assert_eq!(headers, [0; 128]);
            }
        }
    });
}
