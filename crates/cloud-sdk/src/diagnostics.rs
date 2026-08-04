//! Opt-in payload-free lifecycle diagnostics without automatic logging.

mod category;
mod event;
mod observer;

pub use category::{DiagnosticErrorCategory, DiagnosticRequestId, DiagnosticRetryCategory};
pub use event::{DiagnosticContext, DiagnosticEvent, DiagnosticResponse};
pub(crate) use observer::notify;
pub use observer::{DiagnosticObserver, NoopDiagnosticObserver};

#[cfg(test)]
mod tests;
