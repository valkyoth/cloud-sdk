/// Client construction failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuildError {
    /// Reqwest rejected the fixed hardened client configuration.
    ClientBuildFailed,
}

/// Payload-free transport failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportError {
    /// The target could not be composed without parsing or normalization.
    TargetRejected,
    /// A non-empty body omitted its required explicit content type.
    MissingContentType,
    /// A validated header could not be represented by the HTTP implementation.
    HeaderRejected,
    /// Adapter-owned request-body allocation failed.
    RequestBodyAllocationFailed,
    /// The request body length cannot be represented by the HTTP client.
    RequestBodyTooLarge,
    /// Adapter-owned response-body allocation failed.
    ResponseBodyAllocationFailed,
    /// Connection establishment failed.
    ConnectFailed,
    /// The configured request or read deadline expired.
    TimedOut,
    /// Sending failed for another payload-free reason.
    RequestFailed,
    /// The response status is outside the core SDK's admitted HTTP range.
    InvalidStatus,
    /// The declared or observed response body exceeds the caller buffer.
    ResponseTooLarge,
    /// Reading the response body failed.
    ResponseReadFailed,
    /// The final response origin differed from the configured endpoint.
    ResponseOriginChanged,
}
