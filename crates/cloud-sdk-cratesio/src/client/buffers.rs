use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes};

/// Separate caller-owned scratch regions. These must not alias credential,
/// request, metadata or archive inputs. All regions are cleared on every exit.
pub struct RegistryBuffers<'a> {
    /// Credential encoding scratch; use the credential kind's documented bound.
    pub credential: &'a mut [u8],
    /// Request JSON scratch; empty for bodyless operations.
    pub body: &'a mut [u8],
    /// Response body storage, capped by the client and checked decoder.
    pub response: &'a mut [u8],
    /// Retained response-header storage.
    pub headers: &'a mut [u8],
}

pub(super) struct Guard<'a> {
    credential: SecretBuffer<'a>,
    body: SecretBuffer<'a>,
    response: SecretBuffer<'a>,
    headers: SecretBuffer<'a>,
}
impl<'a> Guard<'a> {
    pub(super) fn new(buffers: RegistryBuffers<'a>) -> Self {
        sanitize_bytes(buffers.credential);
        sanitize_bytes(buffers.body);
        sanitize_bytes(buffers.response);
        sanitize_bytes(buffers.headers);
        Self {
            credential: SecretBuffer::new(buffers.credential),
            body: SecretBuffer::new(buffers.body),
            response: SecretBuffer::new(buffers.response),
            headers: SecretBuffer::new(buffers.headers),
        }
    }
    pub(super) fn parts(&mut self) -> RegistryBuffers<'_> {
        RegistryBuffers {
            credential: self.credential.as_mut_slice(),
            body: self.body.as_mut_slice(),
            response: self.response.as_mut_slice(),
            headers: self.headers.as_mut_slice(),
        }
    }
}
