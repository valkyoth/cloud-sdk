use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    CredentialDispatchGuard, LocalAsyncAuthenticatedTransport, OwnedCredentialAttempt,
};
use cloud_sdk::client::{ClientExecutionError, ClientWorkspaceLease};
use cloud_sdk::operation::PreparedExecutionError;
use cloud_sdk::transport::{BoundTransport, DeliveryClassified, DeliveryPhase};

use super::construction::{RobotClient, RobotClientLifecycleError};
use super::operation::{
    RobotClientOperation, RobotClientResponse, RobotDirectClientOperation,
    RobotResponseDecodeError, decode_response,
};
use crate::robot::RobotFailureCategory;

/// Robot client lifecycle or provider-neutral execution failure.
pub enum RobotClientExecutionError<P, T, D> {
    /// Credential lockout, replacement, or binding validation failed.
    Lifecycle(RobotClientLifecycleError),
    /// Preparation, transport, response policy, or checked decoding failed.
    Execution(ClientExecutionError<P, T, RobotResponseDecodeError<D>>),
}

impl<P, T, D> fmt::Debug for RobotClientExecutionError<P, T, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lifecycle(_) => "RobotClientExecutionError::Lifecycle",
            Self::Execution(_) => "RobotClientExecutionError::Execution([redacted])",
        })
    }
}

impl<P, T, D> fmt::Display for RobotClientExecutionError<P, T, D> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lifecycle(_) => "Robot credential lifecycle rejected execution",
            Self::Execution(_) => "Robot operation execution failed",
        })
    }
}

impl<P, T, D> core::error::Error for RobotClientExecutionError<P, T, D> {}

type RobotResult<'operation, O, T> = Result<
    RobotClientResponse<<O as RobotClientOperation>::Output<'operation>>,
    RobotClientExecutionError<
        <O as RobotDirectClientOperation>::PreparationError,
        T,
        <O as RobotClientOperation>::SuccessError,
    >,
>;

impl<T> RobotClient<T>
where
    T: BoundCredentialTransport + BoundTransport,
{
    fn begin(&self) -> Result<OwnedCredentialAttempt, RobotClientLifecycleError> {
        self.require_stable_binding()?;
        self.attempts.begin().map_err(Into::into)
    }

    fn require_stable_binding(&self) -> Result<(), RobotClientLifecycleError> {
        if self.transport().credential_binding().matches(self.binding) {
            Ok(())
        } else {
            Err(RobotClientLifecycleError::CredentialBindingChanged)
        }
    }

    fn reserve_attempt<'client>(
        &'client self,
        attempt: &OwnedCredentialAttempt,
    ) -> Result<CredentialDispatchGuard<'client>, RobotClientLifecycleError> {
        self.require_stable_binding()?;
        self.attempts.reserve_dispatch(attempt).map_err(Into::into)
    }

    fn finish<'operation, O, E>(
        &self,
        dispatch: CredentialDispatchGuard<'_>,
        result: RobotResult<'operation, O, E>,
    ) -> RobotResult<'operation, O, E>
    where
        O: RobotDirectClientOperation + 'operation,
        E: DeliveryClassified,
    {
        if let Err(error) = self.require_stable_binding() {
            return Err(RobotClientExecutionError::Lifecycle(error));
        }
        let rejected = match &result {
            Ok(RobotClientResponse::Failure(failure)) => {
                failure.category() == RobotFailureCategory::AuthenticationRejected
            }
            Err(RobotClientExecutionError::Execution(ClientExecutionError::Decode(error))) => {
                error.closes_credential_generation()
            }
            Err(RobotClientExecutionError::Execution(ClientExecutionError::Execution(error))) => {
                execution_closes_generation(error)
            }
            Ok(RobotClientResponse::Success(_))
            | Err(RobotClientExecutionError::Lifecycle(_))
            | Err(RobotClientExecutionError::Execution(ClientExecutionError::Preparation(_))) => {
                false
            }
        };
        if rejected && let Err(error) = dispatch.reject() {
            return Err(RobotClientExecutionError::Lifecycle(error.into()));
        }
        dispatch.complete();
        result
    }
}

fn execution_closes_generation<E: DeliveryClassified>(error: &PreparedExecutionError<E>) -> bool {
    match error {
        PreparedExecutionError::Transport(error) => {
            error
                .observed_status()
                .is_some_and(|status| status.get() == 401)
                || error.delivery_phase() != DeliveryPhase::NotSent
        }
        PreparedExecutionError::ResponseWriter(_) => true,
        PreparedExecutionError::UnexpectedStatus(status) => status.get() == 401,
        PreparedExecutionError::AuthorizationRequired
        | PreparedExecutionError::AuthorizationInvalid(_)
        | PreparedExecutionError::EndpointIdentity(_)
        | PreparedExecutionError::EndpointMismatch
        | PreparedExecutionError::ResponsePolicy(_) => false,
    }
}

impl<T> RobotClient<T>
where
    T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
    T::Error: DeliveryClassified,
{
    /// Sends one sealed read-only Robot operation synchronously and exactly once.
    pub fn execute_blocking<'operation, O, const N: usize>(
        &self,
        operation: &'operation O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> RobotResult<'operation, O, T::Error>
    where
        O: RobotDirectClientOperation,
    {
        let attempt = self.begin().map_err(RobotClientExecutionError::Lifecycle)?;
        let dispatch = self
            .reserve_attempt(&attempt)
            .map_err(RobotClientExecutionError::Lifecycle)?;
        let binding = self.binding;
        let result = self
            .kernel
            .execute_blocking_with(
                operation,
                lease,
                O::prepare_client,
                |operation, response| decode_response(operation, response, binding),
            )
            .map_err(RobotClientExecutionError::Execution);
        self.finish::<O, T::Error>(dispatch, result)
    }
}

impl<T> RobotClient<T>
where
    T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport + Sync,
    T::Error: DeliveryClassified,
{
    /// Sends one sealed read-only Robot operation through a `Send` future.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<'operation, O, const N: usize>(
        &'operation self,
        operation: &'operation O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> impl core::future::Future<Output = RobotResult<'operation, O, T::Error>> + Send
    where
        O: RobotDirectClientOperation + Sync,
        O::PreparationError: Send,
        O::Output<'operation>: Send,
        O::SuccessError: Send,
        T::Error: Send,
    {
        async move {
            let attempt = self.begin().map_err(RobotClientExecutionError::Lifecycle)?;
            let dispatch = self
                .reserve_attempt(&attempt)
                .map_err(RobotClientExecutionError::Lifecycle)?;
            let binding = self.binding;
            let result = self
                .kernel
                .execute_async_with(
                    operation,
                    lease,
                    O::prepare_client,
                    |operation, response| decode_response(operation, response, binding),
                )
                .await
                .map_err(RobotClientExecutionError::Execution);
            self.finish::<O, T::Error>(dispatch, result)
        }
    }
}

impl<T> RobotClient<T>
where
    T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
    T::Error: DeliveryClassified,
{
    /// Sends one sealed read-only Robot operation through a local future.
    pub async fn execute_local_async<'operation, O, const N: usize>(
        &'operation self,
        operation: &'operation O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> RobotResult<'operation, O, T::Error>
    where
        O: RobotDirectClientOperation,
    {
        let attempt = self.begin().map_err(RobotClientExecutionError::Lifecycle)?;
        let dispatch = self
            .reserve_attempt(&attempt)
            .map_err(RobotClientExecutionError::Lifecycle)?;
        let binding = self.binding;
        let result = self
            .kernel
            .execute_local_async_with(
                operation,
                lease,
                O::prepare_client,
                |operation, response| decode_response(operation, response, binding),
            )
            .await
            .map_err(RobotClientExecutionError::Execution);
        self.finish::<O, T::Error>(dispatch, result)
    }
}

#[cfg(test)]
mod tests {
    use cloud_sdk::operation::PreparedExecutionError;
    use cloud_sdk::transport::{StatusCode, TransportFailure};

    use super::execution_closes_generation;

    #[test]
    fn response_started_and_observed_unauthorized_fail_closed() {
        let unauthorized = StatusCode::new(401).unwrap_or(StatusCode::OK);
        let observed = PreparedExecutionError::Transport(
            TransportFailure::response_started_with_status(unauthorized, ()),
        );
        let uncertain = PreparedExecutionError::Transport(TransportFailure::possibly_sent(()));
        let not_sent = PreparedExecutionError::Transport(TransportFailure::not_sent(()));

        assert!(execution_closes_generation(&observed));
        assert!(execution_closes_generation(&uncertain));
        assert!(!execution_closes_generation(&not_sent));
    }
}
