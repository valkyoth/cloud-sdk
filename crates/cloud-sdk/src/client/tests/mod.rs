mod fixture;

use core::future::Future;
use core::task::{Context, Poll, Waker};

use super::{
    ClientExecutionError, ClientKernel, ClientWorkspace, ClientWorkspacePool,
    WorkspaceAcquireError, WorkspacePoolError,
};
use crate::operation::PreparedExecutionError;
use fixture::{ExampleOperation, FakeTransport, PendingTransport, endpoint, other_endpoint};

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

    let pool = ClientWorkspacePool::<2>::new().expect("bounded pool");
    let mut first = [[0xa5_u8; 128]; 4];
    let mut second = [[0xa5_u8; 128]; 4];
    let mut third = [[0xa5_u8; 128]; 4];
    let first_lease = pool
        .try_acquire(workspace(&mut first))
        .expect("first lease");
    let second_lease = pool
        .try_acquire(workspace(&mut second))
        .expect("second lease");
    assert!(matches!(
        pool.try_acquire(workspace(&mut third)),
        Err(WorkspaceAcquireError::Exhausted)
    ));
    assert_eq!(pool.active_leases(), 2);
    drop(first_lease);
    assert_eq!(pool.active_leases(), 1);
    let third_lease = pool
        .try_acquire(workspace(&mut third))
        .expect("released slot");
    drop((second_lease, third_lease));
    assert_eq!(pool.active_leases(), 0);
    assert_cleared(&first);
    assert_cleared(&second);
    assert_cleared(&third);
}

#[test]
fn blocking_kernel_prepares_sends_decodes_and_clears() {
    let official = endpoint();
    let transport = FakeTransport::success(official);
    let kernel = ClientKernel::new(transport);
    let pool = ClientWorkspacePool::<1>::new().expect("pool");
    let mut storage = [[0xa5_u8; 128]; 4];
    let lease = pool.try_acquire(workspace(&mut storage)).expect("lease");

    let decoded = kernel
        .execute_blocking(&ExampleOperation::read_only(), lease)
        .expect("checked success");
    assert_eq!(decoded.status, 200);
    assert_eq!(decoded.body_len, 2);
    assert!(!decoded.provider_error);
    assert_eq!(kernel.transport().calls(), 1);
    assert_eq!(pool.active_leases(), 0);
    assert_cleared(&storage);
}

#[test]
fn every_execution_mode_uses_the_same_error_decoder_and_cleanup() {
    let official = endpoint();
    let kernel = ClientKernel::new(FakeTransport::provider_error(official));
    let pool = ClientWorkspacePool::<1>::new().expect("pool");

    let mut blocking = [[0xa5_u8; 128]; 4];
    let value = kernel
        .execute_blocking(
            &ExampleOperation::read_only(),
            pool.try_acquire(workspace(&mut blocking)).expect("lease"),
        )
        .expect("blocking error envelope");
    assert_eq!(value.status, 429);
    assert!(value.provider_error);
    assert_cleared(&blocking);

    let mut send_async = [[0xa5_u8; 128]; 4];
    let send_operation = ExampleOperation::read_only();
    let future = kernel.execute_async(
        &send_operation,
        pool.try_acquire(workspace(&mut send_async)).expect("lease"),
    );
    assert_send(&future);
    let value = ready(future).expect("Send async error envelope");
    assert_eq!(value.status, 429);
    assert!(value.provider_error);
    assert_cleared(&send_async);

    let mut local_async = [[0xa5_u8; 128]; 4];
    let value = ready(
        kernel.execute_local_async(
            &ExampleOperation::read_only(),
            pool.try_acquire(workspace(&mut local_async))
                .expect("lease"),
        ),
    )
    .expect("local async error envelope");
    assert_eq!(value.status, 429);
    assert!(value.provider_error);
    assert_cleared(&local_async);
    assert_eq!(kernel.transport().calls(), 3);
}

#[test]
fn endpoint_and_auth_mismatch_fail_closed_and_clear() {
    let pool = ClientWorkspacePool::<1>::new().expect("pool");
    let mismatched = ClientKernel::new(FakeTransport::success(other_endpoint()));
    let mut endpoint_storage = [[0xa5_u8; 128]; 4];
    let result = mismatched.execute_blocking(
        &ExampleOperation::read_only(),
        pool.try_acquire(workspace(&mut endpoint_storage))
            .expect("lease"),
    );
    assert!(matches!(
        result,
        Err(ClientExecutionError::Execution(
            PreparedExecutionError::EndpointMismatch
        ))
    ));
    assert_eq!(mismatched.transport().calls(), 0);
    assert_cleared(&endpoint_storage);

    let rejected = ClientKernel::new(FakeTransport::auth_mismatch(endpoint()));
    let mut auth_storage = [[0xa5_u8; 128]; 4];
    let result = rejected.execute_blocking(
        &ExampleOperation::read_only(),
        pool.try_acquire(workspace(&mut auth_storage))
            .expect("lease"),
    );
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
    let kernel = ClientKernel::new(FakeTransport::success(endpoint()));
    let pool = ClientWorkspacePool::<1>::new().expect("pool");
    let mut storage = [[0xa5_u8; 128]; 4];
    let result = kernel.execute_blocking(
        &ExampleOperation::mutation(),
        pool.try_acquire(workspace(&mut storage)).expect("lease"),
    );
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
    let kernel = ClientKernel::new(PendingTransport::new(endpoint()));
    let pool = ClientWorkspacePool::<1>::new().expect("pool");
    let mut storage = [[0xa5_u8; 128]; 4];
    {
        let lease = pool.try_acquire(workspace(&mut storage)).expect("lease");
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

fn workspace(storage: &mut [[u8; 128]; 4]) -> ClientWorkspace<'_> {
    let [target, request, response, headers] = storage;
    ClientWorkspace::new(target, request, response, headers)
}

fn assert_cleared<const N: usize>(storage: &[[u8; N]; 4]) {
    assert!(storage.iter().flatten().all(|byte| *byte == 0));
}

fn assert_send<T: Send>(_: &T) {}

fn ready<F: Future>(future: F) -> F::Output {
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    match Future::poll(future.as_mut(), &mut context) {
        Poll::Ready(output) => output,
        Poll::Pending => panic!("fixture future unexpectedly pending"),
    }
}
