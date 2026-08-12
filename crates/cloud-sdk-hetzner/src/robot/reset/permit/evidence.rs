use cloud_sdk::authentication::BoundCredentialTransport;
use cloud_sdk::operation::{
    ExecutionPermitError, PermitClock, PermitTimestamp, PlanAuthorizationEvidence,
    PlanFingerprintBuildError,
};

use super::super::RobotResetExecuteRequest;

mod sealed {
    pub trait Sealed {}
}

/// Sealed destructive Reset request carrying authenticated authorization evidence.
pub trait RobotResetPermitRequest: sealed::Sealed + PlanAuthorizationEvidence {
    /// Rechecks preflight freshness and transport credential lineage at dispatch.
    fn validate_authorization_evidence<T: BoundCredentialTransport>(
        &self,
        transport: &T,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError>;
}

impl sealed::Sealed for RobotResetExecuteRequest<'_> {}

impl PlanAuthorizationEvidence for RobotResetExecuteRequest<'_> {
    fn valid_until(&self) -> Option<PermitTimestamp> {
        Some(self.reset.expires_at())
    }

    fn encode<E: Copy>(
        &self,
        writer: &mut cloud_sdk::buffer::SnapshotEncoder<'_, PlanFingerprintBuildError<E>>,
    ) -> Result<(), PlanFingerprintBuildError<E>> {
        writer.bytes(b"hetzner/robot/reset-evidence/v1\0")?;
        let summary = self.reset.reset().summary();
        summary.with_server_ipv4(|address| encode_evidence_field(writer, 1, &address.octets()))?;
        summary.with_server_ipv6_network(|address| {
            encode_evidence_field(writer, 2, &address.octets())
        })?;
        summary
            .number()
            .with_decimal_bytes(|number| encode_evidence_field(writer, 3, number))?;
        encode_evidence_field(writer, 4, self.intent.reset_type().wire().as_bytes())?;
        encode_evidence_field(
            writer,
            5,
            &self.reset.observed_at().as_seconds().to_be_bytes(),
        )?;
        encode_evidence_field(
            writer,
            6,
            &self.reset.expires_at().as_seconds().to_be_bytes(),
        )?;
        self.reset
            .credential()
            .with_bytes(|binding| encode_evidence_field(writer, 7, binding))
    }
}

impl RobotResetPermitRequest for RobotResetExecuteRequest<'_> {
    fn validate_authorization_evidence<T: BoundCredentialTransport>(
        &self,
        transport: &T,
        now: PermitTimestamp,
    ) -> Result<(), ExecutionPermitError> {
        self.reset.validate_at(transport.credential_binding(), now)
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

pub(super) struct SampledPermitClock(pub(super) PermitTimestamp);

impl PermitClock for SampledPermitClock {
    fn now(&self) -> PermitTimestamp {
        self.0
    }
}
