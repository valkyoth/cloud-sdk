use super::SettingsError as Error;
use crate::{
    credentials::ApiToken,
    endpoint::ApiRequestTarget,
    identifiers::{CrateName, Version},
    query::{ApiPath, FixedSegment as F, PathSegment as P, QueryError},
};
use cloud_sdk::{
    Method,
    buffer::{SnapshotEncoder, encode_snapshot_bounded},
};

/// Explicit message replacement. Omission/null upstream both clear the message;
/// there is deliberately no misleading `Unchanged` variant.
#[derive(Clone, Copy)]
pub enum YankMessage<'a> {
    /// Explicitly remove the existing message, serialized as JSON null.
    Clear,
    /// Set bounded text. Text is inert metadata, not trusted Markdown/HTML.
    Set(&'a str),
}
impl core::fmt::Debug for YankMessage<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("YankMessage([redacted])")
    }
}
/// Source-locked PATCH operation. Every attempt needs a fresh permit.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SettingsOperation {
    /// Trusted-publishing-only policy for a crate.
    Crate,
    /// Yank state and message for an exact version.
    Version,
}
impl SettingsOperation {
    /// Official operation ID.
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Crate => "update_crate",
            Self::Version => "update_version",
        }
    }
    /// Exact HTTP verb.
    pub const fn method(self) -> Method {
        Method::Patch
    }
    /// No automatic replay even for state-convergent writes; concurrent intent can change.
    pub const fn permits_automatic_retry(self) -> bool {
        false
    }
}
#[derive(Clone, Copy)]
pub(super) enum Patch<'a> {
    Crate(bool),
    Version(Version<'a>, Option<bool>, YankMessage<'a>),
}
/// Immutable, validated settings intent. No arbitrary fields or empty patches.
pub struct SettingsRequest<'a> {
    pub(super) name: CrateName<'a>,
    pub(super) patch: Patch<'a>,
}
impl<'a> SettingsRequest<'a> {
    /// Change only this crate's trusted-publishing requirement. Disabling it can
    /// weaken publication policy and must be explicitly confirmed.
    pub const fn trustpub_only(name: CrateName<'a>, enabled: bool) -> Self {
        Self {
            name,
            patch: Patch::Crate(enabled),
        }
    }
    /// Set yank state (Some), or omit that field (None) for a message-only edit.
    /// Message clearing is always explicit. Setting a message when unyanking is
    /// rejected; for message-only edits the server must verify the current state.
    /// Empty text is a deliberate string value, distinct from Clear.
    pub fn version(
        name: CrateName<'a>,
        version: Version<'a>,
        yanked: Option<bool>,
        message: YankMessage<'a>,
    ) -> Result<Self, Error> {
        if let YankMessage::Set(text) = message {
            if yanked == Some(false) {
                return Err(Error::Value);
            }
            if text.len() > super::MAX_YANK_MESSAGE_BYTES {
                return Err(Error::Limit);
            }
            if text
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\n' | '\r' | '\t'))
            {
                return Err(Error::Value);
            }
        }
        Ok(Self {
            name,
            patch: Patch::Version(version, yanked, message),
        })
    }
    /// Immutable source operation.
    pub const fn operation(&self) -> SettingsOperation {
        match self.patch {
            Patch::Crate(_) => SettingsOperation::Crate,
            Patch::Version(..) => SettingsOperation::Version,
        }
    }
    /// Atomic target encoding, unchanged on capacity failure.
    pub fn write_target<'b>(
        &self,
        output: &'b mut [u8],
    ) -> Result<ApiRequestTarget<'b>, QueryError> {
        match self.patch {
            Patch::Crate(_) => {
                ApiPath::new(&[P::Fixed(F::Crates), P::Crate(self.name)])?.write(output)
            }
            Patch::Version(version, ..) => ApiPath::new(&[
                P::Fixed(F::Crates),
                P::Crate(self.name),
                P::Version(version),
            ])?
            .write(output),
        }
    }
    /// Inspect bounded JSON in cleanup-owned scratch. Do not retain private copies.
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
            self.patch,
            output,
            super::MAX_SETTINGS_BODY_BYTES,
            Error::Limit,
            encode,
        )
    }
    /// Explicitly authorize exactly this patch and token, including yank or policy
    /// weakening. This is caller consent, not proof of upstream ownership/scope.
    pub fn confirm(self, credential: &'a ApiToken) -> SettingsPermit<'a> {
        SettingsPermit {
            request: self,
            credential,
        }
    }
}
impl core::fmt::Debug for SettingsRequest<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SettingsRequest([redacted])")
    }
}
/// Consumed, non-cloneable authorization bound to one immutable patch and token.
///
/// ```compile_fail
/// use cloud_sdk_cratesio::settings::SettingsPermit;
/// fn duplicate(p: SettingsPermit<'_>) { let _ = p.clone(); }
/// ```
///
/// ```compile_fail
/// use cloud_sdk_cratesio::{settings::SettingsRequest, credentials::ApiToken, identifiers::CrateName};
/// fn rotate(token: &mut ApiToken, name: CrateName<'_>) {
///     let permit = SettingsRequest::trustpub_only(name, true).confirm(token);
///     token.clear();
///     let _ = permit.operation();
/// }
/// ```
pub struct SettingsPermit<'a> {
    pub(super) request: SettingsRequest<'a>,
    pub(super) credential: &'a ApiToken,
}
impl SettingsPermit<'_> {
    /// Exact operation, without revealing target or credential.
    pub const fn operation(&self) -> SettingsOperation {
        self.request.operation()
    }
}
impl core::fmt::Debug for SettingsPermit<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("SettingsPermit([redacted])")
    }
}
fn encode(patch: Patch<'_>, e: &mut SnapshotEncoder<'_, Error>) -> Result<(), Error> {
    match patch {
        Patch::Crate(enabled) => {
            e.string("{\"crate\":{\"trustpub_only\":")?;
            e.string(if enabled { "true" } else { "false" })?;
        }
        Patch::Version(_, yanked, message) => {
            e.string("{\"version\":{")?;
            if let Some(yanked) = yanked {
                e.string("\"yanked\":")?;
                e.string(if yanked { "true," } else { "false," })?;
            }
            e.string("\"yank_message\":")?;
            match message {
                YankMessage::Clear => e.string("null")?,
                YankMessage::Set(s) => e.json_string(s)?,
            }
        }
    }
    e.string("}}")
}
