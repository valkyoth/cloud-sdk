use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    CredentialDispatchGuard, LocalAsyncAuthenticatedTransport, OwnedCredentialAttempt,
};
use cloud_sdk::operation::{
    PermitClock, PermitDisposition, PermitExecutionError, PreparedExecutionError,
};
use cloud_sdk::transport::{BoundTransport, DeliveryClassified};

use super::construction::{RobotClient, RobotClientLifecycleError};
use super::operation::RobotClientOperation;
use crate::robot::*;

/// Failure from client-integrated execution of a request-bound Robot permit.
pub enum RobotPermitClientExecutionError<E> {
    /// Credential generation or binding validation failed.
    Lifecycle(RobotClientLifecycleError),
    /// Robot rejected Basic authentication and the generation was closed.
    AuthenticationRejected(PermitDisposition),
    /// The provider-specific permit execution failed for another reason.
    Permit(PermitExecutionError<E>),
}

impl<E> fmt::Debug for RobotPermitClientExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lifecycle(_) => "RobotPermitClientExecutionError::Lifecycle",
            Self::AuthenticationRejected(_) => {
                "RobotPermitClientExecutionError::AuthenticationRejected"
            }
            Self::Permit(_) => "RobotPermitClientExecutionError::Permit([redacted])",
        })
    }
}

impl<E> fmt::Display for RobotPermitClientExecutionError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Lifecycle(_) => "Robot credential lifecycle rejected permit execution",
            Self::AuthenticationRejected(_) => "Robot credentials were rejected",
            Self::Permit(_) => "Robot permit execution failed",
        })
    }
}

impl<E> core::error::Error for RobotPermitClientExecutionError<E> {}

/// One admitted Robot credential generation ready to consume one SDK permit.
///
/// Acquire this object before creating or beginning a mutation permit attempt.
/// A rejected generation therefore fails before permit authority is consumed.
#[must_use]
pub struct RobotClientAttempt<'client, T> {
    pub(super) client: &'client RobotClient<T>,
    pub(super) credential: OwnedCredentialAttempt,
}

impl<T> RobotClient<T>
where
    T: BoundCredentialTransport + BoundTransport,
{
    /// Begins one permit execution only on the current open credential generation.
    pub fn begin_permit_attempt(
        &self,
    ) -> Result<RobotClientAttempt<'_, T>, RobotClientLifecycleError> {
        if !self.transport().credential_binding().matches(self.binding) {
            return Err(RobotClientLifecycleError::CredentialBindingChanged);
        }
        let credential = self.attempts.begin()?;
        Ok(RobotClientAttempt {
            client: self,
            credential,
        })
    }
}

impl<T> RobotClientAttempt<'_, T>
where
    T: BoundCredentialTransport + BoundTransport,
{
    pub(super) fn reserve_for_dispatch(
        &self,
    ) -> Result<CredentialDispatchGuard<'_>, RobotClientLifecycleError> {
        if !self
            .client
            .transport()
            .credential_binding()
            .matches(self.client.binding)
        {
            return Err(RobotClientLifecycleError::CredentialBindingChanged);
        }
        self.client
            .attempts
            .reserve_dispatch(&self.credential)
            .map_err(Into::into)
    }

    pub(super) fn finish<R, E>(
        &self,
        dispatch: CredentialDispatchGuard<'_>,
        result: Result<R, PermitExecutionError<E>>,
    ) -> Result<R, RobotPermitClientExecutionError<E>>
    where
        E: DeliveryClassified,
    {
        if !self
            .client
            .transport()
            .credential_binding()
            .matches(self.client.binding)
        {
            return Err(RobotPermitClientExecutionError::Lifecycle(
                RobotClientLifecycleError::CredentialBindingChanged,
            ));
        }
        let authentication_rejected = result
            .as_ref()
            .err()
            .is_some_and(|error| execution_observed_unauthorized(error.execution()));
        let closes_generation = result
            .as_ref()
            .err()
            .is_some_and(|error| execution_is_indeterminate(error.execution()))
            || authentication_rejected;
        if closes_generation {
            dispatch
                .reject()
                .map_err(|error| RobotPermitClientExecutionError::Lifecycle(error.into()))?;
        }
        if authentication_rejected {
            let disposition = result
                .as_ref()
                .err()
                .map(PermitExecutionError::disposition)
                .unwrap_or_else(|| unreachable!("rejected Robot result lost its error"));
            return Err(RobotPermitClientExecutionError::AuthenticationRejected(
                disposition,
            ));
        }
        dispatch.complete();
        result.map_err(RobotPermitClientExecutionError::Permit)
    }
}

fn execution_observed_unauthorized<E: DeliveryClassified>(
    error: &PreparedExecutionError<E>,
) -> bool {
    match error {
        PreparedExecutionError::Transport(error) => error
            .observed_status()
            .is_some_and(|status| status.get() == 401),
        PreparedExecutionError::UnexpectedStatus(status) => status.get() == 401,
        _ => false,
    }
}

fn execution_is_indeterminate<E: DeliveryClassified>(error: &PreparedExecutionError<E>) -> bool {
    match error {
        PreparedExecutionError::Transport(error) => {
            error.delivery_phase() != cloud_sdk::transport::DeliveryPhase::NotSent
        }
        PreparedExecutionError::ResponseWriter(_) => true,
        _ => false,
    }
}

impl<T> fmt::Debug for RobotClientAttempt<'_, T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RobotClientAttempt")
            .field("client", &"[bound]")
            .field("credential", &"[redacted]")
            .finish()
    }
}

macro_rules! permit_family {
    (
        $bound:path, $attempt:ident, $checked:ident,
        $blocking:ident, $asynchronous:ident, $local:ident
    ) => {
        impl<'client, T> RobotClientAttempt<'client, T>
        where
            T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
            T::Error: DeliveryClassified,
        {
            #[doc = concat!("Executes one `", stringify!($attempt), "` synchronously.")]
            pub fn $blocking<'permit, 'storage, 'fingerprint, 'request, 'buffer, R, C>(
                self,
                attempt: $attempt<'permit, 'storage, 'fingerprint, 'request, R>,
                clock: &C,
                body: &'buffer mut [u8],
                headers: &'buffer mut [u8],
            ) -> Result<$checked<'buffer, 'request, R>, RobotPermitClientExecutionError<T::Error>>
            where
                R: $bound,
                C: PermitClock + ?Sized,
            {
                let dispatch = match self.reserve_for_dispatch() {
                    Ok(dispatch) => dispatch,
                    Err(error) => {
                        let _ = attempt.complete(cloud_sdk::transport::DeliveryPhase::NotSent);
                        return Err(RobotPermitClientExecutionError::Lifecycle(error));
                    }
                };
                let result =
                    attempt.execute_blocking(clock, self.client.transport(), body, headers);
                self.finish(dispatch, result)
            }
        }

        impl<'client, T> RobotClientAttempt<'client, T>
        where
            T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport + Sync,
            T::Error: DeliveryClassified + Send,
        {
            #[doc = concat!("Executes one `", stringify!($attempt), "` through a `Send` future.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $asynchronous<
                'transport,
                'permit,
                'storage,
                'fingerprint,
                'request,
                'buffer,
                R,
                C,
            >(
                self,
                attempt: $attempt<'permit, 'storage, 'fingerprint, 'request, R>,
                clock: &'transport C,
                body: &'buffer mut [u8],
                headers: &'buffer mut [u8],
            ) -> impl core::future::Future<
                Output = Result<
                    $checked<'buffer, 'request, R>,
                    RobotPermitClientExecutionError<T::Error>,
                >,
            > + Send
            + 'transport
            where
                R: $bound + Sync + 'transport,
                C: PermitClock + Sync + ?Sized,
                'client: 'transport,
                'permit: 'transport,
                'storage: 'transport,
                'fingerprint: 'transport,
                'request: 'transport,
                'buffer: 'transport,
            {
                async move {
                    let dispatch = match self.reserve_for_dispatch() {
                        Ok(dispatch) => dispatch,
                        Err(error) => {
                            let _ = attempt.complete(cloud_sdk::transport::DeliveryPhase::NotSent);
                            return Err(RobotPermitClientExecutionError::Lifecycle(error));
                        }
                    };
                    let result = attempt
                        .execute_async(clock, self.client.transport(), body, headers)
                        .await;
                    self.finish(dispatch, result)
                }
            }
        }

        impl<'client, T> RobotClientAttempt<'client, T>
        where
            T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
            T::Error: DeliveryClassified,
        {
            #[doc = concat!("Executes one `", stringify!($attempt), "` on a local executor.")]
            #[allow(clippy::manual_async_fn)]
            pub fn $local<'transport, 'permit, 'storage, 'fingerprint, 'request, 'buffer, R, C>(
                self,
                attempt: $attempt<'permit, 'storage, 'fingerprint, 'request, R>,
                clock: &'transport C,
                body: &'buffer mut [u8],
                headers: &'buffer mut [u8],
            ) -> impl core::future::Future<
                Output = Result<
                    $checked<'buffer, 'request, R>,
                    RobotPermitClientExecutionError<T::Error>,
                >,
            > + 'transport
            where
                R: $bound + 'transport,
                C: PermitClock + ?Sized,
                'client: 'transport,
                'permit: 'transport,
                'storage: 'transport,
                'fingerprint: 'transport,
                'request: 'transport,
                'buffer: 'transport,
            {
                async move {
                    let dispatch = match self.reserve_for_dispatch() {
                        Ok(dispatch) => dispatch,
                        Err(error) => {
                            let _ = attempt.complete(cloud_sdk::transport::DeliveryPhase::NotSent);
                            return Err(RobotPermitClientExecutionError::Lifecycle(error));
                        }
                    };
                    let result = attempt
                        .execute_local_async(clock, self.client.transport(), body, headers)
                        .await;
                    self.finish(dispatch, result)
                }
            }
        }
    };
}

permit_family!(
    RobotClientOperation,
    CancellationPermitAttempt,
    CheckedCancellation,
    execute_cancellation_blocking,
    execute_cancellation_async,
    execute_cancellation_local_async
);
permit_family!(
    RobotIpPermitRequest,
    RobotIpPermitAttempt,
    CheckedRobotIp,
    execute_ip_blocking,
    execute_ip_async,
    execute_ip_local_async
);
permit_family!(
    RobotSubnetPermitRequest,
    RobotSubnetPermitAttempt,
    CheckedRobotSubnet,
    execute_subnet_blocking,
    execute_subnet_async,
    execute_subnet_local_async
);
permit_family!(
    RobotResetPermitRequest,
    RobotResetPermitAttempt,
    CheckedRobotReset,
    execute_reset_blocking,
    execute_reset_async,
    execute_reset_local_async
);
permit_family!(
    RobotFailoverPermitRequest,
    RobotFailoverPermitAttempt,
    CheckedRobotFailover,
    execute_failover_blocking,
    execute_failover_async,
    execute_failover_local_async
);
permit_family!(
    RobotWolPermitRequest,
    RobotWolPermitAttempt,
    CheckedRobotWol,
    execute_wol_blocking,
    execute_wol_async,
    execute_wol_local_async
);
permit_family!(
    RobotRdnsPermitRequest,
    RobotRdnsPermitAttempt,
    CheckedRobotRdns,
    execute_rdns_blocking,
    execute_rdns_async,
    execute_rdns_local_async
);
permit_family!(
    RobotSshKeyPermitRequest,
    RobotSshKeyPermitAttempt,
    CheckedRobotSshKey,
    execute_ssh_key_blocking,
    execute_ssh_key_async,
    execute_ssh_key_local_async
);
permit_family!(
    RobotFirewallPermitRequest,
    RobotFirewallPermitAttempt,
    CheckedRobotFirewall,
    execute_firewall_blocking,
    execute_firewall_async,
    execute_firewall_local_async
);
permit_family!(
    RobotVSwitchPermitRequest,
    RobotVSwitchPermitAttempt,
    CheckedRobotVSwitch,
    execute_vswitch_blocking,
    execute_vswitch_async,
    execute_vswitch_local_async
);
permit_family!(
    RobotOrderPermitRequest,
    RobotOrderPermitAttempt,
    CheckedRobotOrderMutation,
    execute_order_blocking,
    execute_order_async,
    execute_order_local_async
);
