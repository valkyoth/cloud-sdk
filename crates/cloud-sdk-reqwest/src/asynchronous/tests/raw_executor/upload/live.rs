use super::*;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::Arc,
    thread,
};
use tokio::sync::Notify;

struct GatedSource {
    source: Source,
    gate: Arc<Notify>,
}
impl AsyncStreamSource for GatedSource {
    type Error = ();
    fn replayability(&self) -> StreamReplayability<'_> {
        StreamReplayability::NotReplayable
    }
    async fn read_chunk<'a>(&'a mut self, output: &'a mut [u8]) -> Result<StreamRead, ()> {
        if self.source.reads == 1 {
            self.gate.notified().await;
        }
        self.source.read_chunk(output).await
    }
}

fn server(mode: u8, gate: Arc<Notify>) -> (std::string::String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap_or_else(|e| unreachable!("bind: {e}"));
    let address = listener
        .local_addr()
        .unwrap_or_else(|e| unreachable!("address: {e}"));
    let worker = thread::spawn(move || {
        let (mut stream, _) = listener
            .accept()
            .unwrap_or_else(|e| unreachable!("accept: {e}"));
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap_or_else(|e| unreachable!("timeout: {e}"));
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .unwrap_or_else(|e| unreachable!("timeout: {e}"));
        let mut head = std::vec::Vec::new();
        while !head.ends_with(b"\r\n\r\n") {
            assert!(head.len() < 4096);
            let mut byte = [0];
            stream
                .read_exact(&mut byte)
                .unwrap_or_else(|e| unreachable!("head: {e}"));
            head.extend_from_slice(&byte);
        }
        let mut first = [0; 2];
        stream
            .read_exact(&mut first)
            .unwrap_or_else(|e| unreachable!("first chunk: {e}"));
        assert_eq!(&first, b"ab");
        match mode {
            0 => {
                // A preloading implementation deadlocks before this notification.
                gate.notify_one();
                let mut rest = [0; 3];
                stream
                    .read_exact(&mut rest)
                    .unwrap_or_else(|e| unreachable!("rest: {e}"));
                assert_eq!(&rest, b"cde");
                stream.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap_or_else(|e| unreachable!("reply: {e}"));
            }
            1 => {
                stream.write_all(b"HTTP/1.1 413 Content Too Large\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}").unwrap_or_else(|e| unreachable!("early reply: {e}"));
            }
            _ => thread::sleep(Duration::from_millis(500)),
        }
    });
    (std::format!("http://{address}/v1"), worker)
}

#[test]
fn upload_is_live_and_early_reply_or_deadline_cancels_the_borrowed_producer() {
    run_async_test(async {
        for mode in 0..3 {
            let gate = Arc::new(Notify::new());
            let (endpoint, worker) = server(mode, Arc::clone(&gate));
            let client = RawAsyncClientBuilder::new(
                crate::asynchronous::HttpsEndpoint::local_http(&endpoint)
                    .unwrap_or_else(|_| unreachable!()),
                UserAgent::new("upload-test/1").unwrap_or_else(|_| unreachable!()),
                crate::asynchronous::RequestTimeouts::new(
                    Duration::from_millis(300),
                    Duration::from_millis(100),
                )
                .unwrap_or_else(|_| unreachable!()),
            )
            .build_for_loopback()
            .unwrap_or_else(|_| unreachable!());
            let mut source = GatedSource {
                source: Source {
                    bytes: b"abcde",
                    reads: 0,
                    pending: false,
                },
                gate,
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
            let upload = RawUpload::new(
                client
                    .endpoint_identity()
                    .unwrap_or_else(|_| unreachable!()),
                HeaderValue::new(&value).unwrap_or_else(|_| unreachable!()),
                &mut source,
                stream_policy(5),
                &mut scratch,
            );
            let result = client
                .execute_upload(
                    request(&request_headers),
                    policy(2).unwrap_or_else(|| unreachable!()),
                    upload,
                    response.writer(),
                )
                .await;
            match mode {
                0 => assert_eq!(result, Ok(())),
                1 => assert_eq!(
                    result,
                    Err(cloud_sdk::transport::TransportFailure::response_started(
                        RawHttpError::UploadIncomplete
                    ))
                ),
                _ => assert_eq!(
                    result,
                    Err(cloud_sdk::transport::TransportFailure::possibly_sent(
                        RawHttpError::TimedOut
                    ))
                ),
            }
            assert_eq!(source.source.reads, if mode == 0 { 4 } else { 1 });
            assert_eq!(scratch, [0; 2]);
            drop(response);
            assert_eq!(body, [0; 16]);
            assert_eq!(headers, [0; 128]);
            worker
                .join()
                .unwrap_or_else(|_| unreachable!("server failed"));
        }
    });
}
