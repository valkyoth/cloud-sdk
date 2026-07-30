use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

use crate::transport::TransportRequest;

use super::SigningContext;

/// Caller-provided request-signing implementation.
pub trait RequestSigner {
    /// Signing failure.
    type Error;

    /// Signs the exact versioned canonical input into caller-owned output.
    fn sign(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error>;
}

/// Validated signing-output failure.
pub enum SigningOutputError<E> {
    /// The caller-provided signer rejected the canonical input.
    Signer(E),
    /// A successful signer returned zero output bytes.
    Empty,
    /// A successful signer returned a length beyond the supplied output.
    TooLong,
}

impl<E> fmt::Debug for SigningOutputError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Signer(_) => "SigningOutputError::Signer([redacted])",
            Self::Empty => "SigningOutputError::Empty",
            Self::TooLong => "SigningOutputError::TooLong",
        })
    }
}

impl<E> fmt::Display for SigningOutputError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Signer(_) => "request signer rejected the canonical input",
            Self::Empty => "request signer returned empty output",
            Self::TooLong => "request signer returned an out-of-bounds length",
        })
    }
}

impl<E> core::error::Error for SigningOutputError<E>
where
    E: core::error::Error + 'static,
{
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            Self::Signer(error) => Some(error),
            Self::Empty | Self::TooLong => None,
        }
    }
}

/// Exact request and validated signature produced from one canonical snapshot.
pub struct SignedRequest<'output, 'request> {
    request: TransportRequest<'request>,
    context: SigningContext<'request>,
    storage: &'output mut [u8],
    len: usize,
}

impl<'output, 'request> SignedRequest<'output, 'request> {
    pub(super) fn sign<S: RequestSigner>(
        request: TransportRequest<'request>,
        context: SigningContext<'request>,
        input: &[u8],
        signer: &S,
        output: &'output mut [u8],
    ) -> Result<Self, SigningOutputError<S::Error>> {
        sanitize_bytes(output);
        let mut signed = Self {
            request,
            context,
            storage: output,
            len: 0,
        };
        let len = signer
            .sign(input, signed.storage)
            .map_err(SigningOutputError::Signer)?;
        if len == 0 {
            return Err(SigningOutputError::Empty);
        }
        if len > signed.storage.len() {
            return Err(SigningOutputError::TooLong);
        }
        signed.len = len;
        Ok(signed)
    }

    /// Returns the exact request whose body and metadata were signed.
    #[must_use]
    pub const fn request(&self) -> TransportRequest<'request> {
        self.request
    }

    /// Returns the exact security domain whose bytes were signed.
    #[must_use]
    pub const fn context(&self) -> SigningContext<'request> {
        self.context
    }

    /// Returns the validated signature or MAC bytes.
    #[must_use]
    pub fn signature(&self) -> &[u8] {
        self.storage.get(..self.len).unwrap_or_default()
    }
}

impl fmt::Debug for SignedRequest<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SignedRequest")
            .field("request", &self.request)
            .field("context", &self.context)
            .field("signature_len", &self.len)
            .field("signature", &"[redacted]")
            .finish()
    }
}

impl Drop for SignedRequest<'_, '_> {
    fn drop(&mut self) {
        sanitize_bytes(self.storage);
    }
}
