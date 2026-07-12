//! Bounded fixture response bodies.

use core::fmt;

/// Maximum fixture body length, including one byte beyond the common 8 MiB
/// response-policy ceiling for oversized-input tests.
pub const MAX_FIXTURE_BODY_BYTES: usize = 8_388_609;

/// Fixture body construction or write error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FixtureBodyError {
    /// The represented body exceeds [`MAX_FIXTURE_BODY_BYTES`].
    TooLarge,
    /// The caller-owned destination is smaller than the represented body.
    OutputTooSmall,
}

/// Borrowed or compact repeated-byte fixture body.
#[derive(Clone, Copy, Eq, PartialEq)]
pub enum FixtureBody<'a> {
    /// Borrowed response bytes.
    Bytes(&'a [u8]),
    /// A repeated byte with a bounded logical length.
    Repeated {
        /// Byte copied into the response destination.
        byte: u8,
        /// Logical body length.
        len: usize,
    },
}

impl<'a> FixtureBody<'a> {
    /// Creates a bounded borrowed body.
    pub const fn new(bytes: &'a [u8]) -> Result<Self, FixtureBodyError> {
        if bytes.len() > MAX_FIXTURE_BODY_BYTES {
            return Err(FixtureBodyError::TooLarge);
        }
        Ok(Self::Bytes(bytes))
    }

    /// Creates a compact repeated-byte body.
    pub const fn repeated(byte: u8, len: usize) -> Result<Self, FixtureBodyError> {
        if len > MAX_FIXTURE_BODY_BYTES {
            return Err(FixtureBodyError::TooLarge);
        }
        Ok(Self::Repeated { byte, len })
    }

    /// Returns the represented body length.
    #[must_use]
    pub const fn len(self) -> usize {
        match self {
            Self::Bytes(bytes) => bytes.len(),
            Self::Repeated { len, .. } => len,
        }
    }

    /// Reports whether the represented body is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.len() == 0
    }

    /// Returns borrowed bytes when this is a literal fixture.
    #[must_use]
    pub const fn as_bytes(self) -> Option<&'a [u8]> {
        match self {
            Self::Bytes(bytes) => Some(bytes),
            Self::Repeated { .. } => None,
        }
    }

    /// Atomically writes the represented body into a caller-owned buffer.
    pub fn write_to(self, output: &mut [u8]) -> Result<usize, FixtureBodyError> {
        let len = self.len();
        let target = output
            .get_mut(..len)
            .ok_or(FixtureBodyError::OutputTooSmall)?;
        match self {
            Self::Bytes(bytes) => target.copy_from_slice(bytes),
            Self::Repeated { byte, .. } => target.fill(byte),
        }
        Ok(len)
    }
}

impl fmt::Debug for FixtureBody<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FixtureBody")
            .field("contents", &"[redacted]")
            .field("len", &self.len())
            .finish()
    }
}
