//! Direct bounded JSON parser for checked response admission.

use alloc::string::String;
use alloc::vec::Vec;
use core::str;

use cloud_sdk_sanitization::{SecretString, sanitize_bytes};

use super::{
    MAX_JSON_CONTAINER_ENTRIES, MAX_JSON_DEPTH, MAX_JSON_NODES, MAX_JSON_STRING_BYTES, Map, Number,
    Value,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum JsonError {
    InvalidSyntax,
    InvalidUtf8,
    InvalidNumber,
    DuplicateKey,
    DepthLimit,
    ContainerLimit,
    NodeLimit,
    StringLimit,
}

pub(super) fn parse(input: &[u8]) -> Result<Value, JsonError> {
    let mut parser = Parser {
        input,
        position: 0,
        nodes: 0,
    };
    parser.skip_whitespace();
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.position != input.len() {
        return Err(JsonError::InvalidSyntax);
    }
    Ok(value)
}

struct Parser<'a> {
    input: &'a [u8],
    position: usize,
    nodes: usize,
}

impl Parser<'_> {
    fn parse_value(&mut self, depth: usize) -> Result<Value, JsonError> {
        self.charge_node()?;
        if depth > MAX_JSON_DEPTH {
            return Err(JsonError::DepthLimit);
        }
        match self.current() {
            Some(b'n') => self.parse_literal(b"null", Value::Null),
            Some(b't') => self.parse_literal(b"true", Value::Bool),
            Some(b'f') => self.parse_literal(b"false", Value::Bool),
            Some(b'"') => self.parse_secret_string().map(Value::String),
            Some(b'[') => self.parse_array(depth).map(Value::Array),
            Some(b'{') => self.parse_object(depth).map(Value::Object),
            Some(b'-' | b'0'..=b'9') => self.parse_number().map(Value::Number),
            _ => Err(JsonError::InvalidSyntax),
        }
    }

    fn parse_literal(&mut self, literal: &[u8], value: Value) -> Result<Value, JsonError> {
        let end = self
            .position
            .checked_add(literal.len())
            .ok_or(JsonError::InvalidSyntax)?;
        if self.input.get(self.position..end) != Some(literal) {
            return Err(JsonError::InvalidSyntax);
        }
        self.position = end;
        Ok(value)
    }

    fn parse_array(&mut self, depth: usize) -> Result<Vec<Value>, JsonError> {
        self.advance()?;
        self.skip_whitespace();
        let mut values = Vec::new();
        if self.consume(b']') {
            return Ok(values);
        }
        loop {
            if values.len() >= MAX_JSON_CONTAINER_ENTRIES {
                return Err(JsonError::ContainerLimit);
            }
            values.push(self.parse_value(depth.saturating_add(1))?);
            self.skip_whitespace();
            if self.consume(b']') {
                return Ok(values);
            }
            if !self.consume(b',') {
                return Err(JsonError::InvalidSyntax);
            }
            self.skip_whitespace();
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<Map, JsonError> {
        self.advance()?;
        self.skip_whitespace();
        let mut values = Map::new();
        if self.consume(b'}') {
            return Ok(values);
        }
        loop {
            if values.len() >= MAX_JSON_CONTAINER_ENTRIES {
                return Err(JsonError::ContainerLimit);
            }
            if self.current() != Some(b'"') {
                return Err(JsonError::InvalidSyntax);
            }
            let key = self.parse_key()?;
            if values.contains_key(&key) {
                return Err(JsonError::DuplicateKey);
            }
            self.skip_whitespace();
            if !self.consume(b':') {
                return Err(JsonError::InvalidSyntax);
            }
            self.skip_whitespace();
            let value = self.parse_value(depth.saturating_add(1))?;
            values.insert(key, value);
            self.skip_whitespace();
            if self.consume(b'}') {
                return Ok(values);
            }
            if !self.consume(b',') {
                return Err(JsonError::InvalidSyntax);
            }
            self.skip_whitespace();
        }
    }

    fn parse_secret_string(&mut self) -> Result<SecretString, JsonError> {
        let scan = scan_string(self.input, self.position)?;
        let mut output = SecretString::with_capacity(scan.decoded_len);
        decode_string(self.input, self.position, scan.end, |fragment| {
            output.push_str(fragment);
        })?;
        self.position = scan.end;
        Ok(output)
    }

    fn parse_key(&mut self) -> Result<String, JsonError> {
        let scan = scan_string(self.input, self.position)?;
        let mut output = String::with_capacity(scan.decoded_len);
        decode_string(self.input, self.position, scan.end, |fragment| {
            output.push_str(fragment);
        })?;
        self.position = scan.end;
        Ok(output)
    }

    fn parse_number(&mut self) -> Result<Number, JsonError> {
        let start = self.position;
        let negative = self.consume(b'-');
        match self.current() {
            Some(b'0') => {
                self.advance()?;
                if matches!(self.current(), Some(b'0'..=b'9')) {
                    return Err(JsonError::InvalidNumber);
                }
            }
            Some(b'1'..=b'9') => self.consume_digits()?,
            _ => return Err(JsonError::InvalidNumber),
        }
        let mut float = false;
        if self.consume(b'.') {
            float = true;
            self.require_digit()?;
            self.consume_digits()?;
        }
        if matches!(self.current(), Some(b'e' | b'E')) {
            float = true;
            self.advance()?;
            if matches!(self.current(), Some(b'+' | b'-')) {
                self.advance()?;
            }
            self.require_digit()?;
            self.consume_digits()?;
        }
        let text = str::from_utf8(
            self.input
                .get(start..self.position)
                .ok_or(JsonError::InvalidNumber)?,
        )
        .map_err(|_| JsonError::InvalidUtf8)?;
        if float {
            return parse_finite_float(text);
        }
        if negative {
            text.parse::<i64>()
                .map(Number::Signed)
                .or_else(|_| parse_finite_float(text))
        } else {
            text.parse::<u64>()
                .map(Number::Unsigned)
                .or_else(|_| parse_finite_float(text))
        }
    }

    fn consume_digits(&mut self) -> Result<(), JsonError> {
        while matches!(self.current(), Some(b'0'..=b'9')) {
            self.advance()?;
        }
        Ok(())
    }

    fn require_digit(&self) -> Result<(), JsonError> {
        matches!(self.current(), Some(b'0'..=b'9'))
            .then_some(())
            .ok_or(JsonError::InvalidNumber)
    }

    fn charge_node(&mut self) -> Result<(), JsonError> {
        self.nodes = self.nodes.checked_add(1).ok_or(JsonError::NodeLimit)?;
        (self.nodes <= MAX_JSON_NODES)
            .then_some(())
            .ok_or(JsonError::NodeLimit)
    }

    fn skip_whitespace(&mut self) {
        while matches!(self.current(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.position = self.position.saturating_add(1);
        }
    }

    fn consume(&mut self, expected: u8) -> bool {
        if self.current() == Some(expected) {
            self.position = self.position.saturating_add(1);
            true
        } else {
            false
        }
    }

    fn advance(&mut self) -> Result<(), JsonError> {
        if self.position >= self.input.len() {
            return Err(JsonError::InvalidSyntax);
        }
        self.position = self.position.saturating_add(1);
        Ok(())
    }

    fn current(&self) -> Option<u8> {
        self.input.get(self.position).copied()
    }
}

fn parse_finite_float(text: &str) -> Result<Number, JsonError> {
    let value = text.parse::<f64>().map_err(|_| JsonError::InvalidNumber)?;
    value
        .is_finite()
        .then_some(Number::Float(value))
        .ok_or(JsonError::InvalidNumber)
}

struct StringScan {
    end: usize,
    decoded_len: usize,
}

fn scan_string(input: &[u8], start: usize) -> Result<StringScan, JsonError> {
    if input.get(start) != Some(&b'"') {
        return Err(JsonError::InvalidSyntax);
    }
    let mut position = start.saturating_add(1);
    let mut decoded_len = 0_usize;
    loop {
        let fragment_start = position;
        while let Some(byte) = input.get(position).copied() {
            if byte == b'"' || byte == b'\\' || byte < 0x20 {
                break;
            }
            position = position.saturating_add(1);
        }
        let fragment = input
            .get(fragment_start..position)
            .ok_or(JsonError::InvalidSyntax)?;
        str::from_utf8(fragment).map_err(|_| JsonError::InvalidUtf8)?;
        decoded_len = checked_string_len(decoded_len, fragment.len())?;
        match input.get(position).copied() {
            Some(b'"') => {
                return Ok(StringScan {
                    end: position.saturating_add(1),
                    decoded_len,
                });
            }
            Some(b'\\') => {
                let (character, end) = escaped_character(input, position)?;
                decoded_len = checked_string_len(decoded_len, character.len_utf8())?;
                position = end;
            }
            _ => return Err(JsonError::InvalidSyntax),
        }
    }
}

fn decode_string(
    input: &[u8],
    start: usize,
    end: usize,
    mut push: impl FnMut(&str),
) -> Result<(), JsonError> {
    let mut position = start.saturating_add(1);
    let content_end = end.checked_sub(1).ok_or(JsonError::InvalidSyntax)?;
    while position < content_end {
        let fragment_start = position;
        while position < content_end {
            let byte = input
                .get(position)
                .copied()
                .ok_or(JsonError::InvalidSyntax)?;
            if byte == b'\\' {
                break;
            }
            position = position.saturating_add(1);
        }
        let fragment = str::from_utf8(
            input
                .get(fragment_start..position)
                .ok_or(JsonError::InvalidSyntax)?,
        )
        .map_err(|_| JsonError::InvalidUtf8)?;
        push(fragment);
        if position < content_end {
            let (character, next) = escaped_character(input, position)?;
            let mut encoded = [0_u8; 4];
            let text = character.encode_utf8(&mut encoded);
            push(text);
            sanitize_bytes(&mut encoded);
            position = next;
        }
    }
    Ok(())
}

fn escaped_character(input: &[u8], slash: usize) -> Result<(char, usize), JsonError> {
    let escape = slash.checked_add(1).ok_or(JsonError::InvalidSyntax)?;
    let next = escape.checked_add(1).ok_or(JsonError::InvalidSyntax)?;
    match input.get(escape).copied() {
        Some(b'"') => Ok(('"', next)),
        Some(b'\\') => Ok(('\\', next)),
        Some(b'/') => Ok(('/', next)),
        Some(b'b') => Ok(('\u{0008}', next)),
        Some(b'f') => Ok(('\u{000c}', next)),
        Some(b'n') => Ok(('\n', next)),
        Some(b'r') => Ok(('\r', next)),
        Some(b't') => Ok(('\t', next)),
        Some(b'u') => escaped_unicode(input, next),
        _ => Err(JsonError::InvalidSyntax),
    }
}

fn escaped_unicode(input: &[u8], digits: usize) -> Result<(char, usize), JsonError> {
    let (first, mut end) = hex_quad(input, digits)?;
    let scalar = if (0xd800..=0xdbff).contains(&first) {
        if input.get(end..end.saturating_add(2)) != Some(br"\u") {
            return Err(JsonError::InvalidSyntax);
        }
        let low_start = end.saturating_add(2);
        let (second, low_end) = hex_quad(input, low_start)?;
        if !(0xdc00..=0xdfff).contains(&second) {
            return Err(JsonError::InvalidSyntax);
        }
        end = low_end;
        let high = u32::from(first)
            .checked_sub(0xd800)
            .ok_or(JsonError::InvalidSyntax)?;
        let low = u32::from(second)
            .checked_sub(0xdc00)
            .ok_or(JsonError::InvalidSyntax)?;
        high.checked_mul(0x400)
            .and_then(|value| value.checked_add(low))
            .and_then(|value| value.checked_add(0x1_0000))
            .ok_or(JsonError::InvalidSyntax)?
    } else if (0xdc00..=0xdfff).contains(&first) {
        return Err(JsonError::InvalidSyntax);
    } else {
        u32::from(first)
    };
    char::from_u32(scalar)
        .map(|character| (character, end))
        .ok_or(JsonError::InvalidSyntax)
}

fn hex_quad(input: &[u8], start: usize) -> Result<(u16, usize), JsonError> {
    let end = start.checked_add(4).ok_or(JsonError::InvalidSyntax)?;
    let digits = input.get(start..end).ok_or(JsonError::InvalidSyntax)?;
    let mut value = 0_u16;
    for digit in digits {
        let nibble = u16::from(hex_value(*digit)?);
        value = value
            .checked_mul(16)
            .and_then(|current| current.checked_add(nibble))
            .ok_or(JsonError::InvalidSyntax)?;
    }
    Ok((value, end))
}

fn hex_value(byte: u8) -> Result<u8, JsonError> {
    match byte {
        b'0'..=b'9' => Ok(byte.saturating_sub(b'0')),
        b'a'..=b'f' => Ok(byte.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => Ok(byte.saturating_sub(b'A').saturating_add(10)),
        _ => Err(JsonError::InvalidSyntax),
    }
}

fn checked_string_len(current: usize, additional: usize) -> Result<usize, JsonError> {
    let len = current
        .checked_add(additional)
        .ok_or(JsonError::StringLimit)?;
    (len <= MAX_JSON_STRING_BYTES)
        .then_some(len)
        .ok_or(JsonError::StringLimit)
}
