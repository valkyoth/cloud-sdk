use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::PermitClock;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified};

use super::{RobotClientMutationOperation, RobotMutationPermitAttempt};
use crate::client::robot::{RobotClientAttempt, RobotPermitClientExecutionError};

/// Credential, permit, transport, or typed-decoding failure for one mutation.
pub enum RobotMutationClientExecutionError<E, D> {
    /// Robot credential lifecycle or permit dispatch failed.
    Permit(RobotPermitClientExecutionError<E>),
    /// A checked success response failed operation-specific decoding.
    Decode(D),
}

impl<E, D> fmt::Debug for RobotMutationClientExecutionError<E, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Permit(_) => "RobotMutationClientExecutionError::Permit([redacted])",
            Self::Decode(_) => "RobotMutationClientExecutionError::Decode([redacted])",
        })
    }
}

impl<E, D> fmt::Display for RobotMutationClientExecutionError<E, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Permit(_) => "Robot mutation permit execution failed",
            Self::Decode(_) => "Robot mutation success response decoding failed",
        })
    }
}

impl<E, D> core::error::Error for RobotMutationClientExecutionError<E, D> {}

type MutationResult<'request, R, E> = Result<
    <R as crate::client::RobotClientOperation>::Output<'request>,
    RobotMutationClientExecutionError<E, <R as crate::client::RobotClientOperation>::SuccessError>,
>;

impl<T> RobotClientAttempt<'_, T>
where
    T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
    T::Error: DeliveryClassified,
{
    /// Executes and typed-decodes one generic Robot mutation synchronously.
    pub fn execute_mutation_blocking<'request, R, C>(
        self,
        attempt: RobotMutationPermitAttempt<'_, '_, '_, 'request, R>,
        clock: &C,
        body: &mut [u8],
        headers: &mut [u8],
    ) -> MutationResult<'request, R, T::Error>
    where
        R: RobotClientMutationOperation,
        C: PermitClock + ?Sized,
    {
        let dispatch = match self.reserve_for_dispatch() {
            Ok(dispatch) => dispatch,
            Err(error) => {
                let _ = attempt.complete(cloud_sdk::transport::DeliveryPhase::NotSent);
                return Err(RobotMutationClientExecutionError::Permit(
                    RobotPermitClientExecutionError::Lifecycle(error),
                ));
            }
        };
        let request = attempt.binding.0;
        let binding = self.client.binding;
        let result = attempt
            .inner
            .execute_blocking(clock, self.client.transport(), body, headers);
        let checked = self
            .finish(dispatch, result)
            .map_err(RobotMutationClientExecutionError::Permit)?;
        request
            .decode_success(checked, binding)
            .map_err(RobotMutationClientExecutionError::Decode)
    }
}

impl<'client, T> RobotClientAttempt<'client, T>
where
    T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport + Sync,
    T::Error: DeliveryClassified + Send,
{
    /// Executes and typed-decodes one generic Robot mutation in a `Send` future.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_mutation_async<'transport, 'permit, 'storage, 'fingerprint, 'request, R, C>(
        self,
        attempt: RobotMutationPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>,
        clock: &'transport C,
        body: &'transport mut [u8],
        headers: &'transport mut [u8],
    ) -> impl core::future::Future<Output = MutationResult<'request, R, T::Error>> + Send + 'transport
    where
        R: RobotClientMutationOperation + Sync + 'transport,
        R::Output<'request>: Send,
        R::SuccessError: Send,
        C: PermitClock + Sync + ?Sized,
        'client: 'transport,
        'permit: 'transport,
        'storage: 'transport,
        'fingerprint: 'transport,
        'request: 'transport,
    {
        async move {
            let dispatch = match self.reserve_for_dispatch() {
                Ok(dispatch) => dispatch,
                Err(error) => {
                    let _ = attempt.complete(cloud_sdk::transport::DeliveryPhase::NotSent);
                    return Err(RobotMutationClientExecutionError::Permit(
                        RobotPermitClientExecutionError::Lifecycle(error),
                    ));
                }
            };
            let request = attempt.binding.0;
            let binding = self.client.binding;
            let result = attempt
                .inner
                .execute_async(clock, self.client.transport(), body, headers)
                .await;
            let checked = self
                .finish(dispatch, result)
                .map_err(RobotMutationClientExecutionError::Permit)?;
            request
                .decode_success(checked, binding)
                .map_err(RobotMutationClientExecutionError::Decode)
        }
    }
}

impl<'client, T> RobotClientAttempt<'client, T>
where
    T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
    T::Error: DeliveryClassified,
{
    /// Executes and typed-decodes one generic Robot mutation locally.
    pub async fn execute_mutation_local_async<
        'transport,
        'permit,
        'storage,
        'fingerprint,
        'request,
        R,
        C,
    >(
        self,
        attempt: RobotMutationPermitAttempt<'permit, 'storage, 'fingerprint, 'request, R>,
        clock: &'transport C,
        body: &'transport mut [u8],
        headers: &'transport mut [u8],
    ) -> MutationResult<'request, R, T::Error>
    where
        R: RobotClientMutationOperation + 'transport,
        C: PermitClock + ?Sized,
        'client: 'transport,
        'permit: 'transport,
        'storage: 'transport,
        'fingerprint: 'transport,
        'request: 'transport,
    {
        let dispatch = match self.reserve_for_dispatch() {
            Ok(dispatch) => dispatch,
            Err(error) => {
                let _ = attempt.complete(cloud_sdk::transport::DeliveryPhase::NotSent);
                return Err(RobotMutationClientExecutionError::Permit(
                    RobotPermitClientExecutionError::Lifecycle(error),
                ));
            }
        };
        let request = attempt.binding.0;
        let binding = self.client.binding;
        let result = attempt
            .inner
            .execute_local_async(clock, self.client.transport(), body, headers)
            .await;
        let checked = self
            .finish(dispatch, result)
            .map_err(RobotMutationClientExecutionError::Permit)?;
        request
            .decode_success(checked, binding)
            .map_err(RobotMutationClientExecutionError::Decode)
    }
}
