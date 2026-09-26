//! Bounded, duplicate-rejecting YAML-to-JSON bridge for offline source evidence.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::io::{Read, Write as IoWrite};

use saphyr::Scalar;
use saphyr_parser::{Event, Parser};

const MAX_BYTES: u64 = 10 * 1024 * 1024;
const MAX_EVENTS: usize = 500_000;
const MAX_DEPTH: usize = 64;

fn quote(value: &str, output: &mut String) -> Result<(), String> {
    output.push('"');
    for character in value.chars() {
        match character {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\u{0}'..='\u{1f}' => write!(output, "\\u{:04x}", u32::from(character))
                .map_err(|_| "JSON encoding failed")?,
            _ => output.push(character),
        }
    }
    output.push('"');
    Ok(())
}

fn node<'a>(
    events: &mut impl Iterator<Item = Event<'a>>,
    event: Event<'a>,
    depth: usize,
    output: &mut String,
) -> Result<(), String> {
    if depth > MAX_DEPTH {
        return Err("YAML depth exceeded".into());
    }
    match event {
        Event::Scalar(value, style, 0, None) => {
            let scalar =
                Scalar::parse_from_cow_and_metadata(value, style, None).ok_or("invalid scalar")?;
            match scalar {
                Scalar::String(text) => quote(&text, output)?,
                Scalar::Null => output.push_str("null"),
                Scalar::Boolean(value) => output.push_str(if value { "true" } else { "false" }),
                Scalar::Integer(value) => {
                    write!(output, "{value}").map_err(|_| "integer encoding failed")?;
                }
                Scalar::FloatingPoint(value) if value.is_finite() => {
                    write!(output, "{value}").map_err(|_| "float encoding failed")?;
                }
                _ => return Err("non-finite scalar".into()),
            }
        }
        Event::MappingStart(0, None) => {
            output.push('{');
            let mut keys = BTreeSet::new();
            loop {
                let event = events.next().ok_or("incomplete mapping")?;
                if matches!(event, Event::MappingEnd) {
                    break;
                }
                let Event::Scalar(key, style, 0, None) = event else {
                    return Err("mapping keys must be untagged strings".into());
                };
                let Some(Scalar::String(key)) =
                    Scalar::parse_from_cow_and_metadata(key, style, None)
                else {
                    return Err("mapping key is not a string".into());
                };
                if key == "<<" || !keys.insert(key.clone()) {
                    return Err("duplicate or merge mapping key".into());
                }
                if keys.len() > 1 {
                    output.push(',');
                }
                quote(&key, output)?;
                output.push(':');
                let value = events.next().ok_or("missing mapping value")?;
                node(events, value, depth + 1, output)?;
            }
            output.push('}');
        }
        Event::SequenceStart(0, None) => {
            output.push('[');
            let mut first = true;
            loop {
                let value = events.next().ok_or("incomplete sequence")?;
                if matches!(value, Event::SequenceEnd) {
                    break;
                }
                if !first {
                    output.push(',');
                }
                first = false;
                node(events, value, depth + 1, output)?;
            }
            output.push(']');
        }
        _ => return Err("unsupported YAML event, anchor, alias or tag".into()),
    }
    Ok(())
}

fn convert(source: &str) -> Result<String, String> {
    if source.len() as u64 > MAX_BYTES {
        return Err("source size exceeded".into());
    }
    let mut collected = Vec::new();
    let mut depth = 0_usize;
    for result in Parser::new_from_str(source) {
        if collected.len() >= MAX_EVENTS {
            return Err("YAML event limit exceeded".into());
        }
        let (event, _) = result.map_err(|_| "invalid YAML")?;
        match &event {
            Event::MappingStart(0, None) | Event::SequenceStart(0, None) => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return Err("YAML depth exceeded".into());
                }
            }
            Event::MappingEnd | Event::SequenceEnd => {
                depth = depth.checked_sub(1).ok_or("incoherent YAML depth")?;
            }
            Event::Alias(_)
            | Event::MappingStart(_, _)
            | Event::SequenceStart(_, _)
            | Event::Scalar(_, _, 1.., _)
            | Event::Scalar(_, _, _, Some(_)) => {
                return Err("YAML anchors, aliases and tags forbidden".into());
            }
            _ => {}
        }
        collected.push(event);
    }
    let mut events = collected.into_iter();
    if !matches!(events.next(), Some(Event::StreamStart))
        || !matches!(events.next(), Some(Event::DocumentStart(_)))
    {
        return Err("missing YAML document".into());
    }
    let root = events.next().ok_or("missing YAML root")?;
    let mut output = String::new();
    node(&mut events, root, 0, &mut output)?;
    if !matches!(events.next(), Some(Event::DocumentEnd))
        || !matches!(events.next(), Some(Event::StreamEnd))
        || events.next().is_some()
    {
        return Err("expected exactly one document".into());
    }
    Ok(output)
}

fn run() -> Result<(), String> {
    let mut bytes = Vec::new();
    std::io::stdin()
        .take(MAX_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| "cannot read source")?;
    let source = std::str::from_utf8(&bytes).map_err(|_| "source is not UTF-8")?;
    let output = convert(source)?;
    std::io::stdout()
        .write_all(output.as_bytes())
        .map_err(|_| "cannot write JSON".into())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("source YAML: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_json_values_and_escaping() {
        assert_eq!(
            convert("a: [1, true, null, 0.25]\nb: |\n  a\"b\\c\n"),
            Ok("{\"a\":[1,true,null,0.25],\"b\":\"a\\\"b\\\\c\\u000a\"}".into())
        );
    }

    #[test]
    fn rejects_ambiguous_or_expansive_documents() {
        for source in [
            "a: 1\na: 2",
            "a: {b: 1, b: 2}",
            "a: &a [1]",
            "a: *a",
            "a: !!str text",
            "a: !thing text",
            "<<: {}",
            "1: value",
            "a: .nan",
            "---\na: 1\n---\na: 2",
            "a: [",
        ] {
            assert!(
                convert(source).is_err(),
                "accepted forbidden fixture: {source}"
            );
        }
        assert!(convert(&format!("{}0{}", "[".repeat(65), "]".repeat(65))).is_err());
        assert!(convert(&" ".repeat(MAX_BYTES as usize + 1)).is_err());
        assert!(convert(&format!("[{}]", "0,".repeat(MAX_EVENTS))).is_err());
    }
}
