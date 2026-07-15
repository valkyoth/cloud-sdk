//! Redacted records of prepared request policy and shape.

use core::fmt;

use cloud_sdk::Method;
use cloud_sdk::operation::{OperationMetadata, PreparedRequest, ProviderService, ResponsePolicy};

/// Non-secret record of one prepared request for policy assertions.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PreparedRequestRecord {
    method: Method,
    target_len: usize,
    body_len: usize,
    has_request_content_type: bool,
    service: ProviderService,
    metadata: OperationMetadata,
    response_policy: ResponsePolicy,
}

impl PreparedRequestRecord {
    /// Captures request shape and complete policy without copying target or body bytes.
    #[must_use]
    pub const fn capture(prepared: PreparedRequest<'_>) -> Self {
        let request = prepared.transport_request();
        Self {
            method: request.method(),
            target_len: request.target().as_str().len(),
            body_len: request.body().len(),
            has_request_content_type: request.content_type().is_some(),
            service: prepared.service(),
            metadata: prepared.metadata(),
            response_policy: prepared.response_policy(),
        }
    }

    /// Returns the HTTP method.
    #[must_use]
    pub const fn method(self) -> Method {
        self.method
    }

    /// Returns the redacted request-target length.
    #[must_use]
    pub const fn target_len(self) -> usize {
        self.target_len
    }

    /// Returns the redacted request-body length.
    #[must_use]
    pub const fn body_len(self) -> usize {
        self.body_len
    }

    /// Reports whether a request content type is configured.
    #[must_use]
    pub const fn has_request_content_type(self) -> bool {
        self.has_request_content_type
    }

    /// Returns the bound provider service and endpoint.
    #[must_use]
    pub const fn service(self) -> ProviderService {
        self.service
    }

    /// Returns complete operation safety and retry metadata.
    #[must_use]
    pub const fn metadata(self) -> OperationMetadata {
        self.metadata
    }

    /// Returns complete checked-response policy.
    #[must_use]
    pub const fn response_policy(self) -> ResponsePolicy {
        self.response_policy
    }
}

impl fmt::Debug for PreparedRequestRecord {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedRequestRecord")
            .field("method", &self.method)
            .field("target_len", &self.target_len)
            .field("target", &"[redacted]")
            .field("body_len", &self.body_len)
            .field("body", &"[redacted]")
            .field("has_request_content_type", &self.has_request_content_type)
            .field("service", &self.service)
            .field("metadata", &self.metadata)
            .field("response_policy", &self.response_policy)
            .finish()
    }
}
