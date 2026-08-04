//! Bounded, payload-free request observations for dynamic scenarios.

use core::sync::atomic::{AtomicUsize, Ordering};

use cloud_sdk::Method;
use cloud_sdk::transport::{StatusCode, TransportRequest};

/// Maximum caller-owned record slots accepted by one dynamic mock.
pub const MAX_DYNAMIC_RECORDS: usize = 1_024;

/// Payload-free HTTP method classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordedMethod {
    /// GET.
    Get,
    /// POST.
    Post,
    /// PUT.
    Put,
    /// DELETE.
    Delete,
    /// PATCH.
    Patch,
    /// HEAD.
    Head,
    /// OPTIONS.
    Options,
    /// A validated provider extension method. Its token is not retained.
    Extension,
}

impl RecordedMethod {
    pub(crate) fn from_method(method: Method) -> Self {
        match method {
            Method::Get => Self::Get,
            Method::Post => Self::Post,
            Method::Put => Self::Put,
            Method::Delete => Self::Delete,
            Method::Patch => Self::Patch,
            Method::Head => Self::Head,
            Method::Options => Self::Options,
            _ => Self::Extension,
        }
    }

    const fn encode(self) -> usize {
        match self {
            Self::Get => 0,
            Self::Post => 1,
            Self::Put => 2,
            Self::Delete => 3,
            Self::Patch => 4,
            Self::Head => 5,
            Self::Options => 6,
            Self::Extension => 7,
        }
    }

    const fn decode(value: usize) -> Self {
        match value {
            0 => Self::Get,
            1 => Self::Post,
            2 => Self::Put,
            3 => Self::Delete,
            4 => Self::Patch,
            5 => Self::Head,
            6 => Self::Options,
            _ => Self::Extension,
        }
    }
}

/// One committed request observation without request or response payloads.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecordedRequest {
    sequence: usize,
    method: RecordedMethod,
    target_len: usize,
    body_len: usize,
    header_count: usize,
    status: StatusCode,
}

impl RecordedRequest {
    /// Zero-based successful request sequence.
    #[must_use]
    pub const fn sequence(self) -> usize {
        self.sequence
    }

    /// Payload-free method classification.
    #[must_use]
    pub const fn method(self) -> RecordedMethod {
        self.method
    }

    /// Encoded target length; target bytes are never retained.
    #[must_use]
    pub const fn target_len(self) -> usize {
        self.target_len
    }

    /// Request body length; body bytes are never retained.
    #[must_use]
    pub const fn body_len(self) -> usize {
        self.body_len
    }

    /// Ordered request-header count; names and values are never retained.
    #[must_use]
    pub const fn header_count(self) -> usize {
        self.header_count
    }

    /// Committed response status.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        self.status
    }
}

/// Caller-owned atomic slot for one committed request observation.
pub struct RequestRecordSlot {
    ready: AtomicUsize,
    sequence: AtomicUsize,
    method: AtomicUsize,
    target_len: AtomicUsize,
    body_len: AtomicUsize,
    header_count: AtomicUsize,
    status: AtomicUsize,
}

impl RequestRecordSlot {
    /// Creates an empty reusable slot.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ready: AtomicUsize::new(0),
            sequence: AtomicUsize::new(0),
            method: AtomicUsize::new(0),
            target_len: AtomicUsize::new(0),
            body_len: AtomicUsize::new(0),
            header_count: AtomicUsize::new(0),
            status: AtomicUsize::new(0),
        }
    }

    /// Returns the committed observation, if any.
    #[must_use]
    pub fn get(&self) -> Option<RecordedRequest> {
        if self.ready.load(Ordering::Acquire) == 0 {
            return None;
        }
        let status = u16::try_from(self.status.load(Ordering::Relaxed)).ok()?;
        Some(RecordedRequest {
            sequence: self.sequence.load(Ordering::Relaxed),
            method: RecordedMethod::decode(self.method.load(Ordering::Relaxed)),
            target_len: self.target_len.load(Ordering::Relaxed),
            body_len: self.body_len.load(Ordering::Relaxed),
            header_count: self.header_count.load(Ordering::Relaxed),
            status: StatusCode::new(status)?,
        })
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.ready.load(Ordering::Acquire) == 0
    }

    pub(crate) fn commit(
        &self,
        sequence: usize,
        request: TransportRequest<'_>,
        status: StatusCode,
    ) {
        self.sequence.store(sequence, Ordering::Relaxed);
        self.method.store(
            RecordedMethod::from_method(request.method()).encode(),
            Ordering::Relaxed,
        );
        self.target_len
            .store(request.target().len(), Ordering::Relaxed);
        self.body_len.store(request.body().len(), Ordering::Relaxed);
        self.header_count
            .store(request.headers().as_slice().len(), Ordering::Relaxed);
        self.status
            .store(usize::from(status.get()), Ordering::Relaxed);
        self.ready.store(1, Ordering::Release);
    }
}

impl Default for RequestRecordSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for RequestRecordSlot {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_tuple("RequestRecordSlot")
            .field(&self.get())
            .finish()
    }
}
