use std::format;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::string::String;
use std::sync::mpsc::{Receiver, channel};
use std::thread;
use std::time::Duration;
use std::vec::Vec;

pub(super) struct RecordedRequest {
    pub(super) bytes: Vec<u8>,
}

pub(super) struct TestServer {
    pub(super) endpoint: String,
    pub(super) request: Receiver<RecordedRequest>,
}

pub(super) fn spawn(
    status: &str,
    headers: &[(&str, &str)],
    body: &[u8],
    response_delay: Duration,
) -> Result<TestServer, std::io::Error> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let endpoint = format!("http://{address}/v1");
    let (sender, request) = channel();
    let response = response_bytes(status, headers, body)?;

    thread::spawn(move || {
        let Ok((mut stream, _)) = listener.accept() else {
            return;
        };
        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
        let Ok(bytes) = read_request(&mut stream) else {
            return;
        };
        let _ = sender.send(RecordedRequest { bytes });
        if !response_delay.is_zero() {
            thread::sleep(response_delay);
        }
        let _ = stream.write_all(&response);
        let _ = stream.flush();
    });

    Ok(TestServer { endpoint, request })
}

fn response_bytes(
    status: &str,
    headers: &[(&str, &str)],
    body: &[u8],
) -> Result<Vec<u8>, std::io::Error> {
    let mut response = Vec::new();
    let head = format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\n", body.len());
    response
        .try_reserve(head.len())
        .map_err(|_| std::io::Error::other("response allocation failed"))?;
    response.extend_from_slice(head.as_bytes());
    for (name, value) in headers {
        response.extend_from_slice(name.as_bytes());
        response.extend_from_slice(b": ");
        response.extend_from_slice(value.as_bytes());
        response.extend_from_slice(b"\r\n");
    }
    response.extend_from_slice(b"Connection: close\r\n\r\n");
    response.extend_from_slice(body);
    Ok(response)
}

fn read_request(stream: &mut impl Read) -> Result<Vec<u8>, std::io::Error> {
    let mut request = Vec::new();
    let mut scratch = [0_u8; 1024];
    let mut body_start = None;
    let mut expected = None;
    loop {
        let read = stream.read(&mut scratch)?;
        if read == 0 {
            break;
        }
        let chunk = scratch.get(..read).unwrap_or_default();
        request.extend_from_slice(chunk);
        if body_start.is_none() {
            body_start = request
                .windows(4)
                .position(|window| window == b"\r\n\r\n")
                .and_then(|position| position.checked_add(4));
            if let Some(start) = body_start {
                expected = content_length(request.get(..start).unwrap_or_default())
                    .and_then(|length| start.checked_add(length));
            }
        }
        if expected.is_some_and(|length| request.len() >= length) {
            break;
        }
    }
    Ok(request)
}

fn content_length(headers: &[u8]) -> Option<usize> {
    let text = core::str::from_utf8(headers).ok()?;
    for line in text.split("\r\n") {
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("content-length") {
            return value.trim().parse().ok();
        }
    }
    Some(0)
}
