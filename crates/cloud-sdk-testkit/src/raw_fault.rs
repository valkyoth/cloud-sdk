//! Deterministic delivery-phase fault injection for raw executors.

use cloud_sdk::transport::{
    AsyncRawHttpExecutor, BlockingRawHttpExecutor, RawResponsePolicy, ResponseWriter,
    TransportFailure, TransportRequest,
};

/// Failure point injected by [`RawFaultExecutor`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RawFault {
    /// Fail before any request bytes can be delivered.
    NotSent,
    /// Fail after delivery may have begun.
    PossiblySent,
    /// Fail after a response head has started.
    ResponseStarted,
    /// Exercise the mandatory unknown-to-possibly-sent mapping.
    Unknown,
}

/// Payload-free deterministic raw fault.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawFaultError;

impl core::fmt::Display for RawFaultError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("injected raw transport fault")
    }
}

impl core::error::Error for RawFaultError {}

/// No-allocation executor that fails at one selected delivery phase.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawFaultExecutor {
    fault: RawFault,
}

impl RawFaultExecutor {
    /// Creates one deterministic fault injector.
    #[must_use]
    pub const fn new(fault: RawFault) -> Self {
        Self { fault }
    }

    fn failure(self) -> TransportFailure<RawFaultError> {
        match self.fault {
            RawFault::NotSent => TransportFailure::not_sent(RawFaultError),
            RawFault::PossiblySent => TransportFailure::possibly_sent(RawFaultError),
            RawFault::ResponseStarted => TransportFailure::response_started(RawFaultError),
            RawFault::Unknown => TransportFailure::unknown(RawFaultError),
        }
    }
}

impl BlockingRawHttpExecutor for RawFaultExecutor {
    type Error = TransportFailure<RawFaultError>;

    fn execute(
        &self,
        _request: TransportRequest<'_>,
        _policy: RawResponsePolicy<'_>,
        _response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        Err(self.failure())
    }
}

impl AsyncRawHttpExecutor for RawFaultExecutor {
    type Error = TransportFailure<RawFaultError>;

    async fn execute<'executor, 'request, 'policy, 'writer>(
        &'executor self,
        _request: TransportRequest<'request>,
        _policy: RawResponsePolicy<'policy>,
        _response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'executor: 'writer,
        'request: 'writer,
        'policy: 'writer,
    {
        Err(self.failure())
    }
}
