//! Owned volatile-clearing UTF-8 storage.

use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use crate::sanitize_bytes;

/// Owned UTF-8 text whose initialized bytes are volatile-cleared on drop.
///
/// Construction consumes a `String` and reuses its allocation without making
/// another plaintext copy. Cloning intentionally creates another independently
/// cleared allocation. This cannot clear earlier source copies, allocator
/// metadata, swap, core dumps, debugger snapshots, or downstream copies.
#[derive(Clone, Eq, PartialEq)]
pub struct SecretText {
    bytes: Vec<u8>,
}

impl SecretText {
    /// Takes ownership of a validated UTF-8 string without copying its bytes.
    #[must_use]
    pub fn new(value: String) -> Self {
        Self {
            bytes: value.into_bytes(),
        }
    }

    /// Exposes the secret text through an explicit accessor.
    #[must_use]
    pub fn expose_secret(&self) -> &str {
        core::str::from_utf8(&self.bytes).unwrap_or_default()
    }

    fn sanitize(&mut self) {
        sanitize_bytes(&mut self.bytes);
    }
}

impl Drop for SecretText {
    fn drop(&mut self) {
        self.sanitize();
    }
}

impl fmt::Debug for SecretText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SecretText([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::SecretText;

    #[test]
    fn owns_utf8_without_exposing_debug_output() {
        let secret = SecretText::new(String::from("sensitive"));
        assert_eq!(secret.expose_secret(), "sensitive");
        assert_eq!(alloc::format!("{secret:?}"), "SecretText([redacted])");
    }

    #[test]
    fn drop_uses_the_reviewed_byte_sanitizer() {
        let mut secret = SecretText::new(String::from("sensitive"));
        secret.sanitize();
        assert!(secret.bytes.iter().all(|byte| *byte == 0));
    }
}
