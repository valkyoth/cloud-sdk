use core::fmt;

use cloud_sdk::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, BoundCredentialTransport,
    LocalAsyncAuthenticatedTransport,
};
use cloud_sdk::operation::{PermitClock, PreparedExecutionError};
use cloud_sdk::transport::BoundTransport;

use super::{
    AuthorizedRobotWol, CheckedRobotWol, PreparedRobotWol, RobotWolDecodeError,
    RobotWolEvidenceError, RobotWolGetRequest,
};

/// Failure while executing and decoding an authorizing WOL discovery.
pub enum RobotWolPreflightError<E> {
    /// Authenticated read execution or response policy failed.
    Execution(PreparedExecutionError<E>),
    /// The exact discovery response failed strict provider decoding.
    Decode(RobotWolDecodeError),
    /// Short-lived capability evidence could not be admitted.
    Evidence(RobotWolEvidenceError),
}

impl<E> fmt::Debug for RobotWolPreflightError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Execution(_) => "RobotWolPreflightError::Execution([redacted])",
            Self::Decode(_) => "RobotWolPreflightError::Decode([redacted])",
            Self::Evidence(_) => "RobotWolPreflightError::Evidence([redacted])",
        })
    }
}

impl<E> fmt::Display for RobotWolPreflightError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Execution(_) => "Robot WOL preflight execution failed",
            Self::Decode(_) => "Robot WOL preflight response was rejected",
            Self::Evidence(_) => "Robot WOL preflight evidence was rejected",
        })
    }
}

impl<E> core::error::Error for RobotWolPreflightError<E> {}

impl<'storage, 'request> PreparedRobotWol<'storage, 'request, RobotWolGetRequest> {
    /// Executes discovery and mints short-lived WOL capability evidence.
    pub fn execute_authorizing_blocking<T, C>(
        self,
        clock: &C,
        transport: &T,
        response_body: &mut [u8],
        response_headers: &mut [u8],
    ) -> Result<AuthorizedRobotWol, RobotWolPreflightError<T::Error>>
    where
        T: BlockingAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        C: PermitClock + ?Sized,
    {
        let credential = transport.credential_binding();
        let checked = self
            .inner
            .execute_blocking(transport, response_body, response_headers)
            .map_err(RobotWolPreflightError::Execution)?;
        ensure_credential::<T::Error>(credential, transport.credential_binding())?;
        authorize(
            CheckedRobotWol::from_executed(self.request, checked),
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
    ) -> Result<AuthorizedRobotWol, RobotWolPreflightError<T::Error>>
    where
        T: AsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        C: PermitClock + Sync + ?Sized,
    {
        let credential = transport.credential_binding();
        let checked = self
            .inner
            .execute_async(transport, response_body, response_headers)
            .await
            .map_err(RobotWolPreflightError::Execution)?;
        ensure_credential::<T::Error>(credential, transport.credential_binding())?;
        authorize(
            CheckedRobotWol::from_executed(self.request, checked),
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
    ) -> Result<AuthorizedRobotWol, RobotWolPreflightError<T::Error>>
    where
        T: LocalAsyncAuthenticatedTransport + BoundCredentialTransport + BoundTransport,
        C: PermitClock + ?Sized,
    {
        let credential = transport.credential_binding();
        let checked = self
            .inner
            .execute_local_async(transport, response_body, response_headers)
            .await
            .map_err(RobotWolPreflightError::Execution)?;
        ensure_credential::<T::Error>(credential, transport.credential_binding())?;
        authorize(
            CheckedRobotWol::from_executed(self.request, checked),
            credential,
            clock,
        )
    }
}

fn ensure_credential<E>(
    before: cloud_sdk::authentication::CredentialBinding,
    after: cloud_sdk::authentication::CredentialBinding,
) -> Result<(), RobotWolPreflightError<E>> {
    if before.matches(after) {
        Ok(())
    } else {
        Err(RobotWolPreflightError::Evidence(
            RobotWolEvidenceError::CredentialChangedDuringPreflight,
        ))
    }
}

fn authorize<E, C>(
    checked: CheckedRobotWol<'_, '_, RobotWolGetRequest>,
    credential: cloud_sdk::authentication::CredentialBinding,
    clock: &C,
) -> Result<AuthorizedRobotWol, RobotWolPreflightError<E>>
where
    C: PermitClock + ?Sized,
{
    let wol = checked
        .decode_response()
        .map_err(RobotWolPreflightError::Decode)?;
    AuthorizedRobotWol::new(wol, credential, clock.now()).map_err(RobotWolPreflightError::Evidence)
}
