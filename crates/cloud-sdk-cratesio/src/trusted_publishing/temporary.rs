use super::{ExchangePolicy, TrustedPublishingError as Error};
use crate::{
    credentials::{CredentialOrigin, TrustedPublishingToken},
    discovery::DiscoveryValue,
    publishing::{PublishPermit, PublishRequest},
};
use alloc::string::String;

/// Owned secret with conservative local deadline and intended crate binding.
/// The response has no expiry/scope claims: neither field is authenticated token
/// metadata. Registry checks are authoritative. No Clone, raw token getter or refresh.
pub struct TemporaryToken {
    pub(super) credential: TrustedPublishingToken,
    krate: String,
    started: u64,
    deadline: u64,
}
impl TemporaryToken {
    pub(super) fn new(
        value: DiscoveryValue,
        origin: CredentialOrigin,
        policy: ExchangePolicy<'_>,
        now: u64,
    ) -> Result<Self, Error> {
        policy.time(now)?;
        value.with_text(|s| {
            let suffix = s.strip_prefix("cio_tp_").ok_or(Error::Value)?;
            if suffix.len() != 32 || !suffix.bytes().all(|b| b.is_ascii_alphanumeric()) {
                return Err(Error::Value);
            }
            let (raw, check) = suffix.as_bytes().split_at(31);
            let xor = raw.iter().fold(0u8, |a, b| a ^ b);
            let index = usize::from(xor).checked_rem(62).ok_or(Error::Value)?;
            let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
            if alphabet.get(index) != check.first() {
                return Err(Error::Value);
            }
            Ok(())
        })??;
        let credential =
            TrustedPublishingToken::from_secret_string(origin, value.into_secret_text()?)
                .map_err(|_| Error::Value)?;
        let mut krate = String::new();
        krate
            .try_reserve_exact(policy.config.krate.as_str().len())
            .map_err(|_| Error::Allocation)?;
        krate.push_str(policy.config.krate.as_str());
        Ok(Self {
            credential,
            krate,
            started: policy.started,
            deadline: policy.deadline,
        })
    }
    /// Fixed destination, not secret material.
    pub fn origin(&self) -> CredentialOrigin {
        self.credential.origin()
    }
    /// Local deadline, not a server-supplied expiration assertion.
    pub const fn local_deadline(&self) -> u64 {
        self.deadline
    }
    /// Authorize one publish for the locally intended crate. Supply fresh trusted
    /// Unix time and dispatch immediately; the underlying permit has no clock.
    pub fn confirm_publish<'a>(
        &'a self,
        request: PublishRequest<'a>,
        now: u64,
    ) -> Result<PublishPermit<'a>, Error> {
        if now < self.started || now >= self.deadline {
            return Err(Error::Binding);
        }
        if !request.crate_matches(&self.krate)? {
            return Err(Error::Binding);
        }
        Ok(request.confirm_trusted(&self.credential))
    }
    /// Erase local material. Does not perform remote revocation.
    pub fn clear(&mut self) {
        self.credential.clear();
    }
}
impl core::fmt::Debug for TemporaryToken {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("TemporaryToken([redacted])")
    }
}
