use super::{OwnerChangeError as Error, OwnerSelector};
use crate::{
    credentials::ApiToken,
    endpoint::ApiRequestTarget,
    identifiers::CrateName,
    query::{ApiPath, FixedSegment as F, PathSegment as P, QueryError},
};
use cloud_sdk::{
    Method,
    buffer::{SnapshotEncoder, encode_snapshot_bounded},
};

/// Both operations change authority. Neither authorizes automatic retries.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OwnerChangeOperation {
    /// User invitations or immediate team addition, not guaranteed user acceptance.
    Add,
    /// Destructive removal of existing authority.
    Remove,
}
impl OwnerChangeOperation {
    /// Source-owned operation ID.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Add => "add_owners",
            Self::Remove => "remove_owners",
        }
    }
    /// Cargo-compatible method; DELETE includes a JSON body.
    pub const fn method(self) -> Method {
        match self {
            Self::Add => Method::Put,
            Self::Remove => Method::Delete,
        }
    }
    /// Explicit destruction classification.
    pub const fn is_destructive(self) -> bool {
        matches!(self, Self::Remove)
    }
    /// No implicit replay, even after an ambiguous transport failure.
    pub const fn permits_automatic_retry(self) -> bool {
        false
    }
}
/// Immutable exact crate and bounded batch, with no arbitrary JSON fields.
pub struct OwnerChangeRequest<'a> {
    pub(super) name: CrateName<'a>,
    pub(super) owners: &'a [OwnerSelector<'a>],
    pub(super) operation: OwnerChangeOperation,
}
impl<'a> OwnerChangeRequest<'a> {
    fn new(
        name: CrateName<'a>,
        owners: &'a [OwnerSelector<'a>],
        operation: OwnerChangeOperation,
    ) -> Result<Self, Error> {
        if owners.is_empty() || owners.len() > super::MAX_OWNER_CHANGES {
            return Err(Error::Limit);
        }
        for (index, owner) in owners.iter().enumerate() {
            if owners
                .get(..index)
                .ok_or(Error::Limit)?
                .iter()
                .any(|prior| owner.duplicate(*prior))
            {
                return Err(Error::Value);
            }
        }
        Ok(Self {
            name,
            owners,
            operation,
        })
    }
    /// Build an addition batch; this does not send or grant permission.
    pub fn add(name: CrateName<'a>, owners: &'a [OwnerSelector<'a>]) -> Result<Self, Error> {
        Self::new(name, owners, OwnerChangeOperation::Add)
    }
    /// Build a removal batch; explicit destructive confirmation is still required.
    pub fn remove(name: CrateName<'a>, owners: &'a [OwnerSelector<'a>]) -> Result<Self, Error> {
        Self::new(name, owners, OwnerChangeOperation::Remove)
    }
    /// Exact operation.
    pub const fn operation(&self) -> OwnerChangeOperation {
        self.operation
    }
    /// Atomic target encoding; failed capacity checks leave output untouched.
    pub fn write_target<'b>(
        &self,
        output: &'b mut [u8],
    ) -> Result<ApiRequestTarget<'b>, QueryError> {
        ApiPath::new(&[
            P::Fixed(F::Crates),
            P::Crate(self.name),
            P::Fixed(F::Owners),
        ])?
        .write(output)
    }
    /// Inspect Cargo's `users` body alias in cleanup-owned scratch. The upstream
    /// controller accepts this alias of its OpenAPI `owners` field.
    pub fn with_json_body<R>(
        &self,
        output: &mut [u8],
        inspect: impl FnOnce(&[u8]) -> R,
    ) -> Result<R, Error> {
        let mut output = cloud_sdk_sanitization::SecretBuffer::new(output);
        cloud_sdk_sanitization::sanitize_bytes(output.as_mut_slice());
        let length = self.body(output.as_mut_slice())?;
        Ok(inspect(
            output.as_slice().get(..length).ok_or(Error::Limit)?,
        ))
    }
    pub(super) fn body(&self, output: &mut [u8]) -> Result<usize, Error> {
        encode_snapshot_bounded(
            self.owners,
            output,
            super::MAX_OWNER_CHANGE_BODY_BYTES,
            Error::Limit,
            encode,
        )
    }
    /// Confirm only an addition; cannot accidentally authorize removal.
    pub fn confirm_add(self, credential: &'a ApiToken) -> Result<OwnerChangePermit<'a>, Error> {
        if self.operation != OwnerChangeOperation::Add {
            return Err(Error::Binding);
        }
        Ok(OwnerChangePermit {
            request: self,
            credential,
        })
    }
    /// Explicit destructive consent. The server enforces last-individual-owner
    /// and ownership rules. Consider confirm_removal_after_preflight for local
    /// self-removal/remaining-owner checks against caller-supplied current state.
    pub fn confirm_removal(self, credential: &'a ApiToken) -> Result<OwnerChangePermit<'a>, Error> {
        if self.operation != OwnerChangeOperation::Remove {
            return Err(Error::Binding);
        }
        Ok(OwnerChangePermit {
            request: self,
            credential,
        })
    }
}
impl core::fmt::Debug for OwnerChangeRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("OwnerChangeRequest([redacted])")
    }
}
/// Non-cloneable consumed authority, binding exact request and borrowed token.
/// Caller consent does not prove upstream permissions or token scope.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::ownership::OwnerChangePermit;
/// fn replay(p: OwnerChangePermit<'_>) { let _ = p.clone(); }
/// ```
pub struct OwnerChangePermit<'a> {
    pub(super) request: OwnerChangeRequest<'a>,
    #[cfg_attr(
        not(any(feature = "blocking", feature = "async")),
        expect(dead_code, reason = "retained for authenticated execution")
    )]
    pub(super) credential: &'a ApiToken,
}
impl OwnerChangePermit<'_> {
    /// Exact immutable operation.
    pub const fn operation(&self) -> OwnerChangeOperation {
        self.request.operation
    }
}
impl core::fmt::Debug for OwnerChangePermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("OwnerChangePermit([redacted])")
    }
}
fn encode(owners: &[OwnerSelector<'_>], e: &mut SnapshotEncoder<'_, Error>) -> Result<(), Error> {
    e.string("{\"users\":[")?;
    for (index, owner) in owners.iter().enumerate() {
        if index != 0 {
            e.byte(b',')?;
        }
        // Identifier constructors admit only ASCII alphanumerics, colon, dot,
        // hyphen and underscore; no JSON escape characters can reach this writer.
        let (prefix, name) = owner.parts();
        e.byte(b'"')?;
        e.string(prefix)?;
        e.string(name)?;
        e.byte(b'"')?;
    }
    e.string("]}")
}
