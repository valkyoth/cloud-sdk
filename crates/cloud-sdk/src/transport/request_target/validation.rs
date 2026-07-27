use super::{MAX_REQUEST_TARGET_BYTES, RequestPathError, StructuredQueryError};

pub(super) fn validate_path(value: &str) -> Result<(), RequestPathError> {
    if value.is_empty() {
        return Err(RequestPathError::Empty);
    }
    if value.len() > MAX_REQUEST_TARGET_BYTES {
        return Err(RequestPathError::TooLong);
    }
    if !value.starts_with('/') || value.starts_with("//") {
        return Err(RequestPathError::NotOriginForm);
    }
    if value.contains("//") {
        return Err(RequestPathError::DoubledSlash);
    }
    if value
        .split('/')
        .any(|segment| matches!(segment, "." | ".."))
    {
        return Err(RequestPathError::DotSegment);
    }
    validate_component_bytes(value.as_bytes(), Component::Path, false)
}

pub(super) fn validate_query(value: &str, allow_plus: bool) -> Result<(), StructuredQueryError> {
    if value.len() > MAX_REQUEST_TARGET_BYTES {
        return Err(StructuredQueryError::TooLong);
    }
    if value.is_empty() {
        return Ok(());
    }
    if !allow_plus && value.contains('+') {
        return Err(StructuredQueryError::PlusForbidden);
    }
    for pair in value.split('&') {
        if pair.is_empty() {
            return Err(StructuredQueryError::EmptyPair);
        }
        let (key, value) = match pair.split_once('=') {
            Some(parts) => parts,
            None => (pair, ""),
        };
        if key.is_empty() {
            return Err(StructuredQueryError::EmptyKey);
        }
        if value.contains('=') {
            return Err(StructuredQueryError::MultipleEquals);
        }
    }
    validate_component_bytes(value.as_bytes(), Component::Query, allow_plus)
        .map_err(query_component_error)
}

#[derive(Clone, Copy)]
enum Component {
    Path,
    Query,
}

fn validate_component_bytes(
    bytes: &[u8],
    component: Component,
    allow_plus: bool,
) -> Result<(), RequestPathError> {
    let mut index = 0;
    while index < bytes.len() {
        let byte = *bytes.get(index).ok_or(RequestPathError::InvalidByte)?;
        if byte == b'%' {
            let high_index = index
                .checked_add(1)
                .ok_or(RequestPathError::InvalidPercentTriplet)?;
            let low_index = index
                .checked_add(2)
                .ok_or(RequestPathError::InvalidPercentTriplet)?;
            let high = *bytes
                .get(high_index)
                .ok_or(RequestPathError::InvalidPercentTriplet)?;
            let low = *bytes
                .get(low_index)
                .ok_or(RequestPathError::InvalidPercentTriplet)?;
            if decode_hex_case_insensitive(high).is_none()
                || decode_hex_case_insensitive(low).is_none()
            {
                return Err(RequestPathError::InvalidPercentTriplet);
            }
            if high.is_ascii_lowercase() || low.is_ascii_lowercase() {
                return Err(RequestPathError::LowercasePercentHex);
            }
            let decoded = decode_hex(high)
                .and_then(|high| high.checked_mul(16))
                .and_then(|high| decode_hex(low).and_then(|low| high.checked_add(low)))
                .ok_or(RequestPathError::InvalidPercentTriplet)?;
            validate_encoded(decoded, component)?;
            index = index
                .checked_add(3)
                .ok_or(RequestPathError::InvalidPercentTriplet)?;
            continue;
        }
        let valid = match component {
            Component::Path => is_path_byte(byte),
            Component::Query => {
                is_unreserved(byte) || matches!(byte, b'&' | b'=') || allow_plus && byte == b'+'
            }
        };
        if !valid {
            return Err(RequestPathError::InvalidByte);
        }
        index = index.checked_add(1).ok_or(RequestPathError::InvalidByte)?;
    }
    Ok(())
}

fn validate_encoded(byte: u8, component: Component) -> Result<(), RequestPathError> {
    if matches!(component, Component::Path) && matches!(byte, b'/' | b'\\' | b'?' | b'%') {
        return Err(RequestPathError::EncodedSeparator);
    }
    if byte < 0x20 || byte == 0x7f || matches!(byte, b'#' | b'\\') {
        return Err(RequestPathError::EncodedControl);
    }
    if is_unreserved(byte) {
        return Err(RequestPathError::EncodedUnreserved);
    }
    Ok(())
}

fn query_component_error(error: RequestPathError) -> StructuredQueryError {
    match error {
        RequestPathError::TooLong => StructuredQueryError::TooLong,
        RequestPathError::InvalidPercentTriplet => StructuredQueryError::InvalidPercentTriplet,
        RequestPathError::LowercasePercentHex => StructuredQueryError::LowercasePercentHex,
        RequestPathError::EncodedControl | RequestPathError::EncodedSeparator => {
            StructuredQueryError::EncodedControl
        }
        RequestPathError::EncodedUnreserved => StructuredQueryError::EncodedUnreserved,
        _ => StructuredQueryError::InvalidByte,
    }
}

const fn is_path_byte(byte: u8) -> bool {
    is_unreserved(byte)
        || matches!(
            byte,
            b'/' | b'!'
                | b'$'
                | b'&'
                | b'\''
                | b'('
                | b')'
                | b'*'
                | b'+'
                | b','
                | b';'
                | b'='
                | b':'
                | b'@'
        )
}

const fn is_unreserved(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~')
}

const fn decode_hex_case_insensitive(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'A'..=b'F' => match byte.checked_sub(b'A') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        b'a'..=b'f' => match byte.checked_sub(b'a') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        _ => None,
    }
}

const fn decode_hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => byte.checked_sub(b'0'),
        b'A'..=b'F' => match byte.checked_sub(b'A') {
            Some(value) => value.checked_add(10),
            None => None,
        },
        _ => None,
    }
}
