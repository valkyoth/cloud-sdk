use super::*;

#[test]
fn upload_preflight_rejects_incoherent_requests_without_observing_source() {
    run_async_test(async {
        let client = build_raw_loopback("http://127.0.0.1:1/v1").unwrap_or_else(|| unreachable!());
        for case in 0..8 {
            let mut source = Source {
                bytes: b"abc",
                reads: 0,
                pending: false,
            };
            let mut scratch = [0xa5; 2];
            let mut body = [0xa5; 16];
            let mut headers = [0xa5; 128];
            let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
            let value = std::format!("fixture-{}", std::process::id());
            let request_headers = [
                RequestHeader::new("content-type", "application/octet-stream")
                    .unwrap_or_else(|_| unreachable!()),
            ];
            let mut request = request(if case == 0 { &[] } else { &request_headers });
            if case == 1 {
                request = request.with_body(b"must not combine bodies");
            }
            if case == 2 || case == 3 {
                request = TransportRequest::new(
                    if case == 2 { Method::Get } else { Method::Head },
                    request.target(),
                )
                .with_headers(request.headers());
            }
            let selected = StreamPolicy::new(
                if case == 4 {
                    StreamKind::FiniteDownload
                } else {
                    StreamKind::FiniteUpload
                },
                if case == 5 {
                    StreamFraming::ExecutorOwned
                } else {
                    StreamFraming::Declared(if case == 6 { 0 } else { 3 })
                },
                if case == 7 {
                    StreamSinkMode::Transactional
                } else {
                    StreamSinkMode::Direct
                },
                stream_policy(3).limits(),
            )
            .unwrap_or_else(|_| unreachable!());
            let upload = RawUpload::new(
                client
                    .endpoint_identity()
                    .unwrap_or_else(|_| unreachable!()),
                HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
                &mut source,
                selected,
                &mut scratch,
            );
            let result = client
                .execute_upload(
                    request,
                    super::super::policy(2).unwrap_or_else(|| unreachable!()),
                    upload,
                    response.writer(),
                )
                .await;
            assert_eq!(
                result,
                Err(cloud_sdk::transport::TransportFailure::not_sent(
                    if case == 0 {
                        RawHttpError::MissingContentType
                    } else {
                        RawHttpError::InvalidStreamState
                    }
                ))
            );
            assert_eq!(source.reads, 0);
            assert_eq!(scratch, [0; 2]);
        }
    });
}

#[test]
fn uploaded_responses_still_enforce_media_size_trailer_and_duplicate_rules() {
    run_async_test(async {
        for wire in [
            b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}".as_slice(),
            b"HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 20\r\n\r\n01234567890123456789",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nX-Test: one\r\nX-Test: two\r\nContent-Length: 2\r\n\r\n{}",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n{}\r\n0\r\nX-Test: one\r\n\r\n",
            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 3\r\n\r\n{}",
        ] {
            let server = spawn_raw_response(wire).unwrap_or_else(|e| unreachable!("server: {e}"));
            let client = build_raw_loopback(&server.endpoint).unwrap_or_else(|| unreachable!());
            let mut source = Source { bytes: b"abc", reads: 0, pending: false };
            let mut scratch = [0xa5; 2];
            let mut body = [0xa5; 16];
            let mut headers = [0xa5; 128];
            let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
            let value = std::format!("fixture-{}", std::process::id());
            let request_headers = [RequestHeader::new("content-type", "application/octet-stream").unwrap_or_else(|_| unreachable!())];
            let upload = RawUpload::new(client.endpoint_identity().unwrap_or_else(|_| unreachable!()), HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()), &mut source, stream_policy(3), &mut scratch);
            assert!(client.execute_upload(request(&request_headers), super::super::policy(2).unwrap_or_else(|| unreachable!()), upload, response.writer()).await.is_err());
            assert_eq!(scratch, [0; 2]);
            assert!(response.writer().begin_attempt().is_ok());
            server.request.recv_timeout(Duration::from_secs(2)).unwrap_or_else(|e| unreachable!("request: {e}"));
            assert!(source.bytes.is_empty());
        }
    });
}
