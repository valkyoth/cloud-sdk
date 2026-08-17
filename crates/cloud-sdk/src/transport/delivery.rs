//! Conservative transport-delivery classification.

use core::fmt;

use super::StatusCode;

/// Furthest delivery point known for one failed HTTP attempt.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeliveryPhase {
    /// Validation or connection establishment failed before request delivery.
    NotSent,
    /// Some request bytes may have reached the peer.
    PossiblySent,
    /// Any informational or final response head was observed from the peer.
    ResponseStarted,
}

/// Error whose furthest possible request-delivery phase is explicit.
///
/// Implementations should retain a final response status once observed so
/// authentication and reconciliation policy can classify later failures.
pub trait DeliveryClassified {
    /// Returns the conservative delivery phase for permit transitions.
    fn delivery_phase(&self) -> DeliveryPhase;

    /// Returns a valid final status observed before the failure, when known.
    fn observed_status(&self) -> Option<StatusCode> {
        None
    }
}

/// Payload-redacting transport failure with an explicit delivery phase and
/// optional already-observed final status.
///
/// Unknown send state must use [`Self::unknown`], which conservatively maps to
/// [`DeliveryPhase::PossiblySent`].
#[derive(Clone, Copy)]
pub struct TransportFailure<E> {
    phase: DeliveryPhase,
    observed_status: Option<StatusCode>,
    error: E,
}

impl<E> TransportFailure<E> {
    /// Records a failure proven to have happened before request delivery.
    #[must_use]
    pub const fn not_sent(error: E) -> Self {
        Self {
            phase: DeliveryPhase::NotSent,
            observed_status: None,
            error,
        }
    }

    /// Records a failure after request delivery may have begun.
    #[must_use]
    pub const fn possibly_sent(error: E) -> Self {
        Self {
            phase: DeliveryPhase::PossiblySent,
            observed_status: None,
            error,
        }
    }

    /// Records a failure after any response head was observed.
    #[must_use]
    pub const fn response_started(error: E) -> Self {
        Self {
            phase: DeliveryPhase::ResponseStarted,
            observed_status: None,
            error,
        }
    }

    /// Records a failure after the supplied final response status was observed.
    #[must_use]
    pub const fn response_started_with_status(status: StatusCode, error: E) -> Self {
        Self {
            phase: DeliveryPhase::ResponseStarted,
            observed_status: Some(status),
            error,
        }
    }

    /// Conservatively records an error whose delivery state is unknown.
    #[must_use]
    pub const fn unknown(error: E) -> Self {
        Self::possibly_sent(error)
    }

    /// Returns the conservative delivery phase.
    #[must_use]
    pub const fn phase(&self) -> DeliveryPhase {
        self.phase
    }

    /// Returns a final status observed before response processing failed.
    #[must_use]
    pub const fn observed_status(&self) -> Option<StatusCode> {
        self.observed_status
    }

    /// Preserves this failure while attaching a subsequently known final status.
    #[must_use]
    pub const fn with_observed_status(mut self, status: StatusCode) -> Self {
        self.phase = DeliveryPhase::ResponseStarted;
        self.observed_status = Some(status);
        self
    }

    /// Returns a shared view of the payload-free adapter error.
    #[must_use]
    pub const fn error(&self) -> &E {
        &self.error
    }

    /// Consumes the failure and returns its adapter error.
    #[must_use]
    pub fn into_error(self) -> E {
        self.error
    }

    /// Transforms the payload-free adapter error while preserving delivery state.
    #[must_use]
    pub fn map<F>(self, transform: impl FnOnce(E) -> F) -> TransportFailure<F> {
        TransportFailure {
            phase: self.phase,
            observed_status: self.observed_status,
            error: transform(self.error),
        }
    }
}

impl<E> DeliveryClassified for TransportFailure<E> {
    fn delivery_phase(&self) -> DeliveryPhase {
        self.phase
    }

    fn observed_status(&self) -> Option<StatusCode> {
        self.observed_status
    }
}

impl<E: PartialEq> PartialEq for TransportFailure<E> {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase
            && self.observed_status == other.observed_status
            && self.error == other.error
    }
}

impl<E: Eq> Eq for TransportFailure<E> {}

impl<E> fmt::Debug for TransportFailure<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportFailure")
            .field("phase", &self.phase)
            .field("observed_status", &self.observed_status)
            .field("error", &"[redacted]")
            .finish()
    }
}

impl<E> fmt::Display for TransportFailure<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.phase {
            DeliveryPhase::NotSent => formatter.write_str("transport failed before delivery"),
            DeliveryPhase::PossiblySent => {
                formatter.write_str("transport failed with uncertain delivery")
            }
            DeliveryPhase::ResponseStarted => {
                formatter.write_str("transport failed after response start")
            }
        }
    }
}

impl<E> core::error::Error for TransportFailure<E> {}

#[cfg(test)]
mod tests {
    use core::fmt::{self, Write};

    use super::{DeliveryClassified, DeliveryPhase, StatusCode, TransportFailure};

    struct FixedText {
        bytes: [u8; 96],
        len: usize,
    }

    impl FixedText {
        const fn new() -> Self {
            Self {
                bytes: [0_u8; 96],
                len: 0,
            }
        }

        fn as_str(&self) -> &str {
            core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
        }
    }

    impl Write for FixedText {
        fn write_str(&mut self, value: &str) -> fmt::Result {
            let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
            let output = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
            output.copy_from_slice(value.as_bytes());
            self.len = end;
            Ok(())
        }
    }

    #[test]
    fn unknown_delivery_fails_closed_and_debug_redacts_the_error() {
        let failure = TransportFailure::unknown("secret payload");
        assert_eq!(failure.phase(), DeliveryPhase::PossiblySent);
        let mut debug = FixedText::new();
        assert!(write!(&mut debug, "{failure:?}").is_ok());
        assert!(debug.as_str().contains("PossiblySent"));
        assert!(!debug.as_str().contains("secret payload"));
    }

    #[test]
    fn observed_status_survives_mapping_without_exposing_error_payload() {
        let failure = TransportFailure::response_started_with_status(
            StatusCode::new(401).unwrap_or(StatusCode::OK),
            "secret payload",
        )
        .map(|_| 7_u8);
        assert_eq!(failure.phase(), DeliveryPhase::ResponseStarted);
        assert_eq!(failure.observed_status().map(StatusCode::get), Some(401));
        assert_eq!(
            DeliveryClassified::observed_status(&failure),
            failure.observed_status()
        );
        assert_eq!(*failure.error(), 7);
    }
}
