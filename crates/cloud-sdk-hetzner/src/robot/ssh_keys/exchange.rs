use alloc::{string::String, vec::Vec};

use base64_ng::STRICT_STANDARD_PADDED;
use cloud_sdk::operation::{
    CheckedResponseGuard, PreparationStorage, PrepareOperation, PreparedRequest,
    ResponsePolicyError,
};
use cloud_sdk::transport::ResponseBuffer;
use cloud_sdk_sanitization::{SecretBuffer, SecretString};

use super::decode::{RobotSshKeyDecodeError, decode_robot_ssh_key, decode_robot_ssh_key_list};
use super::model::{RobotSshKey, RobotSshKeyList};
use super::request::*;
use crate::serde::models::ssh_wire::{
    SshKeyIdentity, parse_openssh_key_identity, parse_ssh2_wire_identity,
};

/// Prepared Robot SSH-key request retaining its exact typed association.
pub struct PreparedRobotSshKey<'storage, 'request, R> {
    request: &'request R,
    inner: PreparedRequest<'storage>,
}

impl<'storage, 'request, R> PreparedRobotSshKey<'storage, 'request, R> {
    /// Borrows the provider-neutral prepared request for inspection.
    #[must_use]
    pub const fn as_untyped(&self) -> PreparedRequest<'storage> {
        self.inner
    }

    /// Applies the exact response policy and retains request provenance.
    pub fn validate_response<'buffer>(
        self,
        response: ResponseBuffer<'buffer>,
    ) -> Result<CheckedRobotSshKey<'buffer, 'request, R>, ResponsePolicyError> {
        self.inner
            .validate_response(response)
            .map(|inner| CheckedRobotSshKey {
                request: self.request,
                inner,
            })
    }

    pub(super) fn into_plan_parts(self) -> (PreparedRequest<'storage>, &'request R) {
        (self.inner, self.request)
    }
}

impl<R> core::fmt::Debug for PreparedRobotSshKey<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("PreparedRobotSshKey")
            .field("request", &"[bound]")
            .field("prepared", &self.inner)
            .finish()
    }
}

/// Checked Robot SSH-key response retaining its admitting request.
pub struct CheckedRobotSshKey<'buffer, 'request, R> {
    request: &'request R,
    inner: CheckedResponseGuard<'buffer>,
}

impl<'buffer, 'request, R> CheckedRobotSshKey<'buffer, 'request, R> {
    pub(super) const fn from_executed(
        request: &'request R,
        inner: CheckedResponseGuard<'buffer>,
    ) -> Self {
        Self { request, inner }
    }
}

impl<R> core::fmt::Debug for CheckedRobotSshKey<'_, '_, R> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("CheckedRobotSshKey")
            .field("request", &"[bound]")
            .field("response", &"[checked]")
            .finish()
    }
}

macro_rules! prepare_bound {
    ($($type:ty),+ $(,)?) => {$ (
        impl $type {
            /// Prepares this operation while retaining exact response association.
            pub fn prepare_bound<'storage, 'request>(
                &'request self,
                storage: PreparationStorage<'storage>,
            ) -> Result<PreparedRobotSshKey<'storage, 'request, Self>, RobotSshKeyRequestError> {
                let inner = self.prepare(storage)?;
                Ok(PreparedRobotSshKey { request: self, inner })
            }
        }
    )+ };
}

prepare_bound!(
    RobotSshKeyListRequest,
    RobotSshKeyCreateRequest<'_>,
    RobotSshKeyGetRequest,
    RobotSshKeyUpdateRequest,
    RobotSshKeyDeleteRequest,
);

impl CheckedRobotSshKey<'_, '_, RobotSshKeyListRequest> {
    /// Decodes a bounded list with distinct cryptographic identities.
    pub fn decode_response(self) -> Result<RobotSshKeyList, RobotSshKeyDecodeError> {
        self.inner
            .decode_owned_with_workspace(decode_robot_ssh_key_list)
    }
}

impl CheckedRobotSshKey<'_, '_, RobotSshKeyGetRequest> {
    /// Decodes one key and binds it to the requested fingerprint.
    pub fn decode_response(self) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
        let result = decode_one(self.inner)?;
        if result.fingerprint() == self.request.fingerprint() {
            Ok(result)
        } else {
            Err(RobotSshKeyDecodeError::ResponseIdentityMismatch)
        }
    }
}

impl CheckedRobotSshKey<'_, '_, RobotSshKeyCreateRequest<'_>> {
    /// Requires the created name and cryptographic key identity to match.
    pub fn decode_response(self) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
        let expected = self.request.data().with_text(request_identity)?;
        let result = decode_one(self.inner)?;
        if result.name() == self.request.name() && result.sha256_fingerprint() == expected.sha256()
        {
            Ok(result)
        } else {
            Err(RobotSshKeyDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotSshKey<'_, '_, RobotSshKeyUpdateRequest> {
    /// Requires the provider to preserve the fingerprint and apply the name.
    pub fn decode_response(self) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
        let result = decode_one(self.inner)?;
        if result.fingerprint() == self.request.fingerprint()
            && result.name() == self.request.name()
        {
            Ok(result)
        } else {
            Err(RobotSshKeyDecodeError::MutationOutcomeMismatch)
        }
    }
}

impl CheckedRobotSshKey<'_, '_, RobotSshKeyDeleteRequest> {
    /// Accepts and clears the exact empty delete acknowledgement.
    pub fn decode_response(self) -> Result<(), RobotSshKeyDecodeError> {
        drop(self);
        Ok(())
    }
}

fn decode_one(checked: CheckedResponseGuard<'_>) -> Result<RobotSshKey, RobotSshKeyDecodeError> {
    checked.decode_owned_with_workspace(decode_robot_ssh_key)
}

fn request_identity(value: &str) -> Result<SshKeyIdentity, RobotSshKeyDecodeError> {
    if !value.starts_with("---- BEGIN SSH2 PUBLIC KEY ----\n") {
        return parse_openssh_key_identity(value).map_err(map_model_error);
    }
    let mut encoded = String::new();
    encoded
        .try_reserve_exact(value.len())
        .map_err(|_| RobotSshKeyDecodeError::Allocation)?;
    let mut body_started = false;
    for line in value.lines().skip(1) {
        if line == "---- END SSH2 PUBLIC KEY ----" {
            break;
        }
        let base64 = line
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'+' | b'/' | b'='));
        if base64 {
            body_started = true;
            encoded.push_str(line);
        } else if body_started {
            return Err(RobotSshKeyDecodeError::InvalidKey);
        }
    }
    let encoded = SecretString::from_string(encoded);
    encoded
        .try_with_secret(decode_ssh2)
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?
}

fn decode_ssh2(encoded: &str) -> Result<SshKeyIdentity, RobotSshKeyDecodeError> {
    let decoded_len = STRICT_STANDARD_PADDED
        .decoded_len(encoded.as_bytes())
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?;
    if decoded_len == 0 || decoded_len > super::MAX_ROBOT_SSH_KEY_DATA_BYTES {
        return Err(RobotSshKeyDecodeError::InvalidKey);
    }
    let mut storage = Vec::new();
    storage
        .try_reserve_exact(decoded_len)
        .map_err(|_| RobotSshKeyDecodeError::Allocation)?;
    storage.resize(decoded_len, 0);
    let mut wire = SecretBuffer::new(storage.as_mut_slice());
    let written = STRICT_STANDARD_PADDED
        .decode_into(encoded.as_bytes(), wire.as_mut_slice())
        .map_err(|_| RobotSshKeyDecodeError::InvalidKey)?;
    if written != decoded_len {
        return Err(RobotSshKeyDecodeError::InvalidKey);
    }
    parse_ssh2_wire_identity(wire.as_slice()).map_err(map_model_error)
}

const fn map_model_error(error: crate::serde::ResponseModelError) -> RobotSshKeyDecodeError {
    if matches!(error, crate::serde::ResponseModelError::Allocation) {
        RobotSshKeyDecodeError::Allocation
    } else {
        RobotSshKeyDecodeError::InvalidKey
    }
}

#[cfg(doctest)]
mod compile_fail {
    /// Different operation types cannot consume each other's checked response.
    ///
    /// ```compile_fail
    /// use cloud_sdk_hetzner::robot::{
    ///     CheckedRobotSshKey, RobotSshKeyGetRequest, RobotSshKeyUpdateRequest,
    /// };
    /// fn consume(_: CheckedRobotSshKey<'_, '_, RobotSshKeyGetRequest>) {}
    /// fn wrong(response: CheckedRobotSshKey<'_, '_, RobotSshKeyUpdateRequest>) {
    ///     consume(response);
    /// }
    /// ```
    fn association() {}
}
