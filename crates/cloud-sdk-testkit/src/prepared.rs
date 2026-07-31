//! Redacted records of prepared request policy and shape.

use core::fmt;

use cloud_sdk::Method;
use cloud_sdk::authentication::AuthenticationScopePolicy;
use cloud_sdk::operation::{OperationMetadata, PreparedRequest, ProviderService, ResponsePolicy};
use cloud_sdk::transport::{HeaderSensitivity, RawResponsePolicy};

/// Non-secret record of one prepared request for policy assertions.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PreparedRequestRecord<'endpoint> {
    method: Method,
    target_len: usize,
    body_len: usize,
    has_request_content_type: bool,
    header_count: usize,
    sensitive_header_count: usize,
    service: ProviderService<'endpoint>,
    metadata: OperationMetadata,
    response_policy: ResponsePolicy,
    authentication_policy: AuthenticationScopePolicy<'endpoint>,
    raw_response_policy: RawResponsePolicy<'endpoint>,
}

impl<'endpoint> PreparedRequestRecord<'endpoint> {
    /// Captures request shape and complete policy without copying target or body bytes.
    #[must_use]
    pub fn capture(prepared: PreparedRequest<'endpoint>) -> Self {
        let request = prepared.transport_request();
        Self {
            method: request.method(),
            target_len: request.target().len(),
            body_len: request.body().len(),
            has_request_content_type: request.headers().get("content-type").is_some(),
            header_count: request.headers().as_slice().len(),
            sensitive_header_count: request
                .headers()
                .as_slice()
                .iter()
                .filter(|header| matches!(header.sensitivity(), HeaderSensitivity::Sensitive))
                .count(),
            service: prepared.service(),
            metadata: prepared.metadata(),
            response_policy: prepared.response_policy(),
            authentication_policy: prepared.authentication_policy(),
            raw_response_policy: prepared.raw_response_policy(),
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

    /// Returns the request-header count without exposing values.
    #[must_use]
    pub const fn header_count(self) -> usize {
        self.header_count
    }

    /// Returns the number of headers marked sensitive.
    #[must_use]
    pub const fn sensitive_header_count(self) -> usize {
        self.sensitive_header_count
    }

    /// Returns the bound provider service and endpoint.
    #[must_use]
    pub const fn service(self) -> ProviderService<'endpoint> {
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

    /// Returns the mandatory authentication-scope policy.
    #[must_use]
    pub const fn authentication_policy(self) -> AuthenticationScopePolicy<'endpoint> {
        self.authentication_policy
    }

    /// Returns the status-class raw response policy.
    #[must_use]
    pub const fn raw_response_policy(self) -> RawResponsePolicy<'endpoint> {
        self.raw_response_policy
    }
}

impl fmt::Debug for PreparedRequestRecord<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedRequestRecord")
            .field("method", &self.method)
            .field("target_len", &self.target_len)
            .field("target", &"[redacted]")
            .field("body_len", &self.body_len)
            .field("body", &"[redacted]")
            .field("has_request_content_type", &self.has_request_content_type)
            .field("header_count", &self.header_count)
            .field("sensitive_header_count", &self.sensitive_header_count)
            .field("service", &self.service)
            .field("metadata", &self.metadata)
            .field("response_policy", &self.response_policy)
            .field("authentication_policy", &self.authentication_policy)
            .field("raw_response_policy", &self.raw_response_policy)
            .finish()
    }
}
