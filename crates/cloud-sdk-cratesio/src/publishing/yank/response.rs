use super::{YankError as Error, YankPermit, YankRequest};
use crate::{
    credentials::CredentialOrigin,
    discovery::value::Builder,
    endpoint::OfficialCratesIoEndpoint,
    versions::{VersionRecord, VersionRequest, VersionResponse},
    wire::JsonSuccess,
};
use cloud_sdk::incremental_json::IncrementalJsonProgress;

/// An `ok: true` acknowledgement, not proof of current state or index propagation.
/// It contains no credential and never authorizes replay.
#[derive(Debug)]
pub struct YankAcknowledgement<'a> {
    request: YankRequest<'a>,
    endpoint: OfficialCratesIoEndpoint,
}
/// Bound version snapshot from an explicit read-back, not a concurrency guarantee.
#[derive(Debug)]
pub struct YankObservation {
    requested: bool,
    observed: bool,
    record: VersionRecord,
}
impl YankObservation {
    /// Actual `yanked` value in the returned snapshot.
    pub const fn observed_yanked(&self) -> bool {
        self.observed
    }
    /// False can indicate stale data or an intervening writer; never retry implicitly.
    pub const fn matches_requested_state(&self) -> bool {
        self.requested == self.observed
    }
    /// Complete protected metadata for this exact crate and version.
    pub const fn record(&self) -> &VersionRecord {
        &self.record
    }
}
impl<'a> YankPermit<'a> {
    /// Consume consent and decode the wire-admitted response from its trusted
    /// exchange. The server echoes no IDs: dispatch association is caller-owned.
    pub fn decode_response(
        self,
        success: JsonSuccess<'_>,
    ) -> Result<YankAcknowledgement<'a>, Error> {
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let root = builder.finish()?;
        if !root.required("ok")?.boolean()? {
            return Err(Error::Binding);
        }
        Ok(YankAcknowledgement {
            request: self.request,
            endpoint: match self.credential.origin() {
                CredentialOrigin::Production => OfficialCratesIoEndpoint::production_api(),
                CredentialOrigin::Staging => OfficialCratesIoEndpoint::staging_api(),
            },
        })
    }
}
impl<'a> YankAcknowledgement<'a> {
    /// Intended value only; use explicit read-back for an observed snapshot.
    pub const fn requested_yanked(&self) -> bool {
        self.request.operation.requested_yanked()
    }
    /// Read-back must use this same official authority, without sending credentials.
    pub const fn verification_endpoint(&self) -> OfficialCratesIoEndpoint {
        self.endpoint
    }
    /// Construct an explicit exact-version GET. No hidden polling or index fetches.
    pub fn verification_request(&self) -> VersionRequest<'a> {
        VersionRequest::detail(self.request.name, self.request.version)
    }
    /// Decode a wire-admitted response from the verification request at the same
    /// authority. Wrong crate/version identities fail closed. A mismatching state
    /// is exposed explicitly, not mistaken for confirmation or replay authority.
    pub fn decode_observed_state(
        &self,
        success: JsonSuccess<'_>,
    ) -> Result<YankObservation, Error> {
        let VersionResponse::Version(record) =
            self.verification_request().decode(self.endpoint, success)?
        else {
            return Err(Error::Binding);
        };
        let observed = record.fields().required("yanked")?.boolean()?;
        Ok(YankObservation {
            requested: self.requested_yanked(),
            observed,
            record,
        })
    }
}
