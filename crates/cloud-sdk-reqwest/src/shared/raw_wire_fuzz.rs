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

const MAX_FUZZ_WIRE_BYTES: usize = 16 * 1024;

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
    runtime.block_on(exercise_wire(wire));
}

async fn exercise_wire(wire: &[u8]) {
    let capacity = MAX_FUZZ_WIRE_BYTES.saturating_add(1024);
    let (client_io, mut peer) = tokio::io::duplex(capacity);
    let response_bytes = wire.to_vec();
    let peer_task = tokio::spawn(async move {
        let _ = peer.write_all(&response_bytes).await;
        let _ = peer.shutdown().await;
        let mut request = [0_u8; 1024];
        while peer.read(&mut request).await.is_ok_and(|read| read != 0) {}
    });

    let mut builder = http1::Builder::new();
    builder
        .max_headers(MAX_UPSTREAM_HTTP1_HEADERS)
        .max_buf_size(MAX_UPSTREAM_HTTP1_HEAD_BYTES);
    let Ok((mut sender, connection)) = builder.handshake(TokioIo::new(client_io)).await else {
        peer_task.abort();
        return;
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
        return;
    };
    let state = Arc::new(ResponseState::new(8));
    let observer = Arc::clone(&state);
    hyper::ext::on_informational(&mut request, move |head| {
        observer.observe_informational(head.status().as_u16());
    });
    let response = sender.send_request(request).await;
    if let Ok(response) = response
        && state.informational_rejection().is_none()
    {
        process_response(response).await;
    }
    connection_task.abort();
    peer_task.abort();
}

async fn process_response(response: http::Response<hyper::body::Incoming>) {
    let Some(status) = StatusCode::new(response.status().as_u16()) else {
        return;
    };
    let Ok(content_type) = HeaderName::new("content-type") else {
        return;
    };
    let Ok(request_id) = HeaderName::new("x-request-id") else {
        return;
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
        return;
    };
    let mut body = [0_u8; 1024];
    let mut header_storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
    let mut buffer = ResponseBuffer::new(&mut body, 1024, &mut header_storage);
    let Ok(mut attempt) = buffer.writer().begin_attempt() else {
        return;
    };
    let capacity = attempt.body_capacity();
    let Ok(headers) = attempt.headers_mut() else {
        return;
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
        let _ = attempt.commit(status, len, cloud_sdk::transport::ResponseMetadata::EMPTY);
    }
}
