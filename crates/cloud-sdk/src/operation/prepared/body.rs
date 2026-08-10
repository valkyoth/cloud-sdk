//! Prepared request-body security and replay policy.

/// Whether one prepared request body can be sent again byte-for-byte.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum BodyReplayability {
    /// The body source cannot guarantee an identical subsequent read.
    NotReplayable,
    /// The complete body is an immutable byte snapshot for the request lifetime.
    Replayable,
}

/// Whether a prepared request body contains caller-designated sensitive data.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum RequestBodySensitivity {
    /// The body has no provider-declared confidential fields.
    Public,
    /// The body contains confidential material and requires digest fingerprints.
    Sensitive,
}

impl RequestBodySensitivity {
    /// Reports whether exact canonical fingerprint retention is forbidden.
    #[must_use]
    pub const fn requires_digest(self) -> bool {
        matches!(self, Self::Sensitive)
    }
}
