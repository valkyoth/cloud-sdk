/// Caller-provided monotonic timestamp in seconds.
///
/// This value deliberately has no relationship to provider wall-clock time.
/// Callers must obtain every timestamp in one credential lifetime from the
/// same monotonic clock.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CredentialTimestamp(u64);

impl CredentialTimestamp {
    /// Wraps caller-provided monotonic seconds without acquiring a clock.
    #[must_use]
    pub const fn from_seconds(seconds: u64) -> Self {
        Self(seconds)
    }

    /// Returns the caller-provided monotonic seconds.
    #[must_use]
    pub const fn as_seconds(self) -> u64 {
        self.0
    }
}

/// Invalid OAuth-style credential lifetime.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CredentialLifetimeError {
    /// `expires_in` must be nonzero.
    ZeroExpiresIn,
    /// Refresh lead time must leave a nonempty fresh interval.
    RefreshWindowTooLarge,
    /// The caller timestamp and `expires_in` overflow the representation.
    TimestampOverflow,
}

impl_static_error!(CredentialLifetimeError,
    Self::ZeroExpiresIn => "credential expires_in must be nonzero",
    Self::RefreshWindowTooLarge => "credential refresh window consumes the complete lifetime",
    Self::TimestampOverflow => "credential expiry timestamp overflows",
);

/// State of an expiring credential at one caller-provided timestamp.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CredentialLifetimeState {
    /// The supplied time precedes the acquisition observation.
    ClockRollback,
    /// The credential is before its refresh window.
    Fresh,
    /// The credential is inside its refresh window but not expired.
    RefreshRequired,
    /// The credential has reached or passed its exclusive expiry.
    Expired,
}

/// Bounded OAuth-style lifetime derived from `expires_in` and caller time.
///
/// Expiry is exclusive: a credential is expired when `now >= expires_at`.
/// The provider duration is never interpreted as provider wall-clock time.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct CredentialLifetime {
    observed_at: CredentialTimestamp,
    refresh_at: CredentialTimestamp,
    expires_at: CredentialTimestamp,
    expires_in: u32,
    refresh_before: u32,
}

impl CredentialLifetime {
    /// Converts a provider `expires_in` duration through explicit caller time.
    pub fn from_expires_in(
        observed_at: CredentialTimestamp,
        expires_in: u32,
        refresh_before: u32,
    ) -> Result<Self, CredentialLifetimeError> {
        if expires_in == 0 {
            return Err(CredentialLifetimeError::ZeroExpiresIn);
        }
        if refresh_before >= expires_in {
            return Err(CredentialLifetimeError::RefreshWindowTooLarge);
        }
        let expires_at = observed_at
            .0
            .checked_add(u64::from(expires_in))
            .ok_or(CredentialLifetimeError::TimestampOverflow)?;
        let refresh_at = expires_at
            .checked_sub(u64::from(refresh_before))
            .ok_or(CredentialLifetimeError::TimestampOverflow)?;
        Ok(Self {
            observed_at,
            refresh_at: CredentialTimestamp(refresh_at),
            expires_at: CredentialTimestamp(expires_at),
            expires_in,
            refresh_before,
        })
    }

    /// Returns when the provider duration was observed.
    #[must_use]
    pub const fn observed_at(self) -> CredentialTimestamp {
        self.observed_at
    }

    /// Returns the first timestamp at which refresh is required.
    #[must_use]
    pub const fn refresh_at(self) -> CredentialTimestamp {
        self.refresh_at
    }

    /// Returns the exclusive expiry timestamp.
    #[must_use]
    pub const fn expires_at(self) -> CredentialTimestamp {
        self.expires_at
    }

    /// Returns the admitted provider duration.
    #[must_use]
    pub const fn expires_in(self) -> u32 {
        self.expires_in
    }

    /// Returns the caller-selected refresh lead time.
    #[must_use]
    pub const fn refresh_before(self) -> u32 {
        self.refresh_before
    }

    /// Classifies the lifetime at one observation from the same caller clock.
    #[must_use]
    pub const fn state_at(self, now: CredentialTimestamp) -> CredentialLifetimeState {
        if now.0 < self.observed_at.0 {
            CredentialLifetimeState::ClockRollback
        } else if now.0 >= self.expires_at.0 {
            CredentialLifetimeState::Expired
        } else if now.0 >= self.refresh_at.0 {
            CredentialLifetimeState::RefreshRequired
        } else {
            CredentialLifetimeState::Fresh
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CredentialLifetime, CredentialLifetimeError, CredentialLifetimeState, CredentialTimestamp,
    };

    #[test]
    fn expires_in_uses_explicit_caller_time_and_exclusive_expiry() {
        let lifetime = CredentialLifetime::from_expires_in(
            CredentialTimestamp::from_seconds(1_000),
            3_599,
            300,
        );
        assert!(lifetime.is_ok());
        let Ok(lifetime) = lifetime else {
            unreachable!("credential lifetime fixture construction failed");
        };
        assert_eq!(lifetime.refresh_at().as_seconds(), 4_299);
        assert_eq!(lifetime.expires_at().as_seconds(), 4_599);
        assert_eq!(
            lifetime.state_at(CredentialTimestamp::from_seconds(999)),
            CredentialLifetimeState::ClockRollback
        );
        assert_eq!(
            lifetime.state_at(CredentialTimestamp::from_seconds(4_298)),
            CredentialLifetimeState::Fresh
        );
        assert_eq!(
            lifetime.state_at(CredentialTimestamp::from_seconds(4_299)),
            CredentialLifetimeState::RefreshRequired
        );
        assert_eq!(
            lifetime.state_at(CredentialTimestamp::from_seconds(4_599)),
            CredentialLifetimeState::Expired
        );
    }

    #[test]
    fn invalid_or_overflowing_lifetimes_fail_closed() {
        let now = CredentialTimestamp::from_seconds(10);
        assert_eq!(
            CredentialLifetime::from_expires_in(now, 0, 0),
            Err(CredentialLifetimeError::ZeroExpiresIn)
        );
        assert_eq!(
            CredentialLifetime::from_expires_in(now, 60, 60),
            Err(CredentialLifetimeError::RefreshWindowTooLarge)
        );
        assert_eq!(
            CredentialLifetime::from_expires_in(CredentialTimestamp::from_seconds(u64::MAX), 1, 0,),
            Err(CredentialLifetimeError::TimestampOverflow)
        );
    }
}
