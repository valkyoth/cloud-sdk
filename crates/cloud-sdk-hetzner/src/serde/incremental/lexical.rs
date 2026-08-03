//! Literal, number, and string token processing.

use core::str;

use cloud_sdk_sanitization::sanitize_bytes;

use super::decoder::IncrementalJsonDecoder;
use super::event::{IncrementalJsonError, IncrementalJsonEvent, IncrementalJsonVisitor};
use super::state::{
    Frame, JsonString, Lexical, Literal, LiteralValue, Number, NumberPhase, ObjectPhase,
    StringKind, StringMode,
};

impl IncrementalJsonDecoder {
    pub(super) fn consume_lexical<V: IncrementalJsonVisitor>(
        &mut self,
        byte: u8,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        let lexical = self
            .lexical
            .take()
            .ok_or(IncrementalJsonError::InvalidSyntax)?;
        match lexical {
            Lexical::Literal(literal) => self.consume_literal(literal, byte, visitor),
            Lexical::Number(number) => self.consume_number(number, byte, visitor),
            Lexical::String(string) => self.consume_string(string, byte, visitor),
        }
    }

    pub(super) fn finish_lexical<V: IncrementalJsonVisitor>(
        &mut self,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        let Some(lexical) = self.lexical.take() else {
            return Ok(());
        };
        match lexical {
            Lexical::Number(number) if number.phase.complete() => self.emit_number(number, visitor),
            other => {
                self.lexical = Some(other);
                Err(IncrementalJsonError::IncompleteDocument)
            }
        }
    }

    fn consume_literal<V: IncrementalJsonVisitor>(
        &mut self,
        mut literal: Literal,
        byte: u8,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        if literal.expected.get(literal.position) != Some(&byte) {
            return Err(IncrementalJsonError::InvalidSyntax);
        }
        literal.position = literal.position.saturating_add(1);
        if literal.position == literal.expected.len() {
            let event = match literal.value {
                LiteralValue::Null => IncrementalJsonEvent::Null,
                LiteralValue::Bool(value) => IncrementalJsonEvent::Bool(value),
            };
            self.emit(event, visitor)?;
        } else {
            self.lexical = Some(Lexical::Literal(literal));
        }
        Ok(true)
    }

    fn consume_number<V: IncrementalJsonVisitor>(
        &mut self,
        mut number: Number,
        byte: u8,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        let next = match number.phase {
            NumberPhase::Minus => match byte {
                b'0' => Some(NumberPhase::Zero),
                b'1'..=b'9' => Some(NumberPhase::Integer),
                _ => return Err(IncrementalJsonError::InvalidSyntax),
            },
            NumberPhase::Zero => match byte {
                b'.' => Some(NumberPhase::Dot),
                b'e' | b'E' => Some(NumberPhase::Exponent),
                b'0'..=b'9' => return Err(IncrementalJsonError::InvalidSyntax),
                _ => None,
            },
            NumberPhase::Integer => match byte {
                b'0'..=b'9' => Some(NumberPhase::Integer),
                b'.' => Some(NumberPhase::Dot),
                b'e' | b'E' => Some(NumberPhase::Exponent),
                _ => None,
            },
            NumberPhase::Dot => match byte {
                b'0'..=b'9' => Some(NumberPhase::Fraction),
                _ => return Err(IncrementalJsonError::InvalidSyntax),
            },
            NumberPhase::Fraction => match byte {
                b'0'..=b'9' => Some(NumberPhase::Fraction),
                b'e' | b'E' => Some(NumberPhase::Exponent),
                _ => None,
            },
            NumberPhase::Exponent => match byte {
                b'+' | b'-' => Some(NumberPhase::ExponentSign),
                b'0'..=b'9' => Some(NumberPhase::ExponentDigits),
                _ => return Err(IncrementalJsonError::InvalidSyntax),
            },
            NumberPhase::ExponentSign => match byte {
                b'0'..=b'9' => Some(NumberPhase::ExponentDigits),
                _ => return Err(IncrementalJsonError::InvalidSyntax),
            },
            NumberPhase::ExponentDigits => match byte {
                b'0'..=b'9' => Some(NumberPhase::ExponentDigits),
                _ => None,
            },
        };

        let Some(next) = next else {
            self.emit_number(number, visitor)?;
            return Ok(false);
        };
        self.append_number_byte(&mut number, byte)?;
        if matches!(next, NumberPhase::ExponentDigits) {
            number.exponent_digits = number.exponent_digits.saturating_add(1);
            if number.exponent_digits > self.limits.exponent_digits {
                return Err(IncrementalJsonError::ExponentLimit);
            }
        }
        number.phase = next;
        self.lexical = Some(Lexical::Number(number));
        Ok(true)
    }

    fn append_number_byte<E>(
        &self,
        number: &mut Number,
        byte: u8,
    ) -> Result<(), IncrementalJsonError<E>> {
        if number.text.len() >= self.limits.number_bytes {
            return Err(IncrementalJsonError::NumberLimit);
        }
        let bytes = [byte];
        let text = str::from_utf8(&bytes).map_err(|_| IncrementalJsonError::InvalidSyntax)?;
        number.text.push_str(text);
        Ok(())
    }

    fn emit_number<V: IncrementalJsonVisitor>(
        &mut self,
        number: Number,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        number
            .text
            .try_with_secret(|text| {
                let value = text
                    .parse::<f64>()
                    .map_err(|_| IncrementalJsonError::InvalidNumber)?;
                if !value.is_finite() {
                    return Err(IncrementalJsonError::InvalidNumber);
                }
                self.emit(IncrementalJsonEvent::Number(text), visitor)
            })
            .map_err(|_| IncrementalJsonError::InvalidUtf8)??;
        Ok(())
    }

    fn consume_string<V: IncrementalJsonVisitor>(
        &mut self,
        mut string: JsonString,
        byte: u8,
        visitor: &mut V,
    ) -> Result<bool, IncrementalJsonError<V::Error>> {
        let closes =
            matches!(string.mode, StringMode::Normal) && string.utf8_len == 0 && byte == b'"';
        match string.mode {
            StringMode::Normal => self.consume_normal_string_byte(&mut string, byte, visitor)?,
            StringMode::Escape => self.consume_escape(&mut string, byte, visitor)?,
            StringMode::Unicode { value, digits } => {
                self.consume_unicode(&mut string, value, digits, byte, visitor)?;
            }
            StringMode::LowSlash { high } => {
                if byte != b'\\' {
                    return Err(IncrementalJsonError::InvalidSyntax);
                }
                string.mode = StringMode::LowU { high };
            }
            StringMode::LowU { high } => {
                if byte != b'u' {
                    return Err(IncrementalJsonError::InvalidSyntax);
                }
                string.mode = StringMode::LowUnicode {
                    high,
                    value: 0,
                    digits: 0,
                };
            }
            StringMode::LowUnicode {
                high,
                value,
                digits,
            } => self.consume_low_unicode(&mut string, high, value, digits, byte, visitor)?,
        }
        if !closes {
            self.lexical = Some(Lexical::String(string));
        }
        Ok(true)
    }

    fn consume_normal_string_byte<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        byte: u8,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        if string.utf8_len != 0 {
            if byte & 0xc0 != 0x80 {
                return Err(IncrementalJsonError::InvalidUtf8);
            }
            let slot = string
                .utf8
                .get_mut(string.utf8_len)
                .ok_or(IncrementalJsonError::InvalidUtf8)?;
            *slot = byte;
            string.utf8_len = string.utf8_len.saturating_add(1);
            if string.utf8_len == string.utf8_expected {
                self.deliver_utf8(string, visitor)?;
            }
            return Ok(());
        }
        match byte {
            b'"' => self.finish_string(string, visitor),
            b'\\' => {
                string.mode = StringMode::Escape;
                Ok(())
            }
            0x00..=0x1f => Err(IncrementalJsonError::InvalidSyntax),
            0x20..=0x7f => self.deliver_character(string, char::from(byte), visitor),
            _ => {
                string.utf8_expected = utf8_width(byte)?;
                string.utf8[0] = byte;
                string.utf8_len = 1;
                Ok(())
            }
        }
    }

    fn consume_escape<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        byte: u8,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        let character = match byte {
            b'"' => '"',
            b'\\' => '\\',
            b'/' => '/',
            b'b' => '\u{0008}',
            b'f' => '\u{000c}',
            b'n' => '\n',
            b'r' => '\r',
            b't' => '\t',
            b'u' => {
                string.mode = StringMode::Unicode {
                    value: 0,
                    digits: 0,
                };
                return Ok(());
            }
            _ => return Err(IncrementalJsonError::InvalidSyntax),
        };
        string.mode = StringMode::Normal;
        self.deliver_character(string, character, visitor)
    }

    fn consume_unicode<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        value: u16,
        digits: u8,
        byte: u8,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        let value = append_hex(value, byte)?;
        let digits = digits.saturating_add(1);
        if digits != 4 {
            string.mode = StringMode::Unicode { value, digits };
        } else if (0xd800..=0xdbff).contains(&value) {
            string.mode = StringMode::LowSlash { high: value };
        } else if (0xdc00..=0xdfff).contains(&value) {
            return Err(IncrementalJsonError::InvalidSyntax);
        } else {
            string.mode = StringMode::Normal;
            self.deliver_character(
                string,
                char::from_u32(u32::from(value)).ok_or(IncrementalJsonError::InvalidSyntax)?,
                visitor,
            )?;
        }
        Ok(())
    }

    fn consume_low_unicode<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        high: u16,
        value: u16,
        digits: u8,
        byte: u8,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        let value = append_hex(value, byte)?;
        let digits = digits.saturating_add(1);
        if digits != 4 {
            string.mode = StringMode::LowUnicode {
                high,
                value,
                digits,
            };
            return Ok(());
        }
        if !(0xdc00..=0xdfff).contains(&value) {
            return Err(IncrementalJsonError::InvalidSyntax);
        }
        let scalar = u32::from(high)
            .checked_sub(0xd800)
            .and_then(|high| high.checked_mul(0x400))
            .and_then(|high| {
                u32::from(value)
                    .checked_sub(0xdc00)
                    .and_then(|low| high.checked_add(low))
            })
            .and_then(|scalar| scalar.checked_add(0x1_0000))
            .ok_or(IncrementalJsonError::InvalidSyntax)?;
        string.mode = StringMode::Normal;
        self.deliver_character(
            string,
            char::from_u32(scalar).ok_or(IncrementalJsonError::InvalidSyntax)?,
            visitor,
        )
    }

    fn deliver_utf8<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        let mut scratch = string.utf8;
        let len = string.utf8_len;
        sanitize_bytes(&mut string.utf8);
        string.utf8_len = 0;
        string.utf8_expected = 0;
        let bytes = scratch
            .get(..len)
            .ok_or(IncrementalJsonError::InvalidUtf8)?;
        let result = str::from_utf8(bytes)
            .map_err(|_| IncrementalJsonError::InvalidUtf8)
            .and_then(|text| self.deliver_text(string, text, visitor));
        sanitize_bytes(&mut scratch);
        result
    }

    fn deliver_character<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        character: char,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        let mut scratch = [0_u8; 4];
        let text = character.encode_utf8(&mut scratch);
        let result = self.deliver_text(string, text, visitor);
        sanitize_bytes(&mut scratch);
        result
    }

    fn deliver_text<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        text: &str,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        string.decoded_bytes = string
            .decoded_bytes
            .checked_add(text.len())
            .ok_or(IncrementalJsonError::StringLimit)?;
        if string.decoded_bytes > self.limits.string_bytes {
            return Err(IncrementalJsonError::StringLimit);
        }
        match string.kind {
            StringKind::Key => string
                .key
                .as_mut()
                .ok_or(IncrementalJsonError::InvalidSyntax)?
                .push_str(text),
            StringKind::Value => self.emit(IncrementalJsonEvent::StringFragment(text), visitor)?,
        }
        Ok(())
    }

    fn finish_string<V: IncrementalJsonVisitor>(
        &mut self,
        string: &mut JsonString,
        visitor: &mut V,
    ) -> Result<(), IncrementalJsonError<V::Error>> {
        match string.kind {
            StringKind::Value => self.emit(IncrementalJsonEvent::StringEnd, visitor),
            StringKind::Key => {
                let key = string
                    .key
                    .take()
                    .ok_or(IncrementalJsonError::InvalidSyntax)?;
                let duplicate = match self.frames.last() {
                    Some(Frame::Object(frame)) => frame.keys.contains(&key),
                    _ => return Err(IncrementalJsonError::InvalidSyntax),
                };
                if duplicate {
                    return Err(IncrementalJsonError::DuplicateKey);
                }
                self.charge_token()?;
                self.fields = self
                    .fields
                    .checked_add(1)
                    .ok_or(IncrementalJsonError::FieldLimit)?;
                if self.fields > self.limits.fields {
                    return Err(IncrementalJsonError::FieldLimit);
                }
                let frame = match self.frames.last_mut() {
                    Some(Frame::Object(frame)) => frame,
                    _ => return Err(IncrementalJsonError::InvalidSyntax),
                };
                frame.fields = frame.fields.saturating_add(1);
                if frame.fields > self.limits.object_fields {
                    return Err(IncrementalJsonError::ObjectFieldLimit);
                }
                key.try_with_str(|text| self.emit(IncrementalJsonEvent::Key(text), visitor))
                    .map_err(|_| IncrementalJsonError::InvalidUtf8)??;
                if let Some(Frame::Object(frame)) = self.frames.last_mut() {
                    frame.keys.insert(key);
                    frame.phase = ObjectPhase::Colon;
                }
                Ok(())
            }
        }
    }
}

fn append_hex<E>(value: u16, byte: u8) -> Result<u16, IncrementalJsonError<E>> {
    let digit = match byte {
        b'0'..=b'9' => u16::from(byte.saturating_sub(b'0')),
        b'a'..=b'f' => u16::from(byte.saturating_sub(b'a').saturating_add(10)),
        b'A'..=b'F' => u16::from(byte.saturating_sub(b'A').saturating_add(10)),
        _ => return Err(IncrementalJsonError::InvalidSyntax),
    };
    value
        .checked_mul(16)
        .and_then(|current| current.checked_add(digit))
        .ok_or(IncrementalJsonError::InvalidSyntax)
}

fn utf8_width<E>(byte: u8) -> Result<usize, IncrementalJsonError<E>> {
    match byte {
        0xc2..=0xdf => Ok(2),
        0xe0..=0xef => Ok(3),
        0xf0..=0xf4 => Ok(4),
        _ => Err(IncrementalJsonError::InvalidUtf8),
    }
}
