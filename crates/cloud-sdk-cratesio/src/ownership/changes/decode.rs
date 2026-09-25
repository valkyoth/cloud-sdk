use super::{OwnerChangeError as Error, OwnerChangeOperation, OwnerChangePermit};
use crate::{
    discovery::{DiscoveryValue, value::Builder},
    wire::JsonSuccess,
};
use cloud_sdk::incremental_json::IncrementalJsonProgress;

/// Acknowledgements, not per-owner outcomes inferred from human-readable text.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnershipOutcome {
    /// Users may have new or already-pending invitations; teams may be added.
    /// Does not mean a user accepted or that notification email was delivered.
    AdditionAcknowledged,
    /// Removal acknowledged, not a new authoritative owner-list snapshot.
    RemovalAcknowledged,
}
/// Protected source message bound to the exact trusted request exchange.
#[derive(Debug)]
pub struct OwnerChangeResponse {
    outcome: OwnershipOutcome,
    message: DiscoveryValue,
}
impl OwnerChangeResponse {
    /// Conservative source-backed outcome, without parsing message prose.
    pub const fn outcome(&self) -> OwnershipOutcome {
        self.outcome
    }
    /// Inspect inert bounded provider prose; never render as trusted markup.
    pub fn with_message<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, Error> {
        self.message.with_text(inspect)
    }
}
impl OwnerChangePermit<'_> {
    /// Decode a wire-admitted exact-200 response from this request's exchange.
    /// No crate/owner IDs are echoed. Association relies on trusted dispatch;
    /// an error or timeout can follow an applied mutation and never permits retry.
    pub fn decode_response(&self, success: JsonSuccess<'_>) -> Result<OwnerChangeResponse, Error> {
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let mut root = builder.finish()?;
        if !root.required("ok")?.boolean()? {
            return Err(Error::Binding);
        }
        let message = root.take("msg")?;
        message.with_text(|v| {
            if v.len() > 8192 {
                Err(Error::Limit)
            } else {
                Ok(())
            }
        })??;
        Ok(OwnerChangeResponse {
            message,
            outcome: match self.operation() {
                OwnerChangeOperation::Add => OwnershipOutcome::AdditionAcknowledged,
                OwnerChangeOperation::Remove => OwnershipOutcome::RemovalAcknowledged,
            },
        })
    }
}
