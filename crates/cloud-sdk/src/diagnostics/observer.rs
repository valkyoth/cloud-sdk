use super::DiagnosticEvent;

/// Caller-owned opt-in sink for payload-free lifecycle events.
///
/// Core never logs and never retains the observer or its errors. The shared
/// receiver permits reentrant use and concurrent observation when the caller's
/// implementation supplies the necessary interior synchronization. Returned
/// errors are ignored; panics follow ordinary Rust panic behavior while the
/// workspace lease retains cleanup ownership during unwinding.
pub trait DiagnosticObserver {
    /// Caller-specific observation failure.
    type Error;

    /// Observes one copy-only event. Returned errors never alter SDK execution.
    fn observe(&self, event: DiagnosticEvent) -> Result<(), Self::Error>;
}

/// Disabled observer used by ordinary client methods.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct NoopDiagnosticObserver;

impl DiagnosticObserver for NoopDiagnosticObserver {
    type Error = core::convert::Infallible;

    fn observe(&self, _event: DiagnosticEvent) -> Result<(), Self::Error> {
        Ok(())
    }
}

pub(crate) fn notify<O>(observer: &O, event: DiagnosticEvent)
where
    O: DiagnosticObserver + ?Sized,
{
    let _ignored = observer.observe(event);
}
