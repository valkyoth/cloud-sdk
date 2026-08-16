//! Credential-bound account evidence for Robot order authorization.

use core::fmt;

use cloud_sdk::authentication::CredentialBinding;
use cloud_sdk::operation::{PlanAuthorizationEvidence, PlanFingerprintBuildError};

use super::cost::RobotOrderAccount;
use super::permit::RobotOrderPermitRequest;

/// Account approval bound to one authenticated Robot credential lifecycle.
#[derive(Clone, Copy)]
pub struct RobotOrderAuthorizationEvidence<'a> {
    account: RobotOrderAccount<'a>,
    credential: CredentialBinding,
}

impl<'a> RobotOrderAuthorizationEvidence<'a> {
    /// Binds account approval to the credential-established observations in a request.
    #[must_use]
    pub fn for_request<R: RobotOrderPermitRequest + ?Sized>(
        account: RobotOrderAccount<'a>,
        request: &R,
    ) -> Self {
        Self {
            account,
            credential: request.credential_binding(),
        }
    }

    pub(super) const fn account_bytes(self) -> &'a [u8] {
        self.account.bytes()
    }

    pub(super) const fn credential(self) -> CredentialBinding {
        self.credential
    }
}

impl PlanAuthorizationEvidence for RobotOrderAuthorizationEvidence<'_> {
    fn encode<E: Copy>(
        &self,
        writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>> {
        writer.bytes(b"hetzner/robot/order-authorization/v1\0")?;
        encode_evidence_field(writer, 1, self.account.bytes())?;
        self.credential
            .with_bytes(|bytes| encode_evidence_field(writer, 2, bytes))
    }
}

impl fmt::Debug for RobotOrderAuthorizationEvidence<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RobotOrderAuthorizationEvidence([redacted])")
    }
}

fn encode_evidence_field<E: Copy>(
    writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    tag: u8,
    bytes: &[u8],
) -> Result<(), PlanFingerprintBuildError<E>> {
    writer.byte(tag)?;
    let len = u64::try_from(bytes.len()).map_err(|_| PlanFingerprintBuildError::InputTooLarge)?;
    writer.bytes(&len.to_be_bytes())?;
    writer.bytes(bytes)
}
