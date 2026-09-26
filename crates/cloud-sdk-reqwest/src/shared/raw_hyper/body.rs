use super::{Bytes, RawHttpError};
use core::{
    pin::Pin,
    task::{Context, Poll},
};
use http_body_util::Full;
use hyper::body::{Body, Frame, SizeHint};
#[cfg(test)]
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
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
    #[cfg(test)]
    drop_probe: Option<Arc<AtomicUsize>>,
}
impl SanitizedBody {
    pub(super) fn copy_from(source: &[u8]) -> Result<Self, ()> {
        Self::copy_parts(source, &[])
    }
    pub(super) fn copy_parts(prefix: &[u8], suffix: &[u8]) -> Result<Self, ()> {
        let length = prefix.len().checked_add(suffix.len()).ok_or(())?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(|_| ())?;
        bytes.extend_from_slice(prefix);
        bytes.extend_from_slice(suffix);
        Ok(Self {
            bytes,
            #[cfg(test)]
            drop_probe: None,
        })
    }
    pub(super) fn into_bytes(self) -> Bytes {
        Bytes::from_owner(self)
    }
    #[cfg(test)]
    pub(super) fn with_drop_probe(mut self, probe: Arc<AtomicUsize>) -> Self {
        self.drop_probe = Some(probe);
        self
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
        #[cfg(test)]
        if let Some(probe) = &self.drop_probe {
            assert!(self.bytes.iter().all(|byte| *byte == 0));
            probe.fetch_add(1, Ordering::SeqCst);
        }
    }
}
