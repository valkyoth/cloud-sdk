//! Public visitor and result types for incremental decoding.

use core::fmt;

/// One validated JSON event.
///
/// Text references are valid only for the duration of the visitor call. A
/// string value can produce multiple [`Self::StringFragment`] events,
/// including one event per decoded character at hostile chunk boundaries.
#[derive(Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum IncrementalJsonEvent<'a> {
    /// Starts an object.
    StartObject,
    /// Ends an object.
    EndObject,
    /// Starts an array.
    StartArray,
    /// Ends an array.
    EndArray,
    /// Supplies a complete, duplicate-checked object key.
    Key(&'a str),
    /// Starts a string value.
    StringStart,
    /// Supplies validated decoded UTF-8 from a string value.
    StringFragment(&'a str),
    /// Ends a string value.
    StringEnd,
    /// Supplies one complete, grammar-checked JSON number token.
    Number(&'a str),
    /// Supplies a Boolean value.
    Bool(bool),
    /// Supplies a null value.
    Null,
}

impl fmt::Debug for IncrementalJsonEvent<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::StartObject => "IncrementalJsonEvent::StartObject",
            Self::EndObject => "IncrementalJsonEvent::EndObject",
            Self::StartArray => "IncrementalJsonEvent::StartArray",
            Self::EndArray => "IncrementalJsonEvent::EndArray",
            Self::Key(_) => "IncrementalJsonEvent::Key([redacted])",
            Self::StringStart => "IncrementalJsonEvent::StringStart",
            Self::StringFragment(_) => "IncrementalJsonEvent::StringFragment([redacted])",
            Self::StringEnd => "IncrementalJsonEvent::StringEnd",
            Self::Number(_) => "IncrementalJsonEvent::Number([redacted])",
            Self::Bool(true) => "IncrementalJsonEvent::Bool(true)",
            Self::Bool(false) => "IncrementalJsonEvent::Bool(false)",
            Self::Null => "IncrementalJsonEvent::Null",
        })
    }
}

/// Control returned by an incremental JSON visitor.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum VisitControl {
    /// Continue validating and emitting events.
    Continue,
    /// Stop immediately without claiming that the remaining document is valid.
    Stop,
}

/// Receives validated incremental JSON events.
pub trait IncrementalJsonVisitor {
    /// Visitor-owned error returned without formatting its potentially
    /// sensitive contents in decoder diagnostics.
    type Error;

    /// Handles one event.
    fn visit(&mut self, event: IncrementalJsonEvent<'_>) -> Result<VisitControl, Self::Error>;
}

/// Current terminal or non-terminal decoder outcome.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IncrementalJsonProgress {
    /// More input or [`super::IncrementalJsonDecoder::finish`] is required.
    Pending,
    /// One complete document was validated, including trailing whitespace.
    Complete,
    /// The visitor stopped before complete-document validation.
    Stopped,
}

/// Incremental JSON decoding failure.
#[derive(Eq, PartialEq)]
#[non_exhaustive]
pub enum IncrementalJsonError<E> {
    /// JSON syntax is invalid.
    InvalidSyntax,
    /// A number token cannot be represented as a finite JSON number.
    InvalidNumber,
    /// A string contains invalid UTF-8.
    InvalidUtf8,
    /// An object contains a duplicate key.
    DuplicateKey,
    /// The configured nesting limit was exceeded.
    DepthLimit,
    /// The configured aggregate input limit was exceeded.
    InputLimit,
    /// The configured aggregate token limit was exceeded.
    TokenLimit,
    /// The configured aggregate object-field limit was exceeded.
    FieldLimit,
    /// The configured per-object field limit was exceeded.
    ObjectFieldLimit,
    /// The configured decoded string limit was exceeded.
    StringLimit,
    /// The configured number-token limit was exceeded.
    NumberLimit,
    /// The configured exponent-digit limit was exceeded.
    ExponentLimit,
    /// End of input was reached before the document was complete.
    IncompleteDocument,
    /// The decoder was used after a terminal result.
    TerminalState,
    /// The visitor returned an error.
    Visitor(E),
}

impl<E> IncrementalJsonError<E> {
    /// Returns the visitor error, if this failure originated in the visitor.
    pub fn into_visitor_error(self) -> Option<E> {
        match self {
            Self::Visitor(error) => Some(error),
            _ => None,
        }
    }
}

impl<E> fmt::Debug for IncrementalJsonError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSyntax => "IncrementalJsonError::InvalidSyntax",
            Self::InvalidNumber => "IncrementalJsonError::InvalidNumber",
            Self::InvalidUtf8 => "IncrementalJsonError::InvalidUtf8",
            Self::DuplicateKey => "IncrementalJsonError::DuplicateKey",
            Self::DepthLimit => "IncrementalJsonError::DepthLimit",
            Self::InputLimit => "IncrementalJsonError::InputLimit",
            Self::TokenLimit => "IncrementalJsonError::TokenLimit",
            Self::FieldLimit => "IncrementalJsonError::FieldLimit",
            Self::ObjectFieldLimit => "IncrementalJsonError::ObjectFieldLimit",
            Self::StringLimit => "IncrementalJsonError::StringLimit",
            Self::NumberLimit => "IncrementalJsonError::NumberLimit",
            Self::ExponentLimit => "IncrementalJsonError::ExponentLimit",
            Self::IncompleteDocument => "IncrementalJsonError::IncompleteDocument",
            Self::TerminalState => "IncrementalJsonError::TerminalState",
            Self::Visitor(_) => "IncrementalJsonError::Visitor([redacted])",
        })
    }
}

impl<E> fmt::Display for IncrementalJsonError<E> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSyntax => "invalid JSON syntax",
            Self::InvalidNumber => "invalid or non-finite JSON number",
            Self::InvalidUtf8 => "invalid UTF-8 in JSON string",
            Self::DuplicateKey => "duplicate JSON object key",
            Self::DepthLimit => "JSON nesting limit exceeded",
            Self::InputLimit => "JSON input limit exceeded",
            Self::TokenLimit => "JSON token limit exceeded",
            Self::FieldLimit => "JSON field limit exceeded",
            Self::ObjectFieldLimit => "JSON object field limit exceeded",
            Self::StringLimit => "JSON string limit exceeded",
            Self::NumberLimit => "JSON number limit exceeded",
            Self::ExponentLimit => "JSON exponent limit exceeded",
            Self::IncompleteDocument => "incomplete JSON document",
            Self::TerminalState => "incremental JSON decoder is terminal",
            Self::Visitor(_) => "incremental JSON visitor failed",
        })
    }
}

impl<E> core::error::Error for IncrementalJsonError<E> {}
