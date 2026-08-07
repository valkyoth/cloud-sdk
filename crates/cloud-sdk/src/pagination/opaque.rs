use core::fmt;

use cloud_sdk_sanitization::{SecretBuffer, sanitize_bytes, sanitize_value};

use super::{PaginationError, PaginationLimits};

/// Maximum opaque cursor, marker, or provider-link target length.
pub const MAX_OPAQUE_STATE_BYTES: usize = 8192;
const DIGEST_BYTES: usize = 32;
const HISTORY_PREFIX_BYTES: usize = 2 + DIGEST_BYTES;

struct OpaqueState<'storage> {
    bytes: SecretBuffer<'storage>,
    len: usize,
}

impl<'storage> OpaqueState<'storage> {
    fn transfer_from(
        source: &mut [u8],
        destination: &'storage mut [u8],
        limits: PaginationLimits,
    ) -> Result<Self, PaginationError> {
        sanitize_bytes(destination);
        let source = SecretBuffer::new(source);
        let mut destination = SecretBuffer::new(destination);
        let value = source.as_slice();
        if value.is_empty() {
            return Err(PaginationError::MissingState);
        }
        if value.len() > limits.max_state_bytes() {
            return Err(PaginationError::StateTooLong);
        }
        let output = destination
            .as_mut_slice()
            .get_mut(..value.len())
            .ok_or(PaginationError::OutputTooSmall)?;
        output.copy_from_slice(value);
        Ok(Self {
            bytes: destination,
            len: value.len(),
        })
    }

    fn with_bytes<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        let value = self.bytes.as_slice().get(..self.len).unwrap_or_default();
        inspect(value)
    }
}

impl Drop for OpaqueState<'_> {
    fn drop(&mut self) {
        sanitize_value(&mut self.len);
    }
}

/// Cleanup-owning opaque cursor populated by atomic source transfer.
///
/// This type is intentionally neither `Copy` nor `Clone`.
///
/// ```compile_fail
/// use cloud_sdk::pagination::PaginationCursor;
/// fn require_copy<T: Copy>() {}
/// require_copy::<PaginationCursor<'static>>();
/// ```
pub struct PaginationCursor<'storage> {
    state: OpaqueState<'storage>,
}

impl<'storage> PaginationCursor<'storage> {
    /// Moves source bytes into cleanup-owning caller storage.
    ///
    /// Source and complete destination storage are cleared on every failure.
    /// Source is also cleared after a successful transfer.
    pub fn transfer_from(
        source: &mut [u8],
        destination: &'storage mut [u8],
        limits: PaginationLimits,
    ) -> Result<Self, PaginationError> {
        OpaqueState::transfer_from(source, destination, limits).map(|state| Self { state })
    }

    /// Runs a closure with the exact opaque cursor bytes.
    pub fn with_cursor<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        self.state.with_bytes(inspect)
    }

    pub(super) fn as_bytes(&self) -> &[u8] {
        self.state
            .bytes
            .as_slice()
            .get(..self.state.len)
            .unwrap_or_default()
    }
}

impl fmt::Debug for PaginationCursor<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PaginationCursor([redacted])")
    }
}

/// Cleanup-owning opaque marker populated by atomic source transfer.
///
/// This type is intentionally neither `Copy` nor `Clone`.
///
/// ```compile_fail
/// use cloud_sdk::pagination::PaginationMarker;
/// fn require_copy<T: Copy>() {}
/// require_copy::<PaginationMarker<'static>>();
/// ```
pub struct PaginationMarker<'storage> {
    state: OpaqueState<'storage>,
}

impl<'storage> PaginationMarker<'storage> {
    /// Moves source bytes into cleanup-owning caller storage.
    pub fn transfer_from(
        source: &mut [u8],
        destination: &'storage mut [u8],
        limits: PaginationLimits,
    ) -> Result<Self, PaginationError> {
        OpaqueState::transfer_from(source, destination, limits).map(|state| Self { state })
    }

    /// Runs a closure with the exact opaque marker bytes.
    pub fn with_marker<R>(&self, inspect: impl FnOnce(&[u8]) -> R) -> R {
        self.state.with_bytes(inspect)
    }
}

impl fmt::Debug for PaginationMarker<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("PaginationMarker([redacted])")
    }
}

/// Caller-produced fixed-size cursor digest.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CursorDigest([u8; DIGEST_BYTES]);

impl CursorDigest {
    /// Wraps a digest produced by the caller-selected digest implementation.
    #[must_use]
    pub const fn new(value: [u8; DIGEST_BYTES]) -> Self {
        Self(value)
    }
}

impl fmt::Debug for CursorDigest {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CursorDigest([redacted])")
    }
}

/// Cleanup-owning exact cursor history used for cycle and collision checks.
///
/// Each entry stores both the caller digest and exact cursor bytes. Equal
/// digests with unequal bytes fail as collisions; equal bytes with another
/// digest also fail closed. This type is intentionally neither `Copy` nor
/// `Clone`.
pub struct CursorHistory<'storage> {
    bytes: SecretBuffer<'storage>,
    used: usize,
    entries: u32,
    max_entries: u32,
}

impl<'storage> CursorHistory<'storage> {
    /// Creates empty history in caller-owned cleanup storage.
    pub fn new(storage: &'storage mut [u8], max_entries: u32) -> Result<Self, PaginationError> {
        sanitize_bytes(storage);
        if max_entries == 0 {
            return Err(PaginationError::ZeroLimit);
        }
        Ok(Self {
            bytes: SecretBuffer::new(storage),
            used: 0,
            entries: 0,
            max_entries,
        })
    }

    /// Checks and transactionally records one cursor and digest.
    pub fn observe(
        &mut self,
        cursor: &PaginationCursor<'_>,
        digest: CursorDigest,
    ) -> Result<(), PaginationError> {
        cursor
            .state
            .with_bytes(|state| self.observe_bytes(state, digest))
    }

    /// Returns the number of recorded cursors.
    #[must_use]
    pub const fn entries(&self) -> u32 {
        self.entries
    }

    fn observe_bytes(&mut self, state: &[u8], digest: CursorDigest) -> Result<(), PaginationError> {
        let mut position = 0_usize;
        while position < self.used {
            let prefix_end = position
                .checked_add(HISTORY_PREFIX_BYTES)
                .ok_or(PaginationError::HistoryBudgetExceeded)?;
            let prefix = self
                .bytes
                .as_slice()
                .get(position..prefix_end)
                .ok_or(PaginationError::HistoryBudgetExceeded)?;
            let state_len_bytes: [u8; 2] = prefix
                .get(..2)
                .ok_or(PaginationError::HistoryBudgetExceeded)?
                .try_into()
                .map_err(|_| PaginationError::HistoryBudgetExceeded)?;
            let state_len = usize::from(u16::from_be_bytes(state_len_bytes));
            let stored_digest = prefix
                .get(2..HISTORY_PREFIX_BYTES)
                .ok_or(PaginationError::HistoryBudgetExceeded)?;
            let state_end = prefix_end
                .checked_add(state_len)
                .ok_or(PaginationError::HistoryBudgetExceeded)?;
            let stored_state = self
                .bytes
                .as_slice()
                .get(prefix_end..state_end)
                .ok_or(PaginationError::HistoryBudgetExceeded)?;
            if stored_digest == digest.0 {
                return if stored_state == state {
                    Err(PaginationError::CursorCycle)
                } else {
                    Err(PaginationError::CursorDigestCollision)
                };
            }
            if stored_state == state {
                return Err(PaginationError::CursorDigestChanged);
            }
            position = state_end;
        }
        if self.entries >= self.max_entries {
            return Err(PaginationError::HistoryBudgetExceeded);
        }
        let state_len =
            u16::try_from(state.len()).map_err(|_| PaginationError::HistoryBudgetExceeded)?;
        let next_used = self
            .used
            .checked_add(HISTORY_PREFIX_BYTES)
            .and_then(|value| value.checked_add(state.len()))
            .ok_or(PaginationError::HistoryBudgetExceeded)?;
        let output = self
            .bytes
            .as_mut_slice()
            .get_mut(self.used..next_used)
            .ok_or(PaginationError::HistoryBudgetExceeded)?;
        output
            .get_mut(..2)
            .ok_or(PaginationError::HistoryBudgetExceeded)?
            .copy_from_slice(&state_len.to_be_bytes());
        output
            .get_mut(2..HISTORY_PREFIX_BYTES)
            .ok_or(PaginationError::HistoryBudgetExceeded)?
            .copy_from_slice(&digest.0);
        output
            .get_mut(HISTORY_PREFIX_BYTES..)
            .ok_or(PaginationError::HistoryBudgetExceeded)?
            .copy_from_slice(state);
        self.used = next_used;
        self.entries = self
            .entries
            .checked_add(1)
            .ok_or(PaginationError::HistoryBudgetExceeded)?;
        Ok(())
    }
}

impl Drop for CursorHistory<'_> {
    fn drop(&mut self) {
        sanitize_value(&mut self.used);
        sanitize_value(&mut self.entries);
    }
}

impl fmt::Debug for CursorHistory<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CursorHistory")
            .field("entries", &self.entries)
            .field("state", &"[redacted]")
            .finish()
    }
}
