//! no_std fixed-buffer writing helpers for provider crates.

/// Writes one byte into a caller-owned buffer.
pub fn write_byte<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    byte: u8,
    error: E,
) -> Result<(), E> {
    let slot = output.get_mut(*len).ok_or(error)?;
    *slot = byte;
    *len = len.checked_add(1).ok_or(error)?;
    Ok(())
}

/// Writes a string into a caller-owned buffer without escaping.
pub fn write_str<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    value: &str,
    error: E,
) -> Result<(), E> {
    for byte in value.bytes() {
        write_byte(output, len, byte, error)?;
    }
    Ok(())
}

/// Writes a base-10 unsigned integer into a caller-owned buffer.
pub fn write_u64<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    mut value: u64,
    error: E,
) -> Result<(), E> {
    if value == 0 {
        return write_byte(output, len, b'0', error);
    }

    let mut digits = [0u8; 20];
    let mut cursor = digits.len();
    while value != 0 {
        cursor = cursor.checked_sub(1).ok_or(error)?;
        let digit = u8::try_from(value % 10).map_err(|_| error)?;
        let slot = digits.get_mut(cursor).ok_or(error)?;
        *slot = b'0'.checked_add(digit).ok_or(error)?;
        value /= 10;
    }

    let encoded = digits.get(cursor..).ok_or(error)?;
    for byte in encoded {
        write_byte(output, len, *byte, error)?;
    }
    Ok(())
}

/// Writes a percent-encoded query component into a caller-owned buffer.
pub fn write_percent_encoded<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    value: &str,
    error: E,
) -> Result<(), E> {
    for byte in value.bytes() {
        if is_unreserved(byte) {
            write_byte(output, len, byte, error)?;
        } else {
            write_byte(output, len, b'%', error)?;
            write_byte(output, len, hex_digit(byte >> 4), error)?;
            write_byte(output, len, hex_digit(byte & 0x0f), error)?;
        }
    }
    Ok(())
}

/// Writes `&` unless this is the first query pair.
pub fn write_query_separator<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    error: E,
) -> Result<(), E> {
    if *first {
        *first = false;
        return Ok(());
    }
    write_byte(output, len, b'&', error)
}

/// Writes a percent-encoded query key/value pair.
pub fn write_query_pair<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: &str,
    error: E,
) -> Result<(), E> {
    write_query_separator(output, len, first, error)?;
    write_percent_encoded(output, len, key, error)?;
    write_byte(output, len, b'=', error)?;
    write_percent_encoded(output, len, value, error)
}

/// Writes a percent-encoded query key and base-10 integer value.
pub fn write_query_u64<E: Copy>(
    output: &mut [u8],
    len: &mut usize,
    first: &mut bool,
    key: &str,
    value: u64,
    error: E,
) -> Result<(), E> {
    write_query_separator(output, len, first, error)?;
    write_percent_encoded(output, len, key, error)?;
    write_byte(output, len, b'=', error)?;
    write_u64(output, len, value, error)
}

const fn is_unreserved(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

const fn hex_digit(nibble: u8) -> u8 {
    match nibble {
        0 => b'0',
        1 => b'1',
        2 => b'2',
        3 => b'3',
        4 => b'4',
        5 => b'5',
        6 => b'6',
        7 => b'7',
        8 => b'8',
        9 => b'9',
        10 => b'A',
        11 => b'B',
        12 => b'C',
        13 => b'D',
        14 => b'E',
        _ => b'F',
    }
}

#[cfg(test)]
mod tests {
    use super::{write_percent_encoded, write_query_pair, write_query_u64, write_str, write_u64};

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum TestError {
        TooSmall,
    }

    #[test]
    fn writes_decimal_numbers_including_zero() {
        let mut output = [0u8; 24];
        let mut len = 0;
        assert_eq!(
            write_u64(&mut output, &mut len, 0, TestError::TooSmall),
            Ok(())
        );
        assert_eq!(
            write_str(&mut output, &mut len, ",", TestError::TooSmall),
            Ok(())
        );
        assert_eq!(
            write_u64(&mut output, &mut len, u64::MAX, TestError::TooSmall),
            Ok(())
        );
        let written = output
            .get(..len)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(written, Some("0,18446744073709551615"));
    }

    #[test]
    fn reports_too_small_buffers() {
        let mut output = [0u8; 2];
        let mut len = 0;
        assert_eq!(
            write_u64(&mut output, &mut len, 100, TestError::TooSmall),
            Err(TestError::TooSmall)
        );
    }

    #[test]
    fn writes_percent_encoded_components_and_pairs() {
        let mut output = [0u8; 64];
        let mut len = 0;
        assert_eq!(
            write_percent_encoded(&mut output, &mut len, "env=prod", TestError::TooSmall),
            Ok(())
        );
        let encoded = output
            .get(..len)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(encoded, Some("env%3Dprod"));

        let mut output = [0u8; 64];
        let mut len = 0;
        let mut first = true;
        assert_eq!(
            write_query_pair(
                &mut output,
                &mut len,
                &mut first,
                "label_selector",
                "env=prod",
                TestError::TooSmall,
            ),
            Ok(())
        );
        assert_eq!(
            write_query_u64(
                &mut output,
                &mut len,
                &mut first,
                "page",
                0,
                TestError::TooSmall
            ),
            Ok(())
        );
        let query = output
            .get(..len)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(query, Some("label_selector=env%3Dprod&page=0"));
    }
}
