//! Fixed response decode and continuation staging owned by a checked guard.

use core::fmt;

use cloud_sdk_sanitization::sanitize_bytes;

/// Decoder scratch bytes available to checked provider decoders.
pub const RESPONSE_DECODER_SCRATCH_BYTES: usize = 256;
/// Cursor staging bytes available to checked pagination decoders.
pub const RESPONSE_CURSOR_SCRATCH_BYTES: usize = 1024;
/// Provider-link staging bytes available to checked pagination decoders.
pub const RESPONSE_PROVIDER_LINK_SCRATCH_BYTES: usize = 2048;

/// Complete fixed response workspace cleared with its checked guard.
///
/// The workspace is intentionally neither `Copy` nor `Clone`. Providers may
/// use only the exact staging slice needed for one decode operation.
///
/// ```compile_fail
/// use cloud_sdk::transport::ResponseDecodeWorkspace;
///
/// fn require_copy<T: Copy>() {}
/// require_copy::<ResponseDecodeWorkspace>();
/// ```
pub struct ResponseDecodeWorkspace {
    decoder: [u8; RESPONSE_DECODER_SCRATCH_BYTES],
    cursor: [u8; RESPONSE_CURSOR_SCRATCH_BYTES],
    provider_link: [u8; RESPONSE_PROVIDER_LINK_SCRATCH_BYTES],
}

impl ResponseDecodeWorkspace {
    pub(crate) const fn new() -> Self {
        Self {
            decoder: [0; RESPONSE_DECODER_SCRATCH_BYTES],
            cursor: [0; RESPONSE_CURSOR_SCRATCH_BYTES],
            provider_link: [0; RESPONSE_PROVIDER_LINK_SCRATCH_BYTES],
        }
    }

    /// Creates an independently cleanup-owning provider decode workspace.
    #[must_use]
    pub const fn new_for_provider() -> Self {
        Self::new()
    }

    /// Returns decoder scratch storage.
    pub fn decoder_scratch_mut(&mut self) -> &mut [u8] {
        &mut self.decoder
    }

    /// Returns cursor staging storage.
    pub fn cursor_scratch_mut(&mut self) -> &mut [u8] {
        &mut self.cursor
    }

    /// Returns provider-link staging storage.
    pub fn provider_link_scratch_mut(&mut self) -> &mut [u8] {
        &mut self.provider_link
    }

    fn clear(&mut self) {
        sanitize_bytes(&mut self.decoder);
        sanitize_bytes(&mut self.cursor);
        sanitize_bytes(&mut self.provider_link);
    }
}

impl Drop for ResponseDecodeWorkspace {
    fn drop(&mut self) {
        self.clear();
    }
}

impl fmt::Debug for ResponseDecodeWorkspace {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ResponseDecodeWorkspace([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::ResponseDecodeWorkspace;

    #[test]
    fn complete_workspace_uses_one_clear_path() {
        let mut workspace = ResponseDecodeWorkspace::new();
        workspace.decoder.fill(0x11);
        workspace.cursor.fill(0x22);
        workspace.provider_link.fill(0x33);
        workspace.clear();
        assert!(workspace.decoder.iter().all(|byte| *byte == 0));
        assert!(workspace.cursor.iter().all(|byte| *byte == 0));
        assert!(workspace.provider_link.iter().all(|byte| *byte == 0));
    }
}
