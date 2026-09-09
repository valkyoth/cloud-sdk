use super::{Cursor, Direction, PaginationError};
use crate::query::{Page, Query, Seek};

pub(super) fn check_query<'a>(
    actual: &'a str,
    expected: &str,
    current: Query<'_>,
    direction: Direction,
) -> Result<Cursor<'a>, PaginationError> {
    let mut matched = [false; 64];
    let mut cursor = None;
    for (index, pair) in actual.split('&').enumerate() {
        if index >= matched.len() {
            return Err(PaginationError::Limit);
        }
        let (key, value) = pair.split_once('=').ok_or(PaginationError::Invalid)?;
        if equivalent(key, "page")? || equivalent(key, "seek")? {
            if cursor.is_some() {
                return Err(PaginationError::Invalid);
            }
            // Continuation keys and values have canonical ASCII spellings. No
            // percent decoding or reconstruction of opaque cursor payloads.
            cursor = Some(match key {
                "page" => Cursor::Page(Page::parse(value).map_err(|_| PaginationError::Limit)?),
                "seek" => Cursor::Seek(Seek::new(value)?),
                _ => return Err(PaginationError::Invalid),
            });
            continue;
        }
        let mut found = false;
        for (i, original) in expected.split('&').filter(|p| !p.is_empty()).enumerate() {
            let (old_key, old_value) = original.split_once('=').ok_or(PaginationError::Invalid)?;
            if matches!(old_key, "page" | "seek") {
                continue;
            }
            let seen = matched.get_mut(i).ok_or(PaginationError::Limit)?;
            if !*seen && equivalent(key, old_key)? && equivalent(value, old_value)? {
                *seen = true;
                found = true;
                break;
            }
        }
        if !found {
            return Err(PaginationError::Binding);
        }
    }
    for (i, pair) in expected.split('&').filter(|p| !p.is_empty()).enumerate() {
        let (key, _) = pair.split_once('=').ok_or(PaginationError::Invalid)?;
        if !matches!(key, "page" | "seek") && !matched.get(i).copied().unwrap_or(false) {
            return Err(PaginationError::Binding);
        }
    }
    let cursor = cursor.ok_or(PaginationError::Invalid)?;
    match cursor {
        Cursor::Page(page) => {
            if current.seek().is_some() {
                return Err(PaginationError::Progress);
            }
            let prior = current.page().map(Page::get).unwrap_or(1);
            let wanted = match direction {
                Direction::Next => prior.checked_add(1),
                Direction::Previous => prior.checked_sub(1),
            };
            if wanted != Some(page.get()) {
                return Err(PaginationError::Progress);
            }
        }
        Cursor::Seek(seek) => {
            if current.page().is_some() || current.seek() == Some(seek) {
                return Err(PaginationError::Progress);
            }
        }
    }
    Ok(cursor)
}

// Compare form query components one decoded byte at a time, without allocating
// or recursively decoding. Values are never treated as query syntax.
fn equivalent(left: &str, right: &str) -> Result<bool, PaginationError> {
    let mut left = left.bytes();
    let mut right = right.bytes();
    loop {
        let a = next(&mut left)?;
        let b = next(&mut right)?;
        if a != b {
            return Ok(false);
        }
        if a.is_none() {
            return Ok(true);
        }
    }
}
fn next(bytes: &mut core::str::Bytes<'_>) -> Result<Option<u8>, PaginationError> {
    match bytes.next() {
        None => Ok(None),
        Some(b'+') => Ok(Some(b' ')),
        Some(b'%') => {
            let a = hex(bytes.next().ok_or(PaginationError::Invalid)?)?;
            let b = hex(bytes.next().ok_or(PaginationError::Invalid)?)?;
            Ok(Some(
                a.checked_mul(16)
                    .and_then(|v| v.checked_add(b))
                    .ok_or(PaginationError::Invalid)?,
            ))
        }
        Some(b) => Ok(Some(b)),
    }
}
fn hex(b: u8) -> Result<u8, PaginationError> {
    match b {
        b'0'..=b'9' => b.checked_sub(b'0'),
        b'A'..=b'F' => b.checked_sub(b'A').and_then(|v| v.checked_add(10)),
        b'a'..=b'f' => b.checked_sub(b'a').and_then(|v| v.checked_add(10)),
        _ => None,
    }
    .ok_or(PaginationError::Invalid)
}
