use super::*;
use crate::blocking::{HttpsEndpoint, RawBlockingClientBuilder, RequestTimeouts, UserAgent};
use cloud_sdk::Method;
use cloud_sdk::transport::*;

struct Source(bool);
impl BlockingStreamSource for Source {
    type Error = ();
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    fn read_chunk(&mut self, output: &mut [u8]) -> Result<StreamRead, ()> {
        if self.0 {
            self.0 = false;
            *output.first_mut().ok_or(())? = 1;
            Ok(StreamRead::Chunk(1))
        } else {
            Ok(StreamRead::End)
        }
    }
}

#[test]
fn all_raw_blocking_paths_return_while_dns_is_still_blocked() {
    for mode in 0..4 {
        let (release, blocked) = mpsc::channel();
        let (started, ready) = mpsc::channel();
        let (finished, completion) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let blocked = std::sync::Mutex::new(blocked);
            FIXTURE.with(|slot| {
                *slot.borrow_mut() = Some(std::sync::Arc::new(move || {
                    started
                        .send(())
                        .unwrap_or_else(|e| unreachable!("start: {e}"));
                    blocked
                        .lock()
                        .unwrap_or_else(|_| unreachable!())
                        .recv_timeout(Duration::from_secs(5))
                        .unwrap_or_else(|e| unreachable!("release: {e}"));
                    Err(io::Error::other("controlled DNS failure"))
                }));
            });
            let endpoint = HttpsEndpoint::new_custom(
                "https://dns-stall.invalid/",
                crate::blocking::CustomEndpointAcknowledgement::trusted_operator_configuration(),
            )
            .unwrap_or_else(|e| unreachable!("endpoint: {e}"));
            let timeouts =
                RequestTimeouts::new(Duration::from_millis(100), Duration::from_millis(100))
                    .unwrap_or_else(|e| unreachable!("timeout: {e}"));
            let client = RawBlockingClientBuilder::new(
                endpoint,
                UserAgent::new("cloud-sdk-dns-test/1").unwrap_or_else(|_| unreachable!()),
                timeouts,
            )
            .build()
            .unwrap_or_else(|e| unreachable!("client: {e}"));
            let target = RequestTarget::new("/dns-stall").unwrap_or_else(|_| unreachable!());
            let request = TransportRequest::new(Method::Get, target);
            let mut body = [0xa5; 16];
            let mut headers = [0xa5; 128];
            let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
            let policy = RawResponsePolicy::new(
                16,
                4,
                ResponseMediaPolicy::Optional(&[MediaType::JSON]),
                ResponseMediaPolicy::Optional(&[MediaType::JSON]),
                &[],
                2,
            )
            .unwrap_or_else(|_| unreachable!());
            let token = std::format!("fixture-{}", std::process::id());
            let authorization = HeaderValue::new(&token).unwrap_or_else(|_| unreachable!());
            let expected = client
                .endpoint_identity()
                .unwrap_or_else(|_| unreachable!());
            let result = match mode {
                0 => client.open_stream(request, 16).map(|_| ()),
                1 => client.execute(request, policy, response.writer()),
                2 => client.execute_authorized(
                    expected,
                    authorization,
                    request,
                    policy,
                    response.writer(),
                ),
                _ => {
                    let mut source = Source(true);
                    let mut scratch = [0; 8];
                    let stream_policy = StreamPolicy::new(
                        StreamKind::FiniteUpload,
                        StreamFraming::Declared(1),
                        StreamSinkMode::Direct,
                        StreamLimits::new(8, 8, 8, 32, 2).unwrap_or_else(|_| unreachable!()),
                    )
                    .unwrap_or_else(|_| unreachable!());
                    let request_headers =
                        [
                            RequestHeader::new("content-type", "application/octet-stream")
                                .unwrap_or_else(|_| unreachable!()),
                        ];
                    let request = TransportRequest::new(Method::Put, target).with_headers(
                        RequestHeaders::new(&request_headers).unwrap_or_else(|_| unreachable!()),
                    );
                    client.upload(
                        request,
                        policy,
                        AuthorizedUpload {
                            expected,
                            authorization,
                            source: &mut source,
                            policy: stream_policy,
                            scratch: &mut scratch,
                        },
                        response.writer(),
                    )
                }
            };
            FIXTURE.with(|slot| *slot.borrow_mut() = None);
            finished
                .send(result)
                .unwrap_or_else(|_| unreachable!("complete"));
        });
        let started = ready.recv_timeout(Duration::from_secs(3));
        let result = completion.recv_timeout(Duration::from_secs(2));
        let _ = release.send(());
        assert!(worker.join().is_ok());
        assert!(started.is_ok(), "mode {mode} did not reach DNS");
        let error = result.unwrap_or_else(|e| unreachable!("mode {mode} blocked: {e}"));
        assert!(
            error.is_err_and(|e| matches!(e.error(), crate::blocking::RawHttpError::TimedOut)),
            "mode {mode}"
        );
    }
}
