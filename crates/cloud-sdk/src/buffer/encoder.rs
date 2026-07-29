//! Transactional two-pass encoding over immutable snapshots.

use cloud_sdk_sanitization::sanitize_bytes;

/// Fixed-buffer snapshot encoder used by provider request components.
///
/// Instances are created only by [`encode_snapshot`] and
/// [`encode_snapshot_bounded`]. The first pass measures with checked
/// arithmetic; the second pass receives an exactly sized destination.
pub struct SnapshotEncoder<'output, E> {
    destination: Destination<'output>,
    len: usize,
    max_len: usize,
    error: E,
}

enum Destination<'output> {
    Measure,
    Output(&'output mut [u8]),
    Verify(&'output [u8]),
}

impl<'output, E: Copy> SnapshotEncoder<'output, E> {
    fn measuring(max_len: usize, error: E) -> Self {
        Self {
            destination: Destination::Measure,
            len: 0,
            max_len,
            error,
        }
    }

    fn writing(output: &'output mut [u8], max_len: usize, error: E) -> Self {
        Self {
            destination: Destination::Output(output),
            len: 0,
            max_len,
            error,
        }
    }

    fn verifying(output: &'output [u8], max_len: usize, error: E) -> Self {
        Self {
            destination: Destination::Verify(output),
            len: 0,
            max_len,
            error,
        }
    }

    /// Returns the number of bytes measured or written.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Reports whether no bytes have been measured or written.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Appends one byte.
    pub fn byte(&mut self, value: u8) -> Result<(), E> {
        self.bytes(core::slice::from_ref(&value))
    }

    /// Appends bytes without escaping.
    pub fn bytes(&mut self, value: &[u8]) -> Result<(), E> {
        let end = self.checked_end(value.len())?;
        if let Destination::Output(output) = &mut self.destination {
            let target = output.get_mut(self.len..end).ok_or(self.error)?;
            target.copy_from_slice(value);
        } else if let Destination::Verify(output) = &self.destination {
            let actual = output.get(self.len..end).ok_or(self.error)?;
            if actual != value {
                return Err(self.error);
            }
        }
        self.len = end;
        Ok(())
    }

    /// Appends UTF-8 without escaping.
    pub fn string(&mut self, value: &str) -> Result<(), E> {
        self.bytes(value.as_bytes())
    }

    /// Appends a base-10 unsigned integer.
    pub fn u64(&mut self, mut value: u64) -> Result<(), E> {
        if value == 0 {
            return self.byte(b'0');
        }

        let mut digits = [0_u8; 20];
        let mut cursor = digits.len();
        while value != 0 {
            cursor = cursor.checked_sub(1).ok_or(self.error)?;
            let digit = u8::try_from(value % 10).map_err(|_| self.error)?;
            let slot = digits.get_mut(cursor).ok_or(self.error)?;
            *slot = b'0'.checked_add(digit).ok_or(self.error)?;
            value /= 10;
        }
        self.bytes(digits.get(cursor..).ok_or(self.error)?)
    }

    /// Appends one RFC 3986 percent-encoded component.
    pub fn percent_encoded(&mut self, value: &str) -> Result<(), E> {
        for byte in value.bytes() {
            if super::is_unreserved(byte) {
                self.byte(byte)?;
            } else {
                self.byte(b'%')?;
                self.byte(super::hex_digit(byte >> 4))?;
                self.byte(super::hex_digit(byte & 0x0f))?;
            }
        }
        Ok(())
    }

    /// Appends JSON string contents without surrounding quotes.
    pub fn json_string_escaped(&mut self, value: &str) -> Result<(), E> {
        for byte in value.bytes() {
            match byte {
                b'"' => self.string("\\\"")?,
                b'\\' => self.string("\\\\")?,
                b'\n' => self.string("\\n")?,
                b'\r' => self.string("\\r")?,
                b'\t' => self.string("\\t")?,
                0x00..=0x1f => {
                    self.string("\\u00")?;
                    self.byte(super::hex_digit(byte >> 4))?;
                    self.byte(super::hex_digit(byte & 0x0f))?;
                }
                _ => self.byte(byte)?,
            }
        }
        Ok(())
    }

    /// Appends a complete JSON string.
    pub fn json_string(&mut self, value: &str) -> Result<(), E> {
        self.byte(b'"')?;
        self.json_string_escaped(value)?;
        self.byte(b'"')
    }

    /// Appends a percent-encoded query pair.
    pub fn query_pair(&mut self, first: &mut bool, key: &str, value: &str) -> Result<(), E> {
        self.query_separator(first)?;
        self.percent_encoded(key)?;
        self.byte(b'=')?;
        self.percent_encoded(value)
    }

    /// Appends a percent-encoded query key and base-10 integer.
    pub fn query_u64(&mut self, first: &mut bool, key: &str, value: u64) -> Result<(), E> {
        self.query_separator(first)?;
        self.percent_encoded(key)?;
        self.byte(b'=')?;
        self.u64(value)
    }

    /// Appends `&` unless this is the first query pair.
    pub fn query_separator(&mut self, first: &mut bool) -> Result<(), E> {
        if *first {
            *first = false;
            Ok(())
        } else {
            self.byte(b'&')
        }
    }

    fn checked_end(&self, additional: usize) -> Result<usize, E> {
        let end = self.len.checked_add(additional).ok_or(self.error)?;
        if end > self.max_len {
            return Err(self.error);
        }
        Ok(end)
    }
}

/// Encodes one immutable `Copy` snapshot after an exact measurement pass.
///
/// `encode` is a function pointer, not a capturing closure. Both passes
/// therefore receive only the same by-value snapshot. An undersized output is
/// unchanged. A final compare-only pass checks every emitted byte exactly. If
/// either later pass violates the measured contract, the exact admitted
/// destination is cleared before the error is returned.
pub fn encode_snapshot<S: Copy, E: Copy>(
    snapshot: S,
    output: &mut [u8],
    error: E,
    encode: for<'encoder> fn(S, &mut SnapshotEncoder<'encoder, E>) -> Result<(), E>,
) -> Result<usize, E> {
    encode_snapshot_bounded(snapshot, output, usize::MAX, error, encode)
}

/// Measures one immutable snapshot without writing output.
pub fn measure_snapshot<S: Copy, E: Copy>(
    snapshot: S,
    error: E,
    encode: for<'encoder> fn(S, &mut SnapshotEncoder<'encoder, E>) -> Result<(), E>,
) -> Result<usize, E> {
    measure_snapshot_bounded(snapshot, usize::MAX, error, encode)
}

/// Measures one immutable snapshot under an aggregate byte cap.
pub fn measure_snapshot_bounded<S: Copy, E: Copy>(
    snapshot: S,
    max_len: usize,
    error: E,
    encode: for<'encoder> fn(S, &mut SnapshotEncoder<'encoder, E>) -> Result<(), E>,
) -> Result<usize, E> {
    let mut measure = SnapshotEncoder::measuring(max_len, error);
    encode(snapshot, &mut measure)?;
    Ok(measure.len())
}

/// Encodes one immutable snapshot under an aggregate byte cap.
///
/// Capacity and aggregate-limit failures occur before the output is modified.
pub fn encode_snapshot_bounded<S: Copy, E: Copy>(
    snapshot: S,
    output: &mut [u8],
    max_len: usize,
    error: E,
    encode: for<'encoder> fn(S, &mut SnapshotEncoder<'encoder, E>) -> Result<(), E>,
) -> Result<usize, E> {
    let required = measure_snapshot_bounded(snapshot, max_len, error, encode)?;
    let target = output.get_mut(..required).ok_or(error)?;

    let write_matches = {
        let mut writer = SnapshotEncoder::writing(target, max_len, error);
        encode(snapshot, &mut writer).is_ok() && writer.len() == required
    };
    if !write_matches {
        sanitize_bytes(target);
        return Err(error);
    }

    let verify_matches = {
        let mut verifier = SnapshotEncoder::verifying(target, max_len, error);
        encode(snapshot, &mut verifier).is_ok() && verifier.len() == required
    };
    if !verify_matches {
        sanitize_bytes(target);
        return Err(error);
    }
    Ok(required)
}

#[cfg(test)]
mod tests {
    extern crate std;

    use core::sync::atomic::{AtomicUsize, Ordering};

    use super::{SnapshotEncoder, encode_snapshot, encode_snapshot_bounded};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum TestError {
        Rejected,
    }

    #[test]
    fn exact_snapshot_encoding_preserves_the_tail() {
        let mut output = [0xA5_u8; 16];
        let result = encode_snapshot(
            ("key", 42_u64),
            &mut output,
            TestError::Rejected,
            |(key, value), encoder| {
                encoder.string(key)?;
                encoder.byte(b'=')?;
                encoder.u64(value)
            },
        );

        assert_eq!(result, Ok(6));
        assert_eq!(output.get(..6), Some(b"key=42".as_slice()));
        assert!(
            output
                .get(6..)
                .is_some_and(|tail| tail.iter().all(|b| *b == 0xA5))
        );
    }

    #[test]
    fn every_undersized_capacity_is_unchanged() {
        for capacity in 0..6 {
            let mut output = [0xA5_u8; 6];
            let result = encode_snapshot(
                "secret",
                output.get_mut(..capacity).unwrap_or_default(),
                TestError::Rejected,
                |value, encoder| encoder.string(value),
            );
            assert_eq!(result, Err(TestError::Rejected));
            assert_eq!(output, [0xA5; 6]);
        }
    }

    #[test]
    fn aggregate_cap_is_checked_before_writing() {
        let mut output = [0xA5_u8; 8];
        assert_eq!(
            encode_snapshot_bounded(
                "1234",
                &mut output,
                3,
                TestError::Rejected,
                |value, encoder| encoder.string(value),
            ),
            Err(TestError::Rejected)
        );
        assert_eq!(output, [0xA5; 8]);
    }

    #[test]
    fn arithmetic_overflow_is_reported() {
        let mut encoder = SnapshotEncoder::measuring(usize::MAX, TestError::Rejected);
        encoder.len = usize::MAX;
        assert_eq!(encoder.byte(b'x'), Err(TestError::Rejected));
    }

    #[test]
    fn nondeterministic_same_length_output_is_rejected_and_cleared() {
        static PASS: AtomicUsize = AtomicUsize::new(0);

        fn changing(
            _snapshot: (),
            encoder: &mut SnapshotEncoder<'_, TestError>,
        ) -> Result<(), TestError> {
            let pass = PASS.fetch_add(1, Ordering::Relaxed);
            encoder.byte(if pass == 2 { b'B' } else { b'A' })
        }

        PASS.store(0, Ordering::Relaxed);
        let mut output = [0xA5_u8; 4];
        assert_eq!(
            encode_snapshot((), &mut output, TestError::Rejected, changing),
            Err(TestError::Rejected)
        );
        assert_eq!(output, [0, 0xA5, 0xA5, 0xA5]);
    }
}
