use super::{
    DiagnosticErrorCategory, DiagnosticEvent, DiagnosticObserver, DiagnosticRequestId,
    NoopDiagnosticObserver,
};
use crate::operation::RequestIdPolicy;
use crate::{ProviderId, ServiceId};

#[cfg(feature = "std")]
use super::{DiagnosticContext, DiagnosticResponse, DiagnosticRetryCategory};
#[cfg(feature = "std")]
use crate::operation::OperationImpact;
#[cfg(feature = "std")]
use crate::transport::StatusCode;
#[cfg(feature = "std")]
use crate::{operation_id, provider_id, service_id};

#[cfg(feature = "std")]
use crate::std as test_std;

#[test]
fn request_id_policy_never_exposes_identifier_bytes_or_discard_presence() {
    assert_eq!(
        DiagnosticRequestId::classify(RequestIdPolicy::Discard, false),
        DiagnosticRequestId::Discarded
    );
    assert_eq!(
        DiagnosticRequestId::classify(RequestIdPolicy::Discard, true),
        DiagnosticRequestId::Discarded
    );
    assert_eq!(
        DiagnosticRequestId::classify(RequestIdPolicy::Protected, false),
        DiagnosticRequestId::Absent
    );
    assert_eq!(
        DiagnosticRequestId::classify(RequestIdPolicy::Protected, true),
        DiagnosticRequestId::Protected
    );
    assert_eq!(
        DiagnosticRequestId::classify(RequestIdPolicy::Retain, true),
        DiagnosticRequestId::Retainable
    );
}

struct NonDebugObserverError;

struct FailingObserver;

impl DiagnosticObserver for FailingObserver {
    type Error = NonDebugObserverError;

    fn observe(&self, _event: DiagnosticEvent) -> Result<(), Self::Error> {
        Err(NonDebugObserverError)
    }
}

#[test]
fn observer_errors_need_no_debug_contract_and_are_isolated() {
    super::notify(
        &FailingObserver,
        DiagnosticEvent::PreparationFailed {
            error: DiagnosticErrorCategory::Preparation,
        },
    );
    assert!(
        NoopDiagnosticObserver
            .observe(DiagnosticEvent::PreparationStarted)
            .is_ok()
    );
}

#[test]
fn diagnostic_identity_inputs_enforce_exact_public_bounds() {
    const SIXTY_THREE: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    const ONE_TWENTY_EIGHT: &str = concat!(
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    );
    assert_eq!(SIXTY_THREE.len(), 63);
    assert_eq!(ONE_TWENTY_EIGHT.len(), 128);
    assert!(ProviderId::new(SIXTY_THREE).is_ok());
    assert!(ServiceId::new(SIXTY_THREE).is_ok());
    assert!(crate::operation::OperationId::new(ONE_TWENTY_EIGHT).is_ok());
}

#[cfg(feature = "std")]
#[test]
fn event_debug_output_is_a_stable_payload_free_snapshot() {
    let context = DiagnosticContext::new(
        provider_id!("example"),
        service_id!("compute"),
        Some(operation_id!("list_servers")),
        OperationImpact::ReadOnly,
        DiagnosticRetryCategory::ExplicitPolicy,
    );
    let response = DiagnosticResponse::new(StatusCode::OK, DiagnosticRequestId::Protected);
    assert_eq!(
        test_std::format!(
            "{:?}",
            DiagnosticEvent::PreparationFailed {
                error: DiagnosticErrorCategory::Preparation,
            }
        ),
        "PreparationFailed { error: Preparation }"
    );
    assert_eq!(
        test_std::format!(
            "{:?}",
            DiagnosticEvent::ExecutionFailed {
                context,
                error: DiagnosticErrorCategory::Transport,
            }
        ),
        concat!(
            "ExecutionFailed { context: DiagnosticContext { provider: ",
            "ProviderId(\"example\"), service: ServiceId(\"compute\"), operation: ",
            "Some(OperationId(\"list_servers\")), impact: ReadOnly, retry: ",
            "ExplicitPolicy }, error: Transport }"
        )
    );
    assert_eq!(
        test_std::format!(
            "{:?}",
            DiagnosticEvent::ResponseReceived { context, response }
        ),
        concat!(
            "ResponseReceived { context: DiagnosticContext { provider: ",
            "ProviderId(\"example\"), service: ServiceId(\"compute\"), operation: ",
            "Some(OperationId(\"list_servers\")), impact: ReadOnly, retry: ",
            "ExplicitPolicy }, response: DiagnosticResponse { status: StatusCode(200), ",
            "request_id: Protected } }"
        )
    );
}
