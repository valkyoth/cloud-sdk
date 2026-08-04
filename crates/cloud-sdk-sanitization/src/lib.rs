#![no_std]
#![doc = include_str!("../README.md")]

#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub use sanitization::SecretString;

/// Failure while fallibly appending to protected UTF-8 storage.
#[cfg(feature = "alloc")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum SecretStringAppendError {
    /// The resulting public byte length exceeds the caller's bound.
    TooLong,
    /// The resulting public byte length overflowed `usize`.
    CapacityOverflow,
    /// Protected replacement storage could not be allocated.
    Allocation,
    /// The protected string's internal UTF-8 invariant was not satisfied.
    InvalidUtf8,
}

#[cfg(feature = "alloc")]
impl core::fmt::Display for SecretStringAppendError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str(match self {
            Self::TooLong => "secret string length limit exceeded",
            Self::CapacityOverflow => "secret string capacity overflowed",
            Self::Allocation => "secret string allocation failed",
            Self::InvalidUtf8 => "secret string UTF-8 invariant failed",
        })
    }
}

#[cfg(feature = "alloc")]
impl core::error::Error for SecretStringAppendError {}

/// Fallibly appends text to protected storage within a public byte bound.
///
/// Growth allocates and fills replacement storage before clearing the old
/// allocation and swapping it out. Appends within existing capacity cannot
/// allocate. The borrowed source is not cleared.
#[cfg(feature = "alloc")]
pub fn try_append_secret_string(
    value: &mut SecretString,
    text: &str,
    maximum_bytes: usize,
) -> Result<(), SecretStringAppendError> {
    let required = value
        .len()
        .checked_add(text.len())
        .ok_or(SecretStringAppendError::CapacityOverflow)?;
    if required > maximum_bytes {
        return Err(SecretStringAppendError::TooLong);
    }
    if required <= value.capacity() {
        value.push_str(text);
        return Ok(());
    }

    let capacity = value
        .capacity()
        .max(1)
        .saturating_mul(2)
        .max(required)
        .min(maximum_bytes);
    let mut replacement = SecretString::try_with_capacity(capacity)
        .map_err(|_| SecretStringAppendError::Allocation)?;
    value
        .try_with_secret(|current| {
            replacement.push_str(current);
            replacement.push_str(text);
        })
        .map_err(|_| SecretStringAppendError::InvalidUtf8)?;

    value.clear_secret();
    core::mem::swap(value, &mut replacement);
    Ok(())
}

/// Volatile-clears an ordinary caller-owned byte buffer.
///
/// This delegates to the reviewed `sanitization` crate so the clear cannot be
/// removed as an ordinary dead store.
#[inline]
pub fn sanitize_bytes(bytes: &mut [u8]) {
    sanitization::wipe::bytes(bytes);
}

/// Volatile-clears one value through its reviewed field-wise sanitizer.
///
/// This is intended for fixed scalar bookkeeping and aggregates whose
/// `SecureSanitize` implementation has been explicitly reviewed.
#[inline]
pub fn sanitize_value<T: sanitization::SecureSanitize + ?Sized>(value: &mut T) {
    sanitization::SecureSanitize::secure_sanitize(value);
}

/// Volatile-clears an owned UTF-8 allocation's complete capacity.
#[cfg(feature = "alloc")]
#[inline]
pub fn sanitize_string(value: &mut alloc::string::String) {
    sanitization::wipe::string(value);
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
    #[cfg(feature = "alloc")]
    extern crate alloc;

    #[cfg(feature = "alloc")]
    use super::SecretString;
    #[cfg(feature = "alloc")]
    use super::sanitize_string;
    use super::{SecretBuffer, sanitize_bytes, sanitize_value};
    #[cfg(feature = "alloc")]
    use super::{SecretStringAppendError, try_append_secret_string};

    #[test]
    fn explicit_sanitization_clears_every_byte() {
        let mut bytes = [0xa5_u8; 8];
        sanitize_bytes(&mut bytes);
        assert_eq!(bytes, [0; 8]);
    }

    #[test]
    fn scalar_sanitization_uses_the_same_audited_boundary() {
        let mut value = usize::MAX;
        sanitize_value(&mut value);
        assert_eq!(value, 0);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn owned_string_sanitization_clears_length_and_complete_capacity() {
        let mut value = alloc::string::String::with_capacity(32);
        value.push_str("sensitive key");
        sanitize_string(&mut value);
        assert!(value.is_empty());
        assert_eq!(value.capacity(), 32);
    }

    #[test]
    fn guard_clears_its_full_buffer_on_drop() {
        let mut bytes = [0xa5_u8; 8];
        {
            let mut guarded = SecretBuffer::new(&mut bytes);
            let Some(first) = guarded.as_mut_slice().first_mut() else {
                unreachable!("security fixture construction failed");
            };
            *first = 0x42;

            assert_eq!(guarded.as_slice().first(), Some(&0x42));
        }
        assert_eq!(bytes, [0; 8]);
    }

    #[test]
    fn guard_clears_after_an_early_error() {
        fn write_then_fail(output: &mut [u8]) -> Result<(), ()> {
            let mut guarded = SecretBuffer::new(output);
            let Some(first) = guarded.as_mut_slice().first_mut() else {
                unreachable!("security fixture construction failed");
            };
            *first = 0x42;
            Err(())
        }

        let mut bytes = [0xa5_u8; 8];
        assert_eq!(write_then_fail(&mut bytes), Err(()));
        assert_eq!(bytes, [0; 8]);
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn reexported_secret_string_uses_scoped_access_and_redacted_debug() {
        let secret = SecretString::from_string(alloc::string::String::from("temporary secret"));

        assert_eq!(
            secret.try_with_secret(|value| value == "temporary secret"),
            Ok(true)
        );
        assert!(!alloc::format!("{secret:?}").contains("temporary secret"));
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn protected_string_append_grows_fallibly_within_its_bound() {
        let mut secret =
            SecretString::try_with_capacity(2).unwrap_or_else(|_| SecretString::empty());
        assert_eq!(try_append_secret_string(&mut secret, "ab", 8), Ok(()));
        assert_eq!(try_append_secret_string(&mut secret, "cdef", 8), Ok(()));
        assert_eq!(secret.try_with_secret(|text| text == "abcdef"), Ok(true));
        assert_eq!(
            try_append_secret_string(&mut secret, "ghi", 8),
            Err(SecretStringAppendError::TooLong)
        );
        assert_eq!(secret.try_with_secret(|text| text == "abcdef"), Ok(true));
    }
}
