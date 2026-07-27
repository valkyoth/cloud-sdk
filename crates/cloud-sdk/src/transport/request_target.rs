//! Canonical, provider-neutral HTTP request-target components.

use core::fmt;

mod validation;

use validation::{validate_path, validate_query};

/// Maximum origin-form request-target length admitted by the core contract.
pub const MAX_REQUEST_TARGET_BYTES: usize = 8192;

/// Canonical path validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestPathError {
    /// Paths must not be empty.
    Empty,
    /// Paths must start with exactly one `/`.
    NotOriginForm,
    /// Paths exceed [`MAX_REQUEST_TARGET_BYTES`].
    TooLong,
    /// A raw byte is outside the admitted ASCII path grammar.
    InvalidByte,
    /// Adjacent `/` separators are forbidden.
    DoubledSlash,
    /// `.` and `..` segments are forbidden.
    DotSegment,
    /// A percent triplet is incomplete or malformed.
    InvalidPercentTriplet,
    /// Percent hexadecimal digits must be uppercase.
    LowercasePercentHex,
    /// A percent triplet encodes a structural separator.
    EncodedSeparator,
    /// A percent triplet encodes a control byte.
    EncodedControl,
    /// An unreserved byte was needlessly percent encoded.
    EncodedUnreserved,
}

impl_static_error!(RequestPathError,
    Self::Empty => "request path is empty",
    Self::NotOriginForm => "request path is not in origin form",
    Self::TooLong => "request path exceeds the length limit",
    Self::InvalidByte => "request path contains a forbidden byte",
    Self::DoubledSlash => "request path contains an empty segment",
    Self::DotSegment => "request path contains a dot segment",
    Self::InvalidPercentTriplet => "request path contains malformed percent encoding",
    Self::LowercasePercentHex => "request path percent encoding is not uppercase",
    Self::EncodedSeparator => "request path percent encoding hides a separator",
    Self::EncodedControl => "request path percent encoding hides a control byte",
    Self::EncodedUnreserved => "request path percent encodes an unreserved byte",
);

/// Structured query validation error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StructuredQueryError {
    /// Queries exceed [`MAX_REQUEST_TARGET_BYTES`].
    TooLong,
    /// A raw byte is outside the admitted ASCII query grammar.
    InvalidByte,
    /// Empty `&`-delimited pairs are forbidden.
    EmptyPair,
    /// Query keys must not be empty.
    EmptyKey,
    /// A query pair contains more than one raw `=`.
    MultipleEquals,
    /// A percent triplet is incomplete or malformed.
    InvalidPercentTriplet,
    /// Percent hexadecimal digits must be uppercase.
    LowercasePercentHex,
    /// A percent triplet encodes a control byte or fragment delimiter.
    EncodedControl,
    /// An unreserved byte was needlessly percent encoded.
    EncodedUnreserved,
    /// `+` is forbidden outside an explicit form-query dialect.
    PlusForbidden,
}

impl_static_error!(StructuredQueryError,
    Self::TooLong => "query exceeds the length limit",
    Self::InvalidByte => "query contains a forbidden byte",
    Self::EmptyPair => "query contains an empty pair",
    Self::EmptyKey => "query key is empty",
    Self::MultipleEquals => "query pair contains multiple separators",
    Self::InvalidPercentTriplet => "query contains malformed percent encoding",
    Self::LowercasePercentHex => "query percent encoding is not uppercase",
    Self::EncodedControl => "query percent encoding hides a control or fragment byte",
    Self::EncodedUnreserved => "query percent encodes an unreserved byte",
    Self::PlusForbidden => "query uses form-style plus encoding",
);

/// Request-target validation or assembly error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestTargetError {
    /// The path component was rejected.
    Path(RequestPathError),
    /// The canonical query component was rejected.
    Query(StructuredQueryError),
    /// The complete target exceeds [`MAX_REQUEST_TARGET_BYTES`].
    TooLong,
    /// Caller-owned output cannot hold the complete target.
    OutputTooSmall,
}

impl fmt::Display for RequestTargetError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Path(error) => write!(formatter, "invalid request path: {error}"),
            Self::Query(error) => write!(formatter, "invalid request query: {error}"),
            Self::TooLong => formatter.write_str("request target exceeds the length limit"),
            Self::OutputTooSmall => formatter.write_str("request target output is too small"),
        }
    }
}

impl core::error::Error for RequestTargetError {}

/// Validated canonical origin-form path.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RequestPath<'a>(&'a str);

impl<'a> RequestPath<'a> {
    /// Validates one origin-form path without a query or fragment.
    pub fn new(value: &'a str) -> Result<Self, RequestPathError> {
        validate_path(value)?;
        Ok(Self(value))
    }

    /// Returns the exact validated path bytes as text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

impl fmt::Debug for RequestPath<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RequestPath([redacted])")
    }
}

/// Validated RFC 3986-style query with `%20` space encoding.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct CanonicalQuery<'a>(&'a str);

impl<'a> CanonicalQuery<'a> {
    /// Validates a query without the leading `?`.
    pub fn new(value: &'a str) -> Result<Self, StructuredQueryError> {
        validate_query(value, false)?;
        Ok(Self(value))
    }

    /// Returns the exact validated query bytes as text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }

    /// Iterates over pairs without decoding or reordering them.
    #[must_use]
    pub const fn pairs(self) -> QueryPairs<'a> {
        QueryPairs::new(self.0)
    }
}

impl fmt::Debug for CanonicalQuery<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("CanonicalQuery([redacted])")
    }
}

/// Explicit provider dialect that admits `+` for encoded spaces.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct FormQuery<'a>(&'a str);

impl<'a> FormQuery<'a> {
    /// Validates a form-style query without the leading `?`.
    pub fn new(value: &'a str) -> Result<Self, StructuredQueryError> {
        validate_query(value, true)?;
        Ok(Self(value))
    }

    /// Returns the exact validated query bytes as text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }

    /// Iterates over pairs without decoding or reordering them.
    #[must_use]
    pub const fn pairs(self) -> QueryPairs<'a> {
        QueryPairs::new(self.0)
    }
}

impl fmt::Debug for FormQuery<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("FormQuery([redacted])")
    }
}

/// Explicit request query state.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RequestQuery<'a> {
    /// No `?` delimiter is present.
    Absent,
    /// A canonical query is present; its value may be empty.
    Canonical(CanonicalQuery<'a>),
    /// A form-style query is present; its value may be empty.
    Form(FormQuery<'a>),
}

impl<'a> RequestQuery<'a> {
    /// Reports whether a query delimiter is present.
    #[must_use]
    pub const fn is_present(self) -> bool {
        !matches!(self, Self::Absent)
    }

    /// Returns the exact query bytes, excluding `?`.
    #[must_use]
    pub const fn as_str(self) -> Option<&'a str> {
        match self {
            Self::Absent => None,
            Self::Canonical(query) => Some(query.as_str()),
            Self::Form(query) => Some(query.as_str()),
        }
    }
}

/// One encoded query pair, preserving missing versus empty values.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct QueryPair<'a> {
    key: &'a str,
    value: Option<&'a str>,
}

impl<'a> QueryPair<'a> {
    /// Returns the exact encoded key.
    #[must_use]
    pub const fn key(self) -> &'a str {
        self.key
    }

    /// Returns the exact encoded value, distinguishing `key` from `key=`.
    #[must_use]
    pub const fn value(self) -> Option<&'a str> {
        self.value
    }
}

impl fmt::Debug for QueryPair<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("QueryPair([redacted])")
    }
}

/// Exact-order iterator over encoded query pairs.
#[derive(Clone)]
pub struct QueryPairs<'a> {
    remaining: &'a str,
}

impl<'a> QueryPairs<'a> {
    const fn new(value: &'a str) -> Self {
        Self { remaining: value }
    }
}

impl fmt::Debug for QueryPairs<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("QueryPairs([redacted])")
    }
}

impl<'a> Iterator for QueryPairs<'a> {
    type Item = QueryPair<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining.is_empty() {
            return None;
        }
        let (pair, remaining) = match self.remaining.split_once('&') {
            Some(parts) => parts,
            None => (self.remaining, ""),
        };
        self.remaining = remaining;
        let (key, value) = match pair.split_once('=') {
            Some((key, value)) => (key, Some(value)),
            None => (pair, None),
        };
        Some(QueryPair { key, value })
    }
}

/// Validated complete origin-form HTTP request target.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct RequestTarget<'a> {
    value: &'a str,
    path: RequestPath<'a>,
    query: RequestQuery<'a>,
}

impl<'a> RequestTarget<'a> {
    /// Validates a canonical `/path?query` target.
    ///
    /// Use [`Self::assemble`] when query absence and presence must be selected
    /// explicitly or when a provider uses [`FormQuery`].
    pub fn new(value: &'a str) -> Result<Self, RequestTargetError> {
        if value.len() > MAX_REQUEST_TARGET_BYTES {
            return Err(RequestTargetError::TooLong);
        }
        let (path, query) = match value.split_once('?') {
            Some((path, query)) => (
                RequestPath::new(path).map_err(RequestTargetError::Path)?,
                RequestQuery::Canonical(
                    CanonicalQuery::new(query).map_err(RequestTargetError::Query)?,
                ),
            ),
            None => (
                RequestPath::new(value).map_err(RequestTargetError::Path)?,
                RequestQuery::Absent,
            ),
        };
        Ok(Self { value, path, query })
    }

    /// Atomically writes validated components into caller-owned storage.
    ///
    /// Only `output[..target.len()]`, equivalently [`Self::as_str`], is
    /// initialized by this operation. The remaining tail is left untouched
    /// and may contain data from an earlier use. Never read, log, hash, sign,
    /// or transmit the backing buffer past `target.len()`. Apply the caller's
    /// cleanup policy to the complete buffer before reusing it across a
    /// sensitive boundary.
    pub fn assemble<'output>(
        path: RequestPath<'_>,
        query: RequestQuery<'_>,
        output: &'output mut [u8],
    ) -> Result<RequestTarget<'output>, RequestTargetError> {
        let query_len = query.as_borrowed_str().map_or(0, str::len);
        let delimiter_len = usize::from(query.is_present());
        let len = path
            .as_str()
            .len()
            .checked_add(delimiter_len)
            .and_then(|len| len.checked_add(query_len))
            .ok_or(RequestTargetError::TooLong)?;
        if len > MAX_REQUEST_TARGET_BYTES {
            return Err(RequestTargetError::TooLong);
        }
        let target = output
            .get_mut(..len)
            .ok_or(RequestTargetError::OutputTooSmall)?;
        let path_len = path.as_str().len();
        let (path_output, suffix) = target.split_at_mut(path_len);
        path_output.copy_from_slice(path.as_str().as_bytes());
        if query.is_present() {
            let (delimiter, query_output) = suffix.split_at_mut(1);
            let delimiter = delimiter
                .first_mut()
                .ok_or(RequestTargetError::OutputTooSmall)?;
            *delimiter = b'?';
            if let Some(query) = query.as_borrowed_str() {
                query_output.copy_from_slice(query.as_bytes());
            }
        }
        let value = core::str::from_utf8(target).map_err(|_| RequestTargetError::OutputTooSmall)?;
        let path_value = value
            .get(..path_len)
            .ok_or(RequestTargetError::OutputTooSmall)?;
        let path = RequestPath(path_value);
        let query = match query {
            RequestQuery::Absent => RequestQuery::Absent,
            RequestQuery::Canonical(_) => {
                let query_start = path_len.checked_add(1).ok_or(RequestTargetError::TooLong)?;
                let query_value = value
                    .get(query_start..)
                    .ok_or(RequestTargetError::OutputTooSmall)?;
                RequestQuery::Canonical(CanonicalQuery(query_value))
            }
            RequestQuery::Form(_) => {
                let query_start = path_len.checked_add(1).ok_or(RequestTargetError::TooLong)?;
                let query_value = value
                    .get(query_start..)
                    .ok_or(RequestTargetError::OutputTooSmall)?;
                RequestQuery::Form(FormQuery(query_value))
            }
        };
        Ok(RequestTarget { value, path, query })
    }

    /// Returns the exact validated request target.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }

    /// Returns the validated path component.
    #[must_use]
    pub const fn path(self) -> RequestPath<'a> {
        self.path
    }

    /// Returns the explicit query state and dialect.
    #[must_use]
    pub const fn query(self) -> RequestQuery<'a> {
        self.query
    }

    /// Returns the exact final query bytes for signing or fingerprinting.
    #[must_use]
    pub fn query_bytes(self) -> Option<&'a [u8]> {
        self.query().as_borrowed_str().map(str::as_bytes)
    }

    /// Returns the complete target length.
    #[must_use]
    pub const fn len(self) -> usize {
        self.value.len()
    }

    /// Reports whether the complete target is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        false
    }
}

impl fmt::Debug for RequestTarget<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RequestTarget([redacted])")
    }
}

impl<'a> RequestQuery<'a> {
    const fn as_borrowed_str(self) -> Option<&'a str> {
        match self {
            Self::Absent => None,
            Self::Canonical(query) => Some(query.as_str()),
            Self::Form(query) => Some(query.as_str()),
        }
    }
}

#[cfg(test)]
mod tests;
