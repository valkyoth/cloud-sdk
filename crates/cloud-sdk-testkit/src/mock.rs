//! Deterministic no-allocation mock transport.

use core::fmt;

use cloud_sdk::Method;
use cloud_sdk::transport::{
    AsyncTransport, BlockingTransport, RequestTarget, TransportRequest, TransportResponse,
};

use crate::{FixtureBodyError, ResponseFixture};

/// Expected request fields for one mock exchange.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ExpectedRequest<'a> {
    method: Method,
    target: RequestTarget<'a>,
    body: &'a [u8],
}

impl<'a> ExpectedRequest<'a> {
    /// Creates a bodyless expected request.
    #[must_use]
    pub const fn new(method: Method, target: RequestTarget<'a>) -> Self {
        Self {
            method,
            target,
            body: &[],
        }
    }

    /// Adds the exact expected request body.
    #[must_use]
    pub const fn with_body(mut self, body: &'a [u8]) -> Self {
        self.body = body;
        self
    }

    const fn method(self) -> Method {
        self.method
    }

    const fn target(self) -> RequestTarget<'a> {
        self.target
    }

    const fn body(self) -> &'a [u8] {
        self.body
    }
}

impl fmt::Debug for ExpectedRequest<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ExpectedRequest")
            .field("method", &self.method)
            .field("target", &"[redacted]")
            .field("body", &"[redacted]")
            .finish()
    }
}

/// One expected request and deterministic response.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MockExchange<'a> {
    request: ExpectedRequest<'a>,
    response: ResponseFixture<'a>,
}

impl<'a> MockExchange<'a> {
    /// Creates one mock exchange.
    #[must_use]
    pub const fn new(request: ExpectedRequest<'a>, response: ResponseFixture<'a>) -> Self {
        Self { request, response }
    }
}

/// Deterministic mock transport failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MockError {
    /// No expected exchange remains.
    Exhausted,
    /// HTTP method differs from the next expectation.
    MethodMismatch,
    /// Request target differs from the next expectation.
    TargetMismatch,
    /// Request body differs from the next expectation.
    BodyMismatch,
    /// Caller response buffer cannot hold the complete fixture body.
    ResponseBufferTooSmall,
    /// Internal cursor arithmetic failed closed.
    CursorOverflow,
    /// Fixture metadata could not be represented by the core transport.
    InvalidFixtureMetadata,
}

impl_static_error!(MockError,
    Self::Exhausted => "mock transport has no expected exchange remaining",
    Self::MethodMismatch => "mock request method differs from expectation",
    Self::TargetMismatch => "mock request target differs from expectation",
    Self::BodyMismatch => "mock request body differs from expectation",
    Self::ResponseBufferTooSmall => "mock response buffer is too small",
    Self::CursorOverflow => "mock transport cursor overflowed",
    Self::InvalidFixtureMetadata => "mock fixture metadata is invalid",
);

/// Ordered no-allocation mock implementation of [`BlockingTransport`].
pub struct MockTransport<'a> {
    exchanges: &'a [MockExchange<'a>],
    cursor: usize,
}

impl<'a> MockTransport<'a> {
    /// Creates a mock over an ordered exchange slice.
    #[must_use]
    pub const fn new(exchanges: &'a [MockExchange<'a>]) -> Self {
        Self {
            exchanges,
            cursor: 0,
        }
    }

    /// Returns the number of exchanges not yet consumed.
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.exchanges.len().saturating_sub(self.cursor)
    }

    /// Reports whether every expected exchange was consumed.
    #[must_use]
    pub const fn is_complete(&self) -> bool {
        self.remaining() == 0
    }

    fn send_inner<'buffer>(
        &mut self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, MockError> {
        let exchange = self
            .exchanges
            .get(self.cursor)
            .ok_or(MockError::Exhausted)?;
        if request.method() != exchange.request.method() {
            return Err(MockError::MethodMismatch);
        }
        if request.target() != exchange.request.target() {
            return Err(MockError::TargetMismatch);
        }
        if request.body() != exchange.request.body() {
            return Err(MockError::BodyMismatch);
        }
        let next_cursor = self
            .cursor
            .checked_add(1)
            .ok_or(MockError::CursorOverflow)?;
        let body_len =
            exchange
                .response
                .body()
                .write_to(response_body)
                .map_err(|error| match error {
                    FixtureBodyError::OutputTooSmall | FixtureBodyError::TooLarge => {
                        MockError::ResponseBufferTooSmall
                    }
                })?;
        let initialized = response_body
            .get(..body_len)
            .ok_or(MockError::ResponseBufferTooSmall)?;
        self.cursor = next_cursor;
        let response = TransportResponse::new(exchange.response.status(), initialized);
        let rate_limit = exchange
            .response
            .rate_limit()
            .map(|value| value.into_rate_limit())
            .transpose()
            .map_err(|_| MockError::InvalidFixtureMetadata)?;
        Ok(rate_limit.map_or(response, |value| response.with_rate_limit(value)))
    }
}

impl BlockingTransport for MockTransport<'_> {
    type Error = MockError;

    fn send<'buffer>(
        &mut self,
        request: TransportRequest<'_>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error> {
        self.send_inner(request, response_body)
    }
}

impl AsyncTransport for MockTransport<'_> {
    type Error = MockError;

    async fn send<'transport, 'request, 'buffer>(
        &'transport mut self,
        request: TransportRequest<'request>,
        response_body: &'buffer mut [u8],
    ) -> Result<TransportResponse<'buffer>, Self::Error>
    where
        'request: 'transport,
        'buffer: 'transport,
    {
        self.send_inner(request, response_body)
    }
}

impl fmt::Debug for MockTransport<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("MockTransport")
            .field("remaining", &self.remaining())
            .finish_non_exhaustive()
    }
}
