#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
mod secret_text;

#[cfg(feature = "alloc")]
pub use secret_text::SecretText;

/// Volatile-clears an ordinary caller-owned byte buffer.
///
/// This delegates to the reviewed `sanitization` crate so the clear cannot be
/// removed as an ordinary dead store.
#[inline]
pub fn sanitize_bytes(bytes: &mut [u8]) {
    sanitization::sanitize_bytes(bytes);
}

/// Caller-owned byte buffer that is volatile-cleared when dropped.
///
/// The full borrowed slice is cleared on success, error, or early return. This
/// does not clear the source value or copies made by transports, operating
/// systems, crash handlers, or remote services.
pub struct SecretBuffer<'a> {
    bytes: &'a mut [u8],
}

impl<'a> SecretBuffer<'a> {
    /// Borrows a mutable byte slice until the guard is dropped.
    #[must_use]
    pub const fn new(bytes: &'a mut [u8]) -> Self {
        Self { bytes }
    }

    /// Returns the guarded bytes for request construction.
    #[must_use]
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        self.bytes
    }

    /// Returns the guarded bytes for transport.
    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        self.bytes
    }
}

impl Drop for SecretBuffer<'_> {
    fn drop(&mut self) {
        sanitize_bytes(self.bytes);
    }
}

impl core::fmt::Debug for SecretBuffer<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("SecretBuffer([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::{SecretBuffer, sanitize_bytes};

    #[test]
    fn explicit_sanitization_clears_every_byte() {
        let mut bytes = [0xa5_u8; 8];
        sanitize_bytes(&mut bytes);
        assert_eq!(bytes, [0; 8]);
    }

    #[test]
    fn guard_clears_its_full_buffer_on_drop() {
        let mut bytes = [0xa5_u8; 8];
        {
            let mut guarded = SecretBuffer::new(&mut bytes);
            if let Some(first) = guarded.as_mut_slice().first_mut() {
                *first = 0x42;
            }
            assert_eq!(guarded.as_slice().first(), Some(&0x42));
        }
        assert_eq!(bytes, [0; 8]);
    }

    #[test]
    fn guard_clears_after_an_early_error() {
        fn write_then_fail(output: &mut [u8]) -> Result<(), ()> {
            let mut guarded = SecretBuffer::new(output);
            if let Some(first) = guarded.as_mut_slice().first_mut() {
                *first = 0x42;
            }
            Err(())
        }

        let mut bytes = [0xa5_u8; 8];
        assert_eq!(write_then_fail(&mut bytes), Err(()));
        assert_eq!(bytes, [0; 8]);
    }
}
