use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{PermitClock, PreparedExecutionError};
use cloud_sdk::transport::BoundTransport;

use super::{
    AuthorizedRobotReset, CheckedRobotReset, PreparedRobotReset, RobotResetDecodeError,
    RobotResetEvidenceError, RobotResetGetRequest,
};

/// Failure while executing and decoding an authorizing reset preflight.
pub enum RobotResetPreflightError<E> {
    /// Authenticated read execution or response policy failed.
    Execution(PreparedExecutionError<E>),
    /// The exact detail response failed strict provider decoding.
    Decode(RobotResetDecodeError),
    /// The short-lived observation window could not be represented.
    Evidence(RobotResetEvidenceError),
}

impl<E> fmt::Debug for RobotResetPreflightError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Execution(_) => "RobotResetPreflightError::Execution([redacted])",
            Self::Decode(_) => "RobotResetPreflightError::Decode([redacted])",
            Self::Evidence(_) => "RobotResetPreflightError::Evidence([redacted])",
        })
    }
}

impl<E> fmt::Display for RobotResetPreflightError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Execution(_) => "Robot reset preflight execution failed",
            Self::Decode(_) => "Robot reset preflight response was rejected",
            Self::Evidence(_) => "Robot reset preflight evidence was rejected",
        })
    }
}

impl<E> core::error::Error for RobotResetPreflightError<E> {}

impl<'storage, 'request> PreparedRobotReset<'storage, 'request, RobotResetGetRequest> {
    /// Executes the exact detail request and mints short-lived reset authority.
    pub fn execute_authorizing_blocking<T, C>(
        self,
        clock: &C,
        transport: &T,
        response_body: &mut [u8],
        response_headers: &mut [u8],
    ) -> Result<AuthorizedRobotReset, RobotResetPreflightError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        C: PermitClock + ?Sized,
    {
        let credential = transport.credential_binding();
        let checked = self
            .inner
            .execute_blocking(transport, response_body, response_headers)
            .map_err(RobotResetPreflightError::Execution)?;
        if !credential.matches(transport.credential_binding()) {
            return Err(RobotResetPreflightError::Evidence(
                RobotResetEvidenceError::CredentialChangedDuringPreflight,
            ));
        }
        authorize(
            CheckedRobotReset::from_executed(self.request, checked),
            credential,
            clock,
        )
    }

    /// Send-async equivalent of [`Self::execute_authorizing_blocking`].
    pub async fn execute_authorizing_async<T, C>(
        self,
        clock: &C,
        transport: &T,
        response_body: &mut [u8],
        response_headers: &mut [u8],
    ) -> Result<AuthorizedRobotReset, RobotResetPreflightError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        C: PermitClock + Sync + ?Sized,
    {
        let credential = transport.credential_binding();
        let checked = self
            .inner
            .execute_async(transport, response_body, response_headers)
            .await
            .map_err(RobotResetPreflightError::Execution)?;
        if !credential.matches(transport.credential_binding()) {
            return Err(RobotResetPreflightError::Evidence(
                RobotResetEvidenceError::CredentialChangedDuringPreflight,
            ));
        }
        authorize(
            CheckedRobotReset::from_executed(self.request, checked),
            credential,
            clock,
        )
    }

    /// Local-async equivalent for single-threaded executors.
    pub async fn execute_authorizing_local_async<T, C>(
        self,
        clock: &C,
        transport: &T,
        response_body: &mut [u8],
        response_headers: &mut [u8],
    ) -> Result<AuthorizedRobotReset, RobotResetPreflightError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        C: PermitClock + ?Sized,
    {
        let credential = transport.credential_binding();
        let checked = self
            .inner
            .execute_local_async(transport, response_body, response_headers)
            .await
            .map_err(RobotResetPreflightError::Execution)?;
        if !credential.matches(transport.credential_binding()) {
            return Err(RobotResetPreflightError::Evidence(
                RobotResetEvidenceError::CredentialChangedDuringPreflight,
            ));
        }
        authorize(
            CheckedRobotReset::from_executed(self.request, checked),
            credential,
            clock,
        )
    }
}

fn authorize<E, C>(
    checked: CheckedRobotReset<'_, '_, RobotResetGetRequest>,
    credential: cloud_sdk::authentication::CredentialBinding,
    clock: &C,
) -> Result<AuthorizedRobotReset, RobotResetPreflightError<E>>
where
    C: PermitClock + ?Sized,
{
    let reset = checked
        .decode_response()
        .map_err(RobotResetPreflightError::Decode)?;
    AuthorizedRobotReset::new(reset, credential, clock.now())
        .map_err(RobotResetPreflightError::Evidence)
}
