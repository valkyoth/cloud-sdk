//! Bounded query parameter domains.

/// Maximum query key length admitted by the SDK policy.
pub const MAX_QUERY_KEY_BYTES: usize = 64;

/// Maximum query value length admitted by the SDK policy.
pub const MAX_QUERY_VALUE_BYTES: usize = 1024;

/// Query validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum QueryError {
    /// Query keys must not be empty.
    EmptyKey,
    /// Query keys are capped by [`MAX_QUERY_KEY_BYTES`].
    KeyTooLong,
    /// Query values are capped by [`MAX_QUERY_VALUE_BYTES`].
    ValueTooLong,
    /// Query components must not contain control bytes.
    ControlByte,
    /// Query parameters must be inserted in deterministic key order.
    OutOfOrder,
    /// Query builder capacity was exhausted.
    CapacityExceeded,
    /// Fixed output buffer was too small for percent-encoded output.
    EncodeBufferTooSmall,
}

impl_static_error!(QueryError,
    Self::EmptyKey => "query key is empty",
    Self::KeyTooLong => "query key exceeds the length limit",
    Self::ValueTooLong => "query value exceeds the length limit",
    Self::ControlByte => "query component contains a control byte",
    Self::OutOfOrder => "query parameters are out of deterministic order",
    Self::CapacityExceeded => "query parameter capacity was exceeded",
    Self::EncodeBufferTooSmall => "query output buffer is too small",
);

/// Borrowed query parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct QueryParam<'a> {
    key: &'a str,
    value: &'a str,
}

impl<'a> QueryParam<'a> {
    /// Creates a validated borrowed query parameter.
    pub fn new(key: &'a str, value: &'a str) -> Result<Self, QueryError> {
        if key.is_empty() {
            return Err(QueryError::EmptyKey);
        }
        if key.len() > MAX_QUERY_KEY_BYTES {
            return Err(QueryError::KeyTooLong);
        }
        if value.len() > MAX_QUERY_VALUE_BYTES {
            return Err(QueryError::ValueTooLong);
        }
        if has_control_byte(key) || has_control_byte(value) {
            return Err(QueryError::ControlByte);
        }
        Ok(Self { key, value })
    }

    /// Returns the query key.
    #[must_use]
    pub const fn key(self) -> &'a str {
        self.key
    }

    /// Returns the query value.
    #[must_use]
    pub const fn value(self) -> &'a str {
        self.value
    }

    /// Returns true if the value needs percent encoding before transport.
    #[must_use]
    pub fn value_needs_percent_encoding(self) -> bool {
        needs_percent_encoding(self.value)
    }
}

/// Fixed-capacity query builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueryBuilder<'a, const N: usize> {
    params: [Option<QueryParam<'a>>; N],
    len: usize,
}

impl<'a, const N: usize> QueryBuilder<'a, N> {
    /// Creates an empty fixed-capacity query builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            params: [None; N],
            len: 0,
        }
    }

    /// Adds a parameter. Keys must be inserted in nondecreasing order.
    pub fn push(&mut self, param: QueryParam<'a>) -> Result<(), QueryError> {
        if let Some(previous) = self.last()
            && previous.key() > param.key()
        {
            return Err(QueryError::OutOfOrder);
        }
        let slot = match self.params.get_mut(self.len) {
            Some(slot) => slot,
            None => return Err(QueryError::CapacityExceeded),
        };
        *slot = Some(param);
        self.len = match self.len.checked_add(1) {
            Some(len) => len,
            None => return Err(QueryError::CapacityExceeded),
        };
        Ok(())
    }

    /// Returns the number of inserted parameters.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Returns true when no parameters were inserted.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over inserted parameters in deterministic order.
    pub fn iter(&self) -> impl Iterator<Item = QueryParam<'a>> + '_ {
        self.params.iter().filter_map(|param| *param)
    }

    /// Writes a percent-encoded query string into a caller-owned buffer.
    pub fn write_percent_encoded(&self, output: &mut [u8]) -> Result<usize, QueryError> {
        let mut len = 0;
        let mut first = true;
        for param in self.iter() {
            if first {
                first = false;
            } else {
                write_byte(output, &mut len, b'&')?;
            }
            append_percent_encoded_component(param.key(), output, &mut len)?;
            write_byte(output, &mut len, b'=')?;
            append_percent_encoded_component(param.value(), output, &mut len)?;
        }
        Ok(len)
    }

    fn last(&self) -> Option<QueryParam<'a>> {
        self.iter().last()
    }
}

impl<'a, const N: usize> Default for QueryBuilder<'a, N> {
    fn default() -> Self {
        Self::new()
    }
}

/// Returns true when a query component requires percent encoding.
#[must_use]
pub fn needs_percent_encoding(value: &str) -> bool {
    for byte in value.bytes() {
        if !is_unreserved(byte) {
            return true;
        }
    }
    false
}

/// Writes a percent-encoded query component into a caller-owned buffer.
pub fn write_percent_encoded_component(
    value: &str,
    output: &mut [u8],
) -> Result<usize, QueryError> {
    let mut len = 0;
    append_percent_encoded_component(value, output, &mut len)?;
    Ok(len)
}

fn append_percent_encoded_component(
    value: &str,
    output: &mut [u8],
    len: &mut usize,
) -> Result<(), QueryError> {
    for byte in value.bytes() {
        if is_unreserved(byte) {
            write_byte(output, len, byte)?;
        } else {
            write_byte(output, len, b'%')?;
            write_byte(output, len, hex_digit(byte >> 4))?;
            write_byte(output, len, hex_digit(byte & 0x0f))?;
        }
    }
    Ok(())
}

fn write_byte(output: &mut [u8], len: &mut usize, byte: u8) -> Result<(), QueryError> {
    let slot = match output.get_mut(*len) {
        Some(slot) => slot,
        None => return Err(QueryError::EncodeBufferTooSmall),
    };
    *slot = byte;
    *len = match len.checked_add(1) {
        Some(next) => next,
        None => return Err(QueryError::EncodeBufferTooSmall),
    };
    Ok(())
}

fn has_control_byte(value: &str) -> bool {
    for byte in value.bytes() {
        if byte < 0x20 || byte == 0x7f {
            return true;
        }
    }
    false
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
    use super::{
        QueryBuilder, QueryError, QueryParam, needs_percent_encoding,
        write_percent_encoded_component,
    };

    #[test]
    fn validates_query_param_bounds() {
        assert_eq!(QueryParam::new("", "value"), Err(QueryError::EmptyKey));
        assert_eq!(
            QueryParam::new("key", "bad\n"),
            Err(QueryError::ControlByte)
        );
    }

    #[test]
    fn reports_percent_encoding_policy() {
        assert!(!needs_percent_encoding("label_key"));
        assert!(needs_percent_encoding("name=web"));
    }

    #[test]
    fn preserves_deterministic_order() {
        let mut query = QueryBuilder::<2>::new();
        if let Ok(param) = QueryParam::new("a", "1") {
            assert!(query.push(param).is_ok());
        }
        if let Ok(param) = QueryParam::new("a", "2") {
            assert_eq!(query.push(param), Ok(()));
        }
        assert_eq!(query.len(), 2);
    }

    #[test]
    fn writes_percent_encoded_query_without_allocating() {
        let mut query = QueryBuilder::<2>::new();
        if let Ok(param) = QueryParam::new("label_selector", "env=prod,tier in (web,api)") {
            assert!(query.push(param).is_ok());
        }
        if let Ok(param) = QueryParam::new("page", "1") {
            assert!(query.push(param).is_ok());
        }
        let mut output = [0u8; 128];
        assert_eq!(query.write_percent_encoded(&mut output), Ok(62));
        let encoded = output
            .get(..62)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            encoded,
            Some("label_selector=env%3Dprod%2Ctier%20in%20%28web%2Capi%29&page=1")
        );
    }

    #[test]
    fn reports_too_small_percent_encoding_buffer() {
        let mut output = [0u8; 2];
        assert_eq!(
            write_percent_encoded_component(" ", &mut output),
            Err(QueryError::EncodeBufferTooSmall)
        );
    }

    #[test]
    fn rejects_out_of_order_query_keys() {
        let mut query = QueryBuilder::<2>::new();
        if let Ok(param) = QueryParam::new("b", "1") {
            assert!(query.push(param).is_ok());
        }
        if let Ok(param) = QueryParam::new("a", "1") {
            assert_eq!(query.push(param), Err(QueryError::OutOfOrder));
        }
    }
}
