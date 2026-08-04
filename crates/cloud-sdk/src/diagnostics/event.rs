use crate::operation::{OperationId, OperationImpact, PreparedRequest};
use crate::transport::StatusCode;
use crate::{ProviderId, ServiceId};

use super::{DiagnosticErrorCategory, DiagnosticRequestId, DiagnosticRetryCategory};

/// Validated provider and operation context for one lifecycle event.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticContext {
    provider: ProviderId,
    service: ServiceId,
    operation: Option<OperationId>,
    impact: OperationImpact,
    retry: DiagnosticRetryCategory,
}

impl DiagnosticContext {
    pub(crate) const fn new(
        provider: ProviderId,
        service: ServiceId,
        operation: Option<OperationId>,
        impact: OperationImpact,
        retry: DiagnosticRetryCategory,
    ) -> Self {
        Self {
            provider,
            service,
            operation,
            impact,
            retry,
        }
    }

    pub(crate) fn from_prepared(prepared: &PreparedRequest<'_>) -> Self {
        let service = prepared.service();
        Self::new(
            service.provider_id(),
            service.service_id(),
            prepared.operation_id(),
            prepared.metadata().impact(),
            prepared.metadata().retry_eligibility().into(),
        )
    }

    /// Returns the validated provider identifier.
    #[must_use]
    pub const fn provider(self) -> ProviderId {
        self.provider
    }

    /// Returns the validated provider-owned service identifier.
    #[must_use]
    pub const fn service(self) -> ServiceId {
        self.service
    }

    /// Returns the provider operation identifier when one was bound.
    #[must_use]
    pub const fn operation(self) -> Option<OperationId> {
        self.operation
    }

    /// Returns the validated operation impact.
    #[must_use]
    pub const fn impact(self) -> OperationImpact {
        self.impact
    }

    /// Returns the operation retry category.
    #[must_use]
    pub const fn retry(self) -> DiagnosticRetryCategory {
        self.retry
    }
}

/// Payload-free response metadata admitted for diagnostics.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct DiagnosticResponse {
    status: StatusCode,
    request_id: DiagnosticRequestId,
}

impl DiagnosticResponse {
    pub(crate) const fn new(status: StatusCode, request_id: DiagnosticRequestId) -> Self {
        Self { status, request_id }
    }

    /// Returns the bounded HTTP status code.
    #[must_use]
    pub const fn status(self) -> StatusCode {
        self.status
    }

    /// Returns request-ID disposition without exposing identifier bytes.
    #[must_use]
    pub const fn request_id(self) -> DiagnosticRequestId {
        self.request_id
    }
}

/// Structured client lifecycle event containing no request or response payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticEvent {
    /// A caller-owned workspace was cleared before preparation began.
    PreparationStarted,
    /// Provider preparation failed before an identity could be trusted.
    PreparationFailed {
        /// Payload-free failure category.
        error: DiagnosticErrorCategory,
    },
    /// A complete provider request and policy context was prepared.
    RequestPrepared {
        /// Validated provider operation context.
        context: DiagnosticContext,
    },
    /// One authenticated transport attempt is about to begin.
    DispatchStarted {
        /// Validated provider operation context.
        context: DiagnosticContext,
    },
    /// Authenticated execution failed before checked decoding.
    ExecutionFailed {
        /// Validated provider operation context.
        context: DiagnosticContext,
        /// Payload-free failure category.
        error: DiagnosticErrorCategory,
    },
    /// A committed bounded response was received.
    ResponseReceived {
        /// Validated provider operation context.
        context: DiagnosticContext,
        /// Admitted response metadata.
        response: DiagnosticResponse,
    },
    /// Provider-owned checked decoding failed.
    DecodeFailed {
        /// Validated provider operation context.
        context: DiagnosticContext,
        /// Response metadata when a committed response was observable.
        response: Option<DiagnosticResponse>,
        /// Payload-free failure category.
        error: DiagnosticErrorCategory,
    },
    /// One operation completed successfully.
    Completed {
        /// Validated provider operation context.
        context: DiagnosticContext,
        /// Response metadata when a committed response was observable.
        response: Option<DiagnosticResponse>,
    },
}
