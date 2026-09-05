use super::{Credential, CredentialContext, CredentialError, CredentialKind};
use cloud_sdk::Method;
use cloud_sdk::transport::{BoundTransport, EndpointIdentity, HeaderValue, RequestTarget};
use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};
use core::fmt;

/// Closure-scoped material for a trusted adapter, with redacted diagnostics.
///
/// The adapter must send exactly this method/target to this endpoint, add at
/// most one Authorization header, mark it sensitive, omit cookies and disable
/// redirects. Raw wire access is intentional here, not a safe high-level
/// executor or proof that an arbitrary callback follows those obligations.
pub struct ScopedCredentialMaterial<'a> {
    endpoint: EndpointIdentity<'static>,
    method: Method,
    target: RequestTarget<'a>,
    authorization: Option<HeaderValue<'a>>,
    json: Option<&'a [u8]>,
}

impl<'a> ScopedCredentialMaterial<'a> {
    /// Returns the exact allowed origin.
    #[must_use]
    pub const fn endpoint(&self) -> EndpointIdentity<'static> {
        self.endpoint
    }
    /// Returns the source-locked operation method.
    #[must_use]
    pub const fn method(&self) -> Method {
        self.method
    }
    /// Returns the exact target; it may contain a secret path token.
    #[must_use]
    pub const fn target(&self) -> RequestTarget<'a> {
        self.target
    }
    /// Returns the complete Authorization value, with no additional prefix.
    #[must_use]
    pub const fn authorization(&self) -> Option<HeaderValue<'a>> {
        self.authorization
    }
    /// Returns the OIDC JSON body, which requires application/json.
    #[must_use]
    pub const fn json_body(&self) -> Option<&'a [u8]> {
        self.json
    }
}

impl fmt::Debug for ScopedCredentialMaterial<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ScopedCredentialMaterial([redacted])")
    }
}

impl<K: CredentialKind> Credential<K> {
    /// Supplies bounded wire material to a trusted adapter callback after
    /// checking that the same transport is bound to the credential's origin.
    ///
    /// The full output is cleared before use and on return/error/unwind. The
    /// callback cannot return a borrow into the output. It can deliberately
    /// copy bytes, so it is part of the trusted adapter boundary. This method
    /// performs no I/O and does not validate server-side token scope/expiry.
    ///
    /// A temporary token cannot be used with an API-token context:
    /// ```compile_fail
    /// use cloud_sdk_cratesio::credentials::{Api, CredentialContext, TrustedPublishingToken};
    /// use cloud_sdk::transport::BoundTransport;
    /// fn wrong(token: &TrustedPublishingToken, context: &CredentialContext<'_, Api>, transport: &impl BoundTransport) {
    ///     token.with_material_for_adapter(context, transport, &mut [0; 2048], |_, _| ());
    /// }
    /// ```
    /// Secret material cannot escape its callback lifetime:
    /// ```compile_fail
    /// use cloud_sdk_cratesio::credentials::{Api, ApiToken, CredentialContext};
    /// use cloud_sdk::transport::BoundTransport;
    /// fn escape(token: &ApiToken, context: &CredentialContext<'_, Api>, transport: &impl BoundTransport) {
    ///     let secret = token.with_material_for_adapter(context, transport, &mut [0; 2048], |_, material| material.authorization());
    /// }
    /// ```
    pub fn with_material_for_adapter<T: BoundTransport + ?Sized, R>(
        &self,
        context: &CredentialContext<'_, K>,
        transport: &T,
        output: &mut [u8],
        apply: impl for<'a> FnOnce(&T, ScopedCredentialMaterial<'a>) -> R,
    ) -> Result<R, CredentialError> {
        let mut output = SecretBuffer::new(output);
        sanitize_bytes(output.as_mut_slice());
        if self.origin != context.origin {
            return Err(CredentialError::DestinationMismatch);
        }
        let endpoint = self.origin.endpoint();
        endpoint
            .verify_transport(transport)
            .map_err(|_| CredentialError::DestinationMismatch)?;
        let identity = endpoint
            .identity()
            .map_err(|_| CredentialError::DestinationMismatch)?;
        self.secret
            .try_with_secret(|text| {
                super::kind::validate::<K>(text.as_bytes())?;
                let (prefix, suffix) = match K::KIND {
                    0 => ("", ""),
                    1 => ("Bearer ", ""),
                    2 => ("{\"jwt\":\"", "\"}"),
                    _ => (context.target, ""),
                };
                let length = prefix
                    .len()
                    .checked_add(text.len())
                    .and_then(|length| length.checked_add(suffix.len()))
                    .ok_or(CredentialError::OutputTooSmall)?;
                let wire = output
                    .as_mut_slice()
                    .get_mut(..length)
                    .ok_or(CredentialError::OutputTooSmall)?;
                let (start, rest) = wire.split_at_mut(prefix.len());
                let (secret, end) = rest.split_at_mut(text.len());
                start.copy_from_slice(prefix.as_bytes());
                secret.copy_from_slice(text.as_bytes());
                end.copy_from_slice(suffix.as_bytes());
                let wire =
                    core::str::from_utf8(wire).map_err(|_| CredentialError::StorageUnavailable)?;
                let target = if K::KIND >= 3 { wire } else { context.target };
                let target =
                    RequestTarget::new(target).map_err(|_| CredentialError::InvalidSyntax)?;
                let authorization = if K::KIND <= 1 {
                    Some(HeaderValue::new(wire).map_err(|_| CredentialError::InvalidSyntax)?)
                } else {
                    None
                };
                let json = if K::KIND == 2 {
                    Some(wire.as_bytes())
                } else {
                    None
                };
                Ok(apply(
                    transport,
                    ScopedCredentialMaterial {
                        endpoint: identity,
                        method: context.method,
                        target,
                        authorization,
                        json,
                    },
                ))
            })
            .map_err(|_| CredentialError::StorageUnavailable)?
    }
}
