use cloud_sdk_sanitization::{SecretBoxBytes, sanitize_bytes, sanitize_value};

use super::protected::ProtectedValueError;

pub(super) enum AddressFamily {
    Any,
    V4,
    V6,
}

struct Octets<const N: usize>([u8; N]);

impl<const N: usize> Drop for Octets<N> {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.0);
    }
}

struct Segments([u16; 8]);

impl Drop for Segments {
    fn drop(&mut self) {
        sanitize_value(&mut self.0);
    }
}

struct ProtectedByte(u8);

impl Drop for ProtectedByte {
    fn drop(&mut self) {
        sanitize_value(&mut self.0);
    }
}

struct ProtectedWord(u16);

impl Drop for ProtectedWord {
    fn drop(&mut self) {
        sanitize_value(&mut self.0);
    }
}

pub(super) fn address(
    text: &str,
    expected: AddressFamily,
) -> Result<SecretBoxBytes, ProtectedValueError> {
    if text.as_bytes().contains(&b':') {
        if matches!(expected, AddressFamily::V4) {
            return Err(ProtectedValueError);
        }
        ipv6(text)
    } else {
        if matches!(expected, AddressFamily::V6) {
            return Err(ProtectedValueError);
        }
        ipv4(text)
    }
}

pub(super) fn subnet(text: &str, prefix: &str) -> Result<SecretBoxBytes, ProtectedValueError> {
    let mut parsed_prefix = ProtectedByte(0);
    parse_decimal_byte_into(prefix.as_bytes(), &mut parsed_prefix.0)?;
    let address = address(text, AddressFamily::Any)?;
    let canonical = address.with_secret(|bytes| canonical_network(bytes, parsed_prefix.0));
    if !canonical {
        return Err(ProtectedValueError);
    }
    let len = address.len().checked_add(1).ok_or(ProtectedValueError)?;
    SecretBoxBytes::try_from_fn_bounded(len, 18, |index| {
        Ok::<u8, core::convert::Infallible>(if index.checked_add(1) == Some(len) {
            parsed_prefix.0
        } else {
            address.with_secret(|bytes| bytes.get(index).copied().unwrap_or(0))
        })
    })
    .map_err(|_| ProtectedValueError)
}

pub(super) fn date(text: &str) -> Result<SecretBoxBytes, ProtectedValueError> {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return Err(ProtectedValueError);
    }
    let mut parts = Octets([0_u8; 4]);
    let mut year = ProtectedWord(0);
    for byte in bytes.get(..4).ok_or(ProtectedValueError)? {
        decimal_step_u16(&mut year.0, *byte)?;
    }
    parts.0[0] = u8::try_from(year.0 >> 8).map_err(|_| ProtectedValueError)?;
    parts.0[1] = u8::try_from(year.0 & 0xff).map_err(|_| ProtectedValueError)?;
    parse_decimal_byte_into(bytes.get(5..7).ok_or(ProtectedValueError)?, &mut parts.0[2])?;
    parse_decimal_byte_into(
        bytes.get(8..10).ok_or(ProtectedValueError)?,
        &mut parts.0[3],
    )?;
    let valid = year.0 != 0
        && (1..=12).contains(&parts.0[2])
        && parts.0[3] != 0
        && parts.0[3] <= days_in_month(year.0, parts.0[2]);
    if !valid {
        return Err(ProtectedValueError);
    }
    SecretBoxBytes::try_from_slice(&parts.0, 4).map_err(|_| ProtectedValueError)
}

fn ipv4(text: &str) -> Result<SecretBoxBytes, ProtectedValueError> {
    let mut octets = Octets([0_u8; 4]);
    parse_ipv4_into(text, &mut octets.0)?;
    SecretBoxBytes::try_from_fn_bounded(5, 5, |index| {
        Ok::<u8, core::convert::Infallible>(if index == 0 {
            4
        } else {
            octets.0.get(index.saturating_sub(1)).copied().unwrap_or(0)
        })
    })
    .map_err(|_| ProtectedValueError)
}

fn ipv6(text: &str) -> Result<SecretBoxBytes, ProtectedValueError> {
    let mut segments = Segments([0_u16; 8]);
    parse_ipv6_into(text, &mut segments.0)?;
    SecretBoxBytes::try_from_fn_bounded(17, 17, |index| {
        Ok::<u8, core::convert::Infallible>(if index == 0 {
            6
        } else {
            let segment = segments
                .0
                .get(index.saturating_sub(1) / 2)
                .copied()
                .unwrap_or(0);
            if index % 2 == 1 {
                u8::try_from(segment >> 8).unwrap_or(0)
            } else {
                u8::try_from(segment & 0xff).unwrap_or(0)
            }
        })
    })
    .map_err(|_| ProtectedValueError)
}

fn parse_ipv4_into(text: &str, output: &mut [u8; 4]) -> Result<(), ProtectedValueError> {
    let mut parts = text.split('.');
    for target in output {
        let part = parts.next().ok_or(ProtectedValueError)?;
        if part.is_empty() || (part.len() > 1 && part.starts_with('0')) || part.len() > 3 {
            return Err(ProtectedValueError);
        }
        parse_decimal_byte_into(part.as_bytes(), target)?;
    }
    if parts.next().is_some() {
        return Err(ProtectedValueError);
    }
    Ok(())
}

fn parse_ipv6_into(text: &str, output: &mut [u16; 8]) -> Result<(), ProtectedValueError> {
    if text.is_empty() || text.matches("::").count() > 1 {
        return Err(ProtectedValueError);
    }
    if let Some((left, right)) = text.split_once("::") {
        let left_units = section_units(left)?;
        let right_units = section_units(right)?;
        if left_units
            .checked_add(right_units)
            .ok_or(ProtectedValueError)?
            >= 8
        {
            return Err(ProtectedValueError);
        }
        parse_section(left, output, 0)?;
        parse_section(right, output, 8_usize.saturating_sub(right_units))?;
        return Ok(());
    }
    if section_units(text)? != 8 {
        return Err(ProtectedValueError);
    }
    parse_section(text, output, 0)
}

fn section_units(section: &str) -> Result<usize, ProtectedValueError> {
    if section.is_empty() {
        return Ok(0);
    }
    let mut units = 0_usize;
    let mut pieces = section.split(':').peekable();
    while let Some(piece) = pieces.next() {
        if piece.is_empty() {
            return Err(ProtectedValueError);
        }
        let width = if piece.as_bytes().contains(&b'.') {
            if pieces.peek().is_some() {
                return Err(ProtectedValueError);
            }
            2
        } else {
            1
        };
        units = units.checked_add(width).ok_or(ProtectedValueError)?;
    }
    Ok(units)
}

fn parse_section(
    section: &str,
    output: &mut [u16; 8],
    mut index: usize,
) -> Result<(), ProtectedValueError> {
    if section.is_empty() {
        return Ok(());
    }
    for piece in section.split(':') {
        if piece.as_bytes().contains(&b'.') {
            let mut octets = Octets([0_u8; 4]);
            parse_ipv4_into(piece, &mut octets.0)?;
            *output.get_mut(index).ok_or(ProtectedValueError)? =
                (u16::from(octets.0[0]) << 8) | u16::from(octets.0[1]);
            index = index.checked_add(1).ok_or(ProtectedValueError)?;
            *output.get_mut(index).ok_or(ProtectedValueError)? =
                (u16::from(octets.0[2]) << 8) | u16::from(octets.0[3]);
        } else {
            if piece.is_empty() || piece.len() > 4 {
                return Err(ProtectedValueError);
            }
            let target = output.get_mut(index).ok_or(ProtectedValueError)?;
            for byte in piece.bytes() {
                let nibble = ProtectedWord(match byte {
                    b'0'..=b'9' => u16::from(byte.saturating_sub(b'0')),
                    b'a'..=b'f' => u16::from(byte.saturating_sub(b'a').saturating_add(10)),
                    b'A'..=b'F' => u16::from(byte.saturating_sub(b'A').saturating_add(10)),
                    _ => return Err(ProtectedValueError),
                });
                *target = target
                    .checked_mul(16)
                    .and_then(|value| value.checked_add(nibble.0))
                    .ok_or(ProtectedValueError)?;
            }
        }
        index = index.checked_add(1).ok_or(ProtectedValueError)?;
    }
    Ok(())
}

fn parse_decimal_byte_into(bytes: &[u8], value: &mut u8) -> Result<(), ProtectedValueError> {
    if bytes.is_empty() {
        return Err(ProtectedValueError);
    }
    *value = 0;
    for byte in bytes {
        let digit = byte
            .checked_sub(b'0')
            .filter(|digit| *digit <= 9)
            .ok_or(ProtectedValueError)?;
        *value = value
            .checked_mul(10)
            .and_then(|value| value.checked_add(digit))
            .ok_or(ProtectedValueError)?;
    }
    Ok(())
}

fn decimal_step_u16(value: &mut u16, byte: u8) -> Result<(), ProtectedValueError> {
    let digit = byte
        .checked_sub(b'0')
        .filter(|digit| *digit <= 9)
        .ok_or(ProtectedValueError)?;
    *value = value
        .checked_mul(10)
        .and_then(|value| value.checked_add(u16::from(digit)))
        .ok_or(ProtectedValueError)?;
    Ok(())
}

fn canonical_network(bytes: &[u8], prefix: u8) -> bool {
    let maximum = match bytes.first() {
        Some(4) => 32,
        Some(6) => 128,
        _ => return false,
    };
    if prefix > maximum {
        return false;
    }
    let full = usize::from(prefix / 8);
    let partial = prefix % 8;
    let address = bytes.get(1..).unwrap_or_default();
    if partial != 0 {
        let mask = u8::MAX >> partial;
        if address.get(full).copied().unwrap_or(0) & mask != 0 {
            return false;
        }
    }
    let start = full.saturating_add(usize::from(partial != 0));
    address
        .get(start..)
        .is_some_and(|tail| tail.iter().all(|byte| *byte == 0))
}

const fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        2 if year.is_multiple_of(400) || (year.is_multiple_of(4) && !year.is_multiple_of(100)) => {
            29
        }
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}
