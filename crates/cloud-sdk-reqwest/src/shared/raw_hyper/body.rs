use super::{Bytes, RawHttpError};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use http_body_util::Full;
use hyper::body::{Body, Frame, SizeHint};
use std::vec::Vec;

pub(super) enum RequestBody {
    Full(Full<Bytes>),
    Upload(super::upload::UploadBody),
}
impl RequestBody {
    pub(super) fn full(bytes: Bytes) -> Self {
        Self::Full(Full::new(bytes))
    }
}
impl Body for RequestBody {
    type Data = Bytes;
    type Error = RawHttpError;
    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, RawHttpError>>> {
        match self.get_mut() {
            Self::Full(body) => Pin::new(body)
                .poll_frame(cx)
                .map(|frame| frame.map(|result| result.map_err(|never| match never {}))),
            Self::Upload(body) => Pin::new(body).poll_frame(cx),
        }
    }
    fn is_end_stream(&self) -> bool {
        match self {
            Self::Full(body) => body.is_end_stream(),
            Self::Upload(body) => body.is_end_stream(),
        }
    }
    fn size_hint(&self) -> SizeHint {
        match self {
            Self::Full(body) => body.size_hint(),
            Self::Upload(body) => body.size_hint(),
        }
    }
}

pub(super) struct SanitizedBody {
    bytes: Vec<u8>,
}
impl SanitizedBody {
    pub(super) fn copy_from(source: &[u8]) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(source.len()).map_err(|_| ())?;
        bytes.extend_from_slice(source);
        Ok(Self { bytes })
    }
    pub(super) fn into_bytes(self) -> Bytes {
        Bytes::from_owner(self)
    }
}
impl AsRef<[u8]> for SanitizedBody {
    fn as_ref(&self) -> &[u8] {
        &self.bytes
    }
}
impl Drop for SanitizedBody {
    fn drop(&mut self) {
        cloud_sdk_sanitization::sanitize_bytes(&mut self.bytes);
    }
}
