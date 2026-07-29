//! Conservative transport-delivery classification.

use core::fmt;

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

/// Payload-redacting transport failure with an explicit delivery phase.
///
/// Unknown send state must use [`Self::unknown`], which conservatively maps to
/// [`DeliveryPhase::PossiblySent`].
#[derive(Clone, Copy)]
pub struct TransportFailure<E> {
    phase: DeliveryPhase,
    error: E,
}

impl<E> TransportFailure<E> {
    /// Records a failure proven to have happened before request delivery.
    #[must_use]
    pub const fn not_sent(error: E) -> Self {
        Self {
            phase: DeliveryPhase::NotSent,
            error,
        }
    }

    /// Records a failure after request delivery may have begun.
    #[must_use]
    pub const fn possibly_sent(error: E) -> Self {
        Self {
            phase: DeliveryPhase::PossiblySent,
            error,
        }
    }

    /// Records a failure after any response head was observed.
    #[must_use]
    pub const fn response_started(error: E) -> Self {
        Self {
            phase: DeliveryPhase::ResponseStarted,
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
}

impl<E: PartialEq> PartialEq for TransportFailure<E> {
    fn eq(&self, other: &Self) -> bool {
        self.phase == other.phase && self.error == other.error
    }
}

impl<E: Eq> Eq for TransportFailure<E> {}

impl<E> fmt::Debug for TransportFailure<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("TransportFailure")
            .field("phase", &self.phase)
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

    use super::{DeliveryPhase, TransportFailure};

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
}
