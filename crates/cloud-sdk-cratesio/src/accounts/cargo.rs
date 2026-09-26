//! Explicit Cargo owner-list profile, separate from website account models.
//! The Cargo contract does not distinguish users from teams. These records are
//! inspection metadata, not ownership-change authorization or verified identity.
use super::{AccountError as Error, AccountRequest, MAX_OWNERS};
use crate::{
    credentials::ApiToken,
    discovery::{DiscoveryValue, value::Builder},
    identifiers::CrateName,
    wire::JsonSuccess,
};
use alloc::vec::Vec;
use cloud_sdk::incremental_json::IncrementalJsonProgress;

#[cfg(any(feature = "blocking", feature = "async"))]
mod execution;
#[cfg(test)]
mod tests;

/// Read-only Cargo owner inspection with explicit API-token consent.
/// Unlike website owner inspection, Cargo sends the raw token in Authorization.
pub struct CargoOwnersRequest<'a> {
    name: CrateName<'a>,
    token: &'a ApiToken,
}
impl<'a> CargoOwnersRequest<'a> {
    /// Binds a crate and an explicitly supplied origin-bound token.
    #[must_use]
    pub const fn new(name: CrateName<'a>, token: &'a ApiToken) -> Self {
        Self { name, token }
    }
    /// Writes the fixed owner-list path atomically; no query or body is used.
    pub fn write_target<'b>(
        &self,
        output: &'b mut [u8],
    ) -> Result<crate::endpoint::ApiRequestTarget<'b>, crate::query::QueryError> {
        AccountRequest::owners(self.name).write_target(output)
    }
    /// Returns the credential's closed production/staging destination.
    #[must_use]
    pub fn origin(&self) -> crate::credentials::CredentialOrigin {
        self.token.origin()
    }
}
impl core::fmt::Debug for CargoOwnersRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("CargoOwnersRequest([redacted])")
    }
}

/// A Cargo-profile owner. No user/team classification is inferred from login.
#[derive(Debug)]
pub struct CargoOwner {
    id: u32,
    fields: DiscoveryValue,
}
impl CargoOwner {
    /// Returns the Cargo unsigned 32-bit identifier, not a mutation capability.
    #[must_use]
    pub const fn id(&self) -> u32 {
        self.id
    }
    /// Inspects the protected login without retaining a borrowed string.
    pub fn with_login<R>(&self, inspect: impl FnOnce(&str) -> R) -> Result<R, Error> {
        self.fields.required("login")?.with_text(inspect)
    }
    /// Returns bounded inert metadata; missing and null names remain distinct.
    #[must_use]
    pub const fn fields(&self) -> &DiscoveryValue {
        &self.fields
    }
}
/// Bounded Cargo owner list. Association with a crate relies on the authenticated
/// exchange: the response does not echo the requested crate name.
#[derive(Debug)]
pub struct CargoOwners(Vec<CargoOwner>);
impl CargoOwners {
    /// Borrows all owners; an oversized list is rejected, never truncated.
    #[must_use]
    pub fn items(&self) -> &[CargoOwner] {
        &self.0
    }
    /// Consumes checked JSON and clears its input on every exit. Intended for
    /// the Cargo response profile, not a fallback for website model failures.
    pub fn decode(success: JsonSuccess<'_>) -> Result<Self, Error> {
        let mut builder = Builder::default();
        if success
            .visit(&mut builder)
            .map_err(|e| e.into_visitor_error().unwrap_or(Error::Json))?
            != IncrementalJsonProgress::Complete
        {
            return Err(Error::Json);
        }
        let values = builder.finish()?.take("users")?.into_array()?;
        if values.len() > MAX_OWNERS {
            return Err(Error::Limit);
        }
        let mut items: Vec<CargoOwner> = Vec::new();
        items
            .try_reserve_exact(values.len())
            .map_err(|_| Error::Allocation)?;
        for fields in values {
            let id = u32::try_from(fields.required("id")?.count(u64::from(u32::MAX))?)
                .map_err(|_| Error::Value)?;
            bounded_text(fields.required("login")?, true)?;
            if let Some(name) = fields.get("name")?
                && !name.is_null()
            {
                bounded_text(name, false)?;
            }
            let item = CargoOwner { id, fields };
            for previous in &items {
                // The concrete service has disjoint user/team ID namespaces.
                // Cargo omits kind, so equal numeric IDs alone are not duplicates.
                if previous.with_login(|a| item.with_login(|b| a == b))?? {
                    return Err(Error::Value);
                }
            }
            items.push(item);
        }
        Ok(Self(items))
    }
}
fn bounded_text(value: &DiscoveryValue, nonempty: bool) -> Result<(), Error> {
    value.with_text(|s| {
        if s.len() > 256 {
            return Err(Error::Limit);
        }
        if (nonempty && s.is_empty()) || s.chars().any(char::is_control) {
            return Err(Error::Value);
        }
        Ok(())
    })?
}
