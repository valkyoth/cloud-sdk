mod fixture;
mod response_diagnostics;

use core::cell::Cell;
use core::future::Future;
use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use core::task::{Context, Poll, Waker};

use super::{
    ClientExecutionError, ClientKernel, ClientWorkspace, ClientWorkspacePool,
    WorkspaceAcquireError, WorkspacePoolError,
};
use crate::diagnostics::{
    DiagnosticErrorCategory, DiagnosticEvent, DiagnosticObserver, DiagnosticRequestId,
    DiagnosticRetryCategory,
};
use crate::operation::PreparedExecutionError;
use fixture::{ExampleOperation, FakeTransport, PendingTransport, endpoint, other_endpoint};

macro_rules! require_ok {
    ($expression:expr) => {{
        let result = $expression;
        assert!(result.is_ok());
        let Ok(value) = result else { return };
        value
    }};
}

macro_rules! require_some {
    ($expression:expr) => {{
        let result = $expression;
        assert!(result.is_some());
        let Some(value) = result else { return };
        value
    }};
}

#[test]
fn workspace_pool_rejects_invalid_bounds_and_reuses_released_slots() {
    assert!(matches!(
        ClientWorkspacePool::<0>::new(),
        Err(WorkspacePoolError::ZeroCapacity)
    ));
    assert!(matches!(
        ClientWorkspacePool::<129>::new(),
        Err(WorkspacePoolError::CapacityTooLarge)
    ));

    let pool = require_ok!(ClientWorkspacePool::<2>::new());
    let mut first = [[0xa5_u8; 128]; 4];
    let mut second = [[0xa5_u8; 128]; 4];
    let mut third = [[0xa5_u8; 128]; 4];
    let first_lease = require_ok!(pool.try_acquire(workspace(&mut first)));
    let second_lease = require_ok!(pool.try_acquire(workspace(&mut second)));
    assert!(matches!(
        pool.try_acquire(workspace(&mut third)),
        Err(WorkspaceAcquireError::Exhausted)
    ));
    assert_eq!(pool.active_leases(), 2);
    drop(first_lease);
    assert_eq!(pool.active_leases(), 1);
    let third_lease = require_ok!(pool.try_acquire(workspace(&mut third)));
    drop((second_lease, third_lease));
    assert_eq!(pool.active_leases(), 0);
    assert_cleared(&first);
    assert_cleared(&second);
    assert_cleared(&third);
}

#[test]
fn blocking_kernel_prepares_sends_decodes_and_clears() {
    let official = require_some!(endpoint());
    let transport = FakeTransport::success(official);
    let kernel = ClientKernel::new(transport);
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let mut storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut storage)));

    let decoded = require_ok!(kernel.execute_blocking(&ExampleOperation::read_only(), lease));
    assert_eq!(decoded.status, 200);
    assert_eq!(decoded.body_len, 2);
    assert!(!decoded.provider_error);
    assert_eq!(kernel.transport().calls(), 1);
    assert_eq!(pool.active_leases(), 0);
    assert_cleared(&storage);
}

#[test]
fn every_execution_mode_uses_the_same_error_decoder_and_cleanup() {
    let official = require_some!(endpoint());
    let kernel = ClientKernel::new(FakeTransport::provider_error(official));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());

    let mut blocking = [[0xa5_u8; 128]; 4];
    let blocking_lease = require_ok!(pool.try_acquire(workspace(&mut blocking)));
    let value =
        require_ok!(kernel.execute_blocking(&ExampleOperation::read_only(), blocking_lease));
    assert_eq!(value.status, 429);
    assert!(value.provider_error);
    assert_cleared(&blocking);

    let mut send_async = [[0xa5_u8; 128]; 4];
    let send_operation = ExampleOperation::read_only();
    let send_lease = require_ok!(pool.try_acquire(workspace(&mut send_async)));
    let future = kernel.execute_async(&send_operation, send_lease);
    assert_send(&future);
    let value = require_some!(ready(future));
    let value = require_ok!(value);
    assert_eq!(value.status, 429);
    assert!(value.provider_error);
    assert_cleared(&send_async);

    let mut local_async = [[0xa5_u8; 128]; 4];
    let local_lease = require_ok!(pool.try_acquire(workspace(&mut local_async)));
    let value = require_some!(ready(
        kernel.execute_local_async(&ExampleOperation::read_only(), local_lease,)
    ));
    let value = require_ok!(value);
    assert_eq!(value.status, 429);
    assert!(value.provider_error);
    assert_cleared(&local_async);
    assert_eq!(kernel.transport().calls(), 3);
}

#[test]
fn endpoint_and_auth_mismatch_fail_closed_and_clear() {
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let mismatched = ClientKernel::new(FakeTransport::success(require_some!(other_endpoint())));
    let mut endpoint_storage = [[0xa5_u8; 128]; 4];
    let endpoint_lease = require_ok!(pool.try_acquire(workspace(&mut endpoint_storage)));
    let result = mismatched.execute_blocking(&ExampleOperation::read_only(), endpoint_lease);
    assert!(matches!(
        result,
        Err(ClientExecutionError::Execution(
            PreparedExecutionError::EndpointMismatch
        ))
    ));
    assert_eq!(mismatched.transport().calls(), 0);
    assert_cleared(&endpoint_storage);

    let rejected = ClientKernel::new(FakeTransport::auth_mismatch(require_some!(endpoint())));
    let mut auth_storage = [[0xa5_u8; 128]; 4];
    let auth_lease = require_ok!(pool.try_acquire(workspace(&mut auth_storage)));
    let result = rejected.execute_blocking(&ExampleOperation::read_only(), auth_lease);
    assert!(matches!(
        result,
        Err(ClientExecutionError::Execution(
            PreparedExecutionError::Transport(_)
        ))
    ));
    assert_eq!(rejected.transport().calls(), 1);
    assert_cleared(&auth_storage);
}

#[test]
fn mutation_without_a_permit_never_reaches_transport() {
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(endpoint())));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let mut storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut storage)));
    let result = kernel.execute_blocking(&ExampleOperation::mutation(), lease);
    assert!(matches!(
        result,
        Err(ClientExecutionError::Execution(
            PreparedExecutionError::AuthorizationRequired
        ))
    ));
    assert_eq!(kernel.transport().calls(), 0);
    assert_cleared(&storage);
}

#[test]
fn cancelled_async_request_releases_slot_and_clears_every_buffer() {
    let kernel = ClientKernel::new(PendingTransport::new(require_some!(endpoint())));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let mut storage = [[0xa5_u8; 128]; 4];
    {
        let lease = require_ok!(pool.try_acquire(workspace(&mut storage)));
        let operation = ExampleOperation::read_only();
        let future = kernel.execute_async(&operation, lease);
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
        assert_eq!(pool.active_leases(), 1);
    }
    assert_eq!(pool.active_leases(), 0);
    assert_cleared(&storage);
}

struct NonDebugObserverError;

struct RecordingObserver {
    events: [AtomicU8; 24],
    len: AtomicUsize,
    invalid: AtomicBool,
    fail: bool,
}

impl RecordingObserver {
    const fn new(fail: bool) -> Self {
        Self {
            events: [const { AtomicU8::new(0) }; 24],
            len: AtomicUsize::new(0),
            invalid: AtomicBool::new(false),
            fail,
        }
    }

    fn events(&self) -> [u8; 24] {
        core::array::from_fn(|index| {
            self.events
                .get(index)
                .map(|event| event.load(Ordering::Acquire))
                .unwrap_or(0)
        })
    }
}

impl DiagnosticObserver for RecordingObserver {
    type Error = NonDebugObserverError;

    fn observe(&self, event: DiagnosticEvent) -> Result<(), Self::Error> {
        let (code, valid) = classify_event(event);
        if !valid {
            self.invalid.store(true, Ordering::Release);
        }
        let index = self.len.fetch_add(1, Ordering::AcqRel);
        if let Some(slot) = self.events.get(index) {
            slot.store(code, Ordering::Release);
        }
        if self.fail {
            Err(NonDebugObserverError)
        } else {
            Ok(())
        }
    }
}

fn classify_event(event: DiagnosticEvent) -> (u8, bool) {
    match event {
        DiagnosticEvent::PreparationStarted => (1, true),
        DiagnosticEvent::PreparationFailed { error } => {
            (6, error == DiagnosticErrorCategory::Preparation)
        }
        DiagnosticEvent::RequestPrepared { context } => (2, context_is_expected(context)),
        DiagnosticEvent::DispatchStarted { context } => (3, context_is_expected(context)),
        DiagnosticEvent::ExecutionFailed { context, error } => {
            let code = match error {
                DiagnosticErrorCategory::Authorization => 7,
                DiagnosticErrorCategory::Endpoint => 8,
                DiagnosticErrorCategory::Transport => 9,
                DiagnosticErrorCategory::ResponseTransaction => 10,
                DiagnosticErrorCategory::ResponsePolicy => 11,
                DiagnosticErrorCategory::Preparation | DiagnosticErrorCategory::Decode => 0,
            };
            (code, code != 0 && context_is_expected(context))
        }
        DiagnosticEvent::ResponseReceived { context, response } => (
            4,
            context_is_expected(context) && response_is_expected(response),
        ),
        DiagnosticEvent::DecodeFailed {
            context,
            response,
            error,
        } => (
            12,
            context_is_expected(context)
                && response.is_none_or(response_is_expected)
                && error == DiagnosticErrorCategory::Decode,
        ),
        DiagnosticEvent::Completed { context, response } => (
            5,
            context_is_expected(context) && response.is_none_or(response_is_expected),
        ),
    }
}

fn context_is_expected(context: crate::diagnostics::DiagnosticContext) -> bool {
    context.provider().as_str() == "example"
        && context.service().as_str() == "compute"
        && context.operation().map(|operation| operation.as_str()) == Some("list_servers")
        && context.retry() == DiagnosticRetryCategory::ExplicitPolicy
}

fn response_is_expected(response: crate::diagnostics::DiagnosticResponse) -> bool {
    response.status().get() == 200 && response.request_id() == DiagnosticRequestId::Discarded
}

#[test]
fn observed_execution_has_the_same_payload_free_sequence_in_every_mode() {
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(endpoint())));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let observer = RecordingObserver::new(false);

    let mut blocking = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut blocking)));
    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::read_only(), lease, &observer)
            .is_ok()
    );

    let mut send_async = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut send_async)));
    let operation = ExampleOperation::read_only();
    let future = kernel.execute_async_observed(&operation, lease, &observer);
    assert_send(&future);
    assert!(require_some!(ready(future)).is_ok());

    let mut local_async = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut local_async)));
    assert!(
        require_some!(ready(kernel.execute_local_async_observed(
            &ExampleOperation::read_only(),
            lease,
            &observer,
        )))
        .is_ok()
    );

    let events = observer.events();
    assert_eq!(observer.len.load(Ordering::Acquire), 15);
    assert!(!observer.invalid.load(Ordering::Acquire));
    for offset in [0_usize, 5, 10] {
        assert_eq!(
            events.get(offset..offset.saturating_add(5)),
            Some(&[1, 2, 3, 4, 5][..])
        );
    }
    assert_cleared(&blocking);
    assert_cleared(&send_async);
    assert_cleared(&local_async);
}

#[test]
fn observer_errors_never_change_execution_or_cleanup() {
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(endpoint())));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let observer = RecordingObserver::new(true);
    let mut storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut storage)));

    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::read_only(), lease, &observer)
            .is_ok()
    );
    assert_eq!(observer.len.load(Ordering::Acquire), 5);
    assert_cleared(&storage);
}

#[test]
fn observed_failures_report_only_structural_categories() {
    let pool = require_ok!(ClientWorkspacePool::<1>::new());

    let observer = RecordingObserver::new(false);
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(other_endpoint())));
    let mut endpoint_storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut endpoint_storage)));
    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::read_only(), lease, &observer)
            .is_err()
    );
    assert_eq!(&observer.events()[..4], &[1, 2, 3, 8]);
    assert!(!observer.invalid.load(Ordering::Acquire));
    assert_cleared(&endpoint_storage);

    let observer = RecordingObserver::new(false);
    let kernel = ClientKernel::new(FakeTransport::auth_mismatch(require_some!(endpoint())));
    let mut transport_storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut transport_storage)));
    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::read_only(), lease, &observer)
            .is_err()
    );
    assert_eq!(&observer.events()[..4], &[1, 2, 3, 9]);
    assert!(!observer.invalid.load(Ordering::Acquire));
    assert_cleared(&transport_storage);

    let observer = RecordingObserver::new(false);
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(endpoint())));
    let mut authorization_storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut authorization_storage)));
    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::mutation(), lease, &observer)
            .is_err()
    );
    assert_eq!(&observer.events()[..4], &[1, 2, 3, 7]);
    assert!(!observer.invalid.load(Ordering::Acquire));
    assert_cleared(&authorization_storage);
}

#[test]
fn observed_preparation_and_decode_failures_are_bounded_and_clear_storage() {
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(endpoint())));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());

    let observer = RecordingObserver::new(false);
    let mut target = [0xa5_u8; 4];
    let mut request = [0xa5_u8; 128];
    let mut response = [0xa5_u8; 128];
    let mut headers = [0xa5_u8; 128];
    let short_workspace =
        ClientWorkspace::new(&mut target, &mut request, &mut response, &mut headers);
    let lease = require_ok!(pool.try_acquire(short_workspace));
    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::read_only(), lease, &observer)
            .is_err()
    );
    assert_eq!(&observer.events()[..2], &[1, 6]);
    assert!(!observer.invalid.load(Ordering::Acquire));
    assert!(
        target
            .iter()
            .chain(request.iter())
            .chain(response.iter())
            .chain(headers.iter())
            .all(|byte| *byte == 0)
    );

    let observer = RecordingObserver::new(false);
    let mut storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut storage)));
    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::rejecting_decode(), lease, &observer)
            .is_err()
    );
    assert_eq!(&observer.events()[..5], &[1, 2, 3, 4, 12]);
    assert!(!observer.invalid.load(Ordering::Acquire));
    assert_cleared(&storage);
}

struct ReentrantObserver {
    inside: Cell<bool>,
    calls: Cell<usize>,
}

impl DiagnosticObserver for ReentrantObserver {
    type Error = core::convert::Infallible;

    fn observe(&self, event: DiagnosticEvent) -> Result<(), Self::Error> {
        self.calls.set(self.calls.get().saturating_add(1));
        if event == DiagnosticEvent::PreparationStarted && !self.inside.replace(true) {
            let _result = self.observe(DiagnosticEvent::PreparationFailed {
                error: DiagnosticErrorCategory::Preparation,
            });
            self.inside.set(false);
        }
        Ok(())
    }
}

#[test]
fn shared_observer_contract_permits_reentrant_callbacks_without_sdk_state() {
    let kernel = ClientKernel::new(FakeTransport::success(require_some!(endpoint())));
    let pool = require_ok!(ClientWorkspacePool::<1>::new());
    let observer = ReentrantObserver {
        inside: Cell::new(false),
        calls: Cell::new(0),
    };
    let mut storage = [[0xa5_u8; 128]; 4];
    let lease = require_ok!(pool.try_acquire(workspace(&mut storage)));

    assert!(
        kernel
            .execute_blocking_observed(&ExampleOperation::read_only(), lease, &observer)
            .is_ok()
    );
    assert_eq!(observer.calls.get(), 6);
    assert_cleared(&storage);
}

fn workspace(storage: &mut [[u8; 128]; 4]) -> ClientWorkspace<'_> {
    let [target, request, response, headers] = storage;
    ClientWorkspace::new(target, request, response, headers)
}

fn assert_cleared<const N: usize>(storage: &[[u8; N]; 4]) {
    assert!(storage.iter().flatten().all(|byte| *byte == 0));
}

fn assert_send<T: Send>(_: &T) {}

fn ready<F: Future>(future: F) -> Option<F::Output> {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match Future::poll(future.as_mut(), &mut context) {
        Poll::Ready(output) => Some(output),
        Poll::Pending => None,
    }
}
