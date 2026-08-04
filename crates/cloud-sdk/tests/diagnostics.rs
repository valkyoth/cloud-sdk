//! Downstream compile proof for the payload-free observer contract.

use cloud_sdk::diagnostics::{DiagnosticEvent, DiagnosticObserver, NoopDiagnosticObserver};

struct DownstreamError;

struct DownstreamObserver;

impl DiagnosticObserver for DownstreamObserver {
    type Error = DownstreamError;

    fn observe(&self, _event: DiagnosticEvent) -> Result<(), Self::Error> {
        Err(DownstreamError)
    }
}

#[test]
fn downstream_observers_need_no_error_formatting_contract() {
    assert!(
        DownstreamObserver
            .observe(DiagnosticEvent::PreparationStarted)
            .is_err()
    );
    assert!(
        NoopDiagnosticObserver
            .observe(DiagnosticEvent::PreparationStarted)
            .is_ok()
    );
}
