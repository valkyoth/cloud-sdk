//! Incremental decoder state machine and structural grammar.

use alloc::collections::BTreeSet;
use alloc::vec::Vec;

use cloud_sdk_sanitization::SecretString;

use super::event::{
    IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonProgress, IncrementalJsonVisitor,
    VisitControl,
};
use super::limits::IncrementalJsonLimits;
use super::state::{
    ArrayPhase, DocumentPhase, Frame, IncrementalKey, JsonString, Lexical, Literal, LiteralValue,
    Number, NumberPhase, ObjectFrame, ObjectPhase, StringKind, StringMode, Terminal,
};

/// Bounded parser for one JSON document supplied in arbitrary chunks.
///
/// The decoder allocates only structural state, duplicate-detection keys, and
/// one bounded number token. It never owns complete string values. Input
/// chunks remain caller-owned and are not erased by this type.
pub struct IncrementalJsonDecoder {
    pub(super) limits: IncrementalJsonLimits,
    pub(super) terminal: Terminal,
    pub(super) document: DocumentPhase,
    pub(super) frames: Vec<Frame>,
    pub(super) lexical: Option<Lexical>,
    pub(super) input_bytes: usize,
    pub(super) tokens: usize,
    pub(super) fields: usize,
}

impl IncrementalJsonDecoder {
    /// Creates a decoder with reviewed default limits.
    #[must_use]
    pub fn new() -> Self {
        Self::with_limits(IncrementalJsonLimits::DEFAULT)
    }

    /// Creates a decoder with caller-lowered reviewed limits.
    #[must_use]
    pub fn with_limits(limits: IncrementalJsonLimits) -> Self {
        Self {
            limits,
            terminal: Terminal::Active,
            document: DocumentPhase::Value,
            frames: Vec::new(),
            lexical: None,
            input_bytes: 0,
            tokens: 0,
            fields: 0,
        }
    }

    /// Validates one chunk and emits every complete event it contains.
    ///
    /// A successful push remains [`IncrementalJsonProgress::Pending`] until
    /// [`Self::finish`] confirms end of input. After any error, completion, or
    /// visitor stop, the decoder is terminal.
    pub fn push<V: IncrementalJsonVisitor>(
        &mut self,
        input: &[u8],
        visitor: &mut V,
    ) -> Result<IncrementalJsonProgress, IncrementalJsonError<V::Error>> {
        match self.terminal {
            Terminal::Stopped => return Ok(IncrementalJsonProgress::Stopped),
            Terminal::Active => {}
            Terminal::Complete | Terminal::Failed => {
                return Err(IncrementalJsonError::TerminalState);
            }
        }
        self.input_bytes = match self.input_bytes.checked_add(input.len()) {
            Some(total) if total <= self.limits.input_bytes => total,
            _ => return self.fail(IncrementalJsonError::InputLimit),
        };

        let mut position = 0;
        while position < input.len() {
            let byte = input
                .get(position)
                .copied()
                .ok_or(IncrementalJsonError::InvalidSyntax)?;
            let consumed = match self.consume_byte(byte, visitor) {
                Ok(consumed) => consumed,
                Err(error) => return self.fail(error),
            };
            if matches!(self.terminal, Terminal::Stopped) {
                return Ok(IncrementalJsonProgress::Stopped);
            }
            if consumed {
                position = position.saturating_add(1);
            }
        }
        Ok(IncrementalJsonProgress::Pending)
    }

    /// Declares end of input and requires one complete JSON document.
    pub fn finish<V: IncrementalJsonVisitor>(
        &mut self,
        visitor: &mut V,
    ) -> Result<IncrementalJsonProgress, IncrementalJsonError<V::Error>> {
        match self.terminal {
            Terminal::Stopped => return Ok(IncrementalJsonProgress::Stopped),
            Terminal::Active => {}
            Terminal::Complete | Terminal::Failed => {
                return Err(IncrementalJsonError::TerminalState);
            }
        }
        if let Err(error) = self.finish_lexical(visitor) {
            return self.fail(error);
        }
        if matches!(self.terminal, Terminal::Stopped) {
            return Ok(IncrementalJsonProgress::Stopped);
        }
        if self.lexical.is_some()
            || !self.frames.is_empty()
            || !matches!(self.document, DocumentPhase::Trailing)
        {
            return self.fail(IncrementalJsonError::IncompleteDocument);
        }
        self.terminal = Terminal::Complete;
        Ok(IncrementalJsonProgress::Complete)
    }

    pub(super) fn consume_byte<V: IncrementalJsonVisitor>(
        &mut self,
        byte: u8,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        if self.lexical.is_some() {
            return self.consume_lexical(byte, visitor);
        }
        if is_whitespace(byte) {
            return Ok(true);
        }
        if matches!(self.document, DocumentPhase::Trailing) && self.frames.is_empty() {
            return Err(IncrementalJsonError::InvalidSyntax);
        }

        match self.frames.last() {
            Some(Frame::Array(ArrayPhase::ValueOrEnd | ArrayPhase::CommaOrEnd)) if byte == b']' => {
                self.close_array(visitor)
            }
            Some(Frame::Object(frame))
                if matches!(frame.phase, ObjectPhase::KeyOrEnd | ObjectPhase::CommaOrEnd)
                    && byte == b'}' =>
            {
                self.close_object(visitor)
            }
            Some(Frame::Array(ArrayPhase::CommaOrEnd)) if byte == b',' => {
                if let Some(Frame::Array(phase)) = self.frames.last_mut() {
                    *phase = ArrayPhase::Value;
                }
                Ok(true)
            }
            Some(Frame::Object(frame))
                if matches!(frame.phase, ObjectPhase::CommaOrEnd) && byte == b',' =>
            {
                if let Some(Frame::Object(frame)) = self.frames.last_mut() {
                    frame.phase = ObjectPhase::Key;
                }
                Ok(true)
            }
            Some(Frame::Object(frame)) if matches!(frame.phase, ObjectPhase::Colon) => {
                if byte != b':' {
                    return Err(IncrementalJsonError::InvalidSyntax);
                }
                if let Some(Frame::Object(frame)) = self.frames.last_mut() {
                    frame.phase = ObjectPhase::Value;
                }
                Ok(true)
            }
            Some(Frame::Object(frame))
                if matches!(frame.phase, ObjectPhase::KeyOrEnd | ObjectPhase::Key) =>
            {
                if byte != b'"' {
                    return Err(IncrementalJsonError::InvalidSyntax);
                }
                self.start_key();
                Ok(true)
            }
            _ if self.expects_value() => self.start_value(byte, visitor),
            _ => Err(IncrementalJsonError::InvalidSyntax),
        }
    }

    fn expects_value(&self) -> bool {
        match self.frames.last() {
            Some(Frame::Array(phase)) => {
                matches!(phase, ArrayPhase::ValueOrEnd | ArrayPhase::Value)
            }
            Some(Frame::Object(frame)) => matches!(frame.phase, ObjectPhase::Value),
            None => matches!(self.document, DocumentPhase::Value),
        }
    }

    fn mark_value_started<E>(&mut self) -> Result<(), IncrementalJsonError<E>> {
        self.charge_token()?;
        match self.frames.last_mut() {
            Some(Frame::Array(phase)) => *phase = ArrayPhase::CommaOrEnd,
            Some(Frame::Object(frame)) => frame.phase = ObjectPhase::CommaOrEnd,
            None => self.document = DocumentPhase::Trailing,
        }
        Ok(())
    }

    fn start_value<V: IncrementalJsonVisitor>(
        &mut self,
        byte: u8,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        self.mark_value_started()?;
        match byte {
            b'{' => {
                self.open_container(Frame::Object(ObjectFrame {
                    phase: ObjectPhase::KeyOrEnd,
                    fields: 0,
                    keys: BTreeSet::new(),
                }))?;
                self.emit(IncrementalJsonEvent::StartObject, visitor)?;
            }
            b'[' => {
                self.open_container(Frame::Array(ArrayPhase::ValueOrEnd))?;
                self.emit(IncrementalJsonEvent::StartArray, visitor)?;
            }
            b'"' => {
                self.lexical = Some(Lexical::String(JsonString {
                    kind: StringKind::Value,
                    mode: StringMode::Normal,
                    decoded_bytes: 0,
                    utf8: [0; 4],
                    utf8_len: 0,
                    utf8_expected: 0,
                    key: None,
                }));
                self.emit(IncrementalJsonEvent::StringStart, visitor)?;
            }
            b'n' => self.start_literal(b"null", LiteralValue::Null),
            b't' => self.start_literal(b"true", LiteralValue::Bool(true)),
            b'f' => self.start_literal(b"false", LiteralValue::Bool(false)),
            b'-' | b'0'..=b'9' => self.start_number(byte),
            _ => return Err(IncrementalJsonError::InvalidSyntax),
        }
        Ok(true)
    }

    fn start_literal(&mut self, expected: &'static [u8], value: LiteralValue) {
        self.lexical = Some(Lexical::Literal(Literal {
            expected,
            position: 1,
            value,
        }));
    }

    fn start_number(&mut self, byte: u8) {
        let mut text = SecretString::with_capacity(self.limits.number_bytes.min(32));
        text.push_str(match byte {
            b'-' => "-",
            b'0' => "0",
            b'1' => "1",
            b'2' => "2",
            b'3' => "3",
            b'4' => "4",
            b'5' => "5",
            b'6' => "6",
            b'7' => "7",
            b'8' => "8",
            b'9' => "9",
            _ => "",
        });
        let phase = match byte {
            b'-' => NumberPhase::Minus,
            b'0' => NumberPhase::Zero,
            _ => NumberPhase::Integer,
        };
        self.lexical = Some(Lexical::Number(Number {
            text,
            phase,
            exponent_digits: 0,
        }));
    }

    fn start_key(&mut self) {
        self.lexical = Some(Lexical::String(JsonString {
            kind: StringKind::Key,
            mode: StringMode::Normal,
            decoded_bytes: 0,
            utf8: [0; 4],
            utf8_len: 0,
            utf8_expected: 0,
            key: Some(IncrementalKey::with_capacity(32)),
        }));
    }

    fn open_container<E>(&mut self, frame: Frame) -> Result<(), IncrementalJsonError<E>> {
        if self.frames.len() >= self.limits.depth {
            return Err(IncrementalJsonError::DepthLimit);
        }
        self.frames.push(frame);
        Ok(())
    }

    fn close_array<V: IncrementalJsonVisitor>(
        &mut self,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        self.frames.pop();
        self.emit(IncrementalJsonEvent::EndArray, visitor)?;
        Ok(true)
    }

    fn close_object<V: IncrementalJsonVisitor>(
        &mut self,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        self.frames.pop();
        self.emit(IncrementalJsonEvent::EndObject, visitor)?;
        Ok(true)
    }

    pub(super) fn emit<V: IncrementalJsonVisitor>(
        &mut self,
        event: IncrementalJsonEvent<'_>,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        match visitor
            .visit(event)
            .map_err(IncrementalJsonError::Visitor)?
        {
            VisitControl::Continue => Ok(()),
            VisitControl::Stop => {
                self.terminal = Terminal::Stopped;
                Ok(())
            }
        }
    }

    pub(super) fn charge_token<E>(&mut self) -> Result<(), IncrementalJsonError<E>> {
        self.tokens = self
            .tokens
            .checked_add(1)
            .ok_or(IncrementalJsonError::TokenLimit)?;
        if self.tokens > self.limits.tokens {
            return Err(IncrementalJsonError::TokenLimit);
        }
        Ok(())
    }

    fn fail<T, E>(&mut self, error: IncrementalJsonError<E>) -> Result<T, IncrementalJsonError<E>> {
        self.terminal = Terminal::Failed;
        self.lexical = None;
        self.frames.clear();
        Err(error)
    }
}

impl Default for IncrementalJsonDecoder {
    fn default() -> Self {
        Self::new()
    }
}

impl core::fmt::Debug for IncrementalJsonDecoder {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("IncrementalJsonDecoder { state: [redacted] }")
    }
}

const fn is_whitespace(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r')
}
