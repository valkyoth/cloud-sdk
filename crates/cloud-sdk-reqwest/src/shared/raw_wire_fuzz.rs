use std::sync::Arc;

use bytes::Bytes;
use cloud_sdk::Method;
use cloud_sdk::transport::{
    HeaderName, MediaType, RawResponsePolicy, ResponseBuffer, ResponseMediaPolicy, StatusCode,
};
use http_body_util::Empty;
use hyper::client::conn::http1;
use hyper_util::rt::TokioIo;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use super::raw_hyper::{ResponseState, read_bounded_body};
use super::{MAX_UPSTREAM_HTTP1_HEAD_BYTES, MAX_UPSTREAM_HTTP1_HEADERS, inspect_response_head};

const MAX_FUZZ_WIRE_BYTES: usize = MAX_UPSTREAM_HTTP1_HEAD_BYTES.saturating_add(1024);

/// Exercises Hyper HTTP/1 wire parsing and production response processing.
///
/// This entry point exists only under the opt-in `fuzzing` feature.
#[doc(hidden)]
pub fn fuzz_raw_http1_wire(data: &[u8]) {
    if tokio::runtime::Handle::try_current().is_ok() {
        return;
    }
    let Ok(runtime) = tokio::runtime::Builder::new_current_thread().build() else {
        return;
    };
    let wire = data
        .get(..data.len().min(MAX_FUZZ_WIRE_BYTES))
        .unwrap_or_default();
    let _ = runtime.block_on(exercise_wire(wire));
}

async fn exercise_wire(wire: &[u8]) -> bool {
    let capacity = MAX_FUZZ_WIRE_BYTES.saturating_add(1024);
    let (client_io, mut peer) = tokio::io::duplex(capacity);
    let response_bytes = wire.to_vec();
    let peer_task = tokio::spawn(async move {
        let mut request = [0_u8; 1024];
        let mut request_len = 0_usize;
        while request_len < request.len() {
            let Some(remaining) = request.get_mut(request_len..) else {
                return;
            };
            let Ok(read) = peer.read(remaining).await else {
                return;
            };
            if read == 0 {
                return;
            }
            request_len = request_len.saturating_add(read);
            if request
                .get(..request_len)
                .is_some_and(|bytes| bytes.windows(4).any(|window| window == b"\r\n\r\n"))
            {
                break;
            }
        }
        let _ = peer.write_all(&response_bytes).await;
        let _ = peer.shutdown().await;
    });

    let mut builder = http1::Builder::new();
    builder
        .max_headers(MAX_UPSTREAM_HTTP1_HEADERS)
        .max_buf_size(MAX_UPSTREAM_HTTP1_HEAD_BYTES);
    let Ok((mut sender, connection)) = builder.handshake(TokioIo::new(client_io)).await else {
        peer_task.abort();
        return false;
    };
    let connection_task = tokio::spawn(async move {
        let _ = connection.await;
    });
    let Ok(mut request) = http::Request::builder()
        .method(http::Method::GET)
        .uri("/")
        .body(Empty::<Bytes>::new())
    else {
        connection_task.abort();
        peer_task.abort();
        return false;
    };
    let state = Arc::new(ResponseState::new(8));
    let observer = Arc::clone(&state);
    hyper::ext::on_informational(&mut request, move |head| {
        observer.observe_informational(head.status().as_u16());
    });
    let response = sender.send_request(request).await;
    let accepted = match response {
        Ok(response) if state.informational_rejection().is_none() => {
            process_response(response).await
        }
        Ok(_) | Err(_) => false,
    };
    connection_task.abort();
    peer_task.abort();
    accepted
}

async fn process_response(response: http::Response<hyper::body::Incoming>) -> bool {
    let Some(status) = StatusCode::new(response.status().as_u16()) else {
        return false;
    };
    let Ok(content_type) = HeaderName::new("content-type") else {
        return false;
    };
    let Ok(request_id) = HeaderName::new("x-request-id") else {
        return false;
    };
    let admitted = [content_type, request_id];
    let media = [MediaType::JSON];
    let Ok(policy) = RawResponsePolicy::new(
        1024,
        1024,
        ResponseMediaPolicy::Optional(&media),
        ResponseMediaPolicy::Optional(&media),
        &admitted,
        8,
    ) else {
        return false;
    };
    let mut body = [0_u8; 1024];
    let mut header_storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
    let mut buffer = ResponseBuffer::new(&mut body, 1024, &mut header_storage);
    let Ok(mut attempt) = buffer.writer().begin_attempt() else {
        return false;
    };
    let capacity = attempt.body_capacity();
    let Ok(headers) = attempt.headers_mut() else {
        return false;
    };
    let limit = inspect_response_head(
        Method::Get,
        status,
        response.headers(),
        policy,
        headers,
        capacity,
    );
    if let Ok(limit) = limit
        && let Ok(len) = read_bounded_body(response.into_body(), &mut attempt, limit).await
    {
        return attempt
            .commit(status, len, cloud_sdk::transport::ResponseMetadata::EMPTY)
            .is_ok();
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{MAX_UPSTREAM_HTTP1_HEAD_BYTES, exercise_wire};

    #[test]
    fn wire_parser_exercises_below_exact_and_plus_one_head_bounds() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .build()
            .unwrap_or_else(|_| unreachable!());
        runtime.block_on(async {
            let below =
                response_with_encoded_header_len(MAX_UPSTREAM_HTTP1_HEAD_BYTES.saturating_sub(1));
            let exact = response_with_encoded_header_len(MAX_UPSTREAM_HTTP1_HEAD_BYTES);
            let oversized =
                response_with_encoded_header_len(MAX_UPSTREAM_HTTP1_HEAD_BYTES.saturating_add(1));
            assert!(exercise_wire(&below).await);
            assert!(exercise_wire(&exact).await);
            assert!(!exercise_wire(&oversized).await);
        });
    }

    fn response_with_encoded_header_len(encoded_len: usize) -> std::vec::Vec<u8> {
        const STATUS: &[u8] = b"HTTP/1.1 200 OK\r\n";
        const FILL_HEADERS: usize = 98;
        const CONTENT_LENGTH: &[u8] = b"Content-Length: 0\r\n";
        let filler_names: std::vec::Vec<std::string::String> = (0..FILL_HEADERS)
            .map(|index| std::format!("X-Fill-{index:02}"))
            .collect();
        let fixed_encoded_len = filler_names
            .iter()
            .map(|name| name.len().saturating_add(4))
            .sum::<usize>()
            .saturating_add("content-length".len())
            .saturating_add(1)
            .saturating_add(4);
        let fill_len = encoded_len
            .checked_sub(fixed_encoded_len)
            .unwrap_or_else(|| unreachable!());
        let mut response = std::vec::Vec::with_capacity(
            STATUS.len().saturating_add(encoded_len).saturating_add(2),
        );
        response.extend_from_slice(STATUS);
        let base = fill_len / FILL_HEADERS;
        let remainder = fill_len % FILL_HEADERS;
        for (index, name) in filler_names.iter().enumerate() {
            response.extend_from_slice(name.as_bytes());
            response.extend_from_slice(b": ");
            let value_len = base
                .checked_add(usize::from(index < remainder))
                .unwrap_or_else(|| unreachable!());
            response.resize(response.len().saturating_add(value_len), b'a');
            response.extend_from_slice(b"\r\n");
        }
        response.extend_from_slice(CONTENT_LENGTH);
        response.extend_from_slice(b"\r\n");
        assert_eq!(
            response.len(),
            STATUS.len().saturating_add(encoded_len).saturating_add(2)
        );
        response
    }
}
