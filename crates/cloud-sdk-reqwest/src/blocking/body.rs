use std::io::{Cursor, Read, Result as IoResult};
use std::vec::Vec;

use cloud_sdk_sanitization::sanitize_bytes;

pub(super) struct SanitizedRequestBody {
    cursor: Cursor<Vec<u8>>,
}

impl SanitizedRequestBody {
    pub(super) fn new(source: &[u8]) -> Result<Self, ()> {
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(source.len()).map_err(|_| ())?;
        bytes.extend_from_slice(source);
        Ok(Self {
            cursor: Cursor::new(bytes),
        })
    }
}

impl Read for SanitizedRequestBody {
    fn read(&mut self, output: &mut [u8]) -> IoResult<usize> {
        self.cursor.read(output)
    }
}

impl Drop for SanitizedRequestBody {
    fn drop(&mut self) {
        sanitize_bytes(self.cursor.get_mut());
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ReadBodyError {
    TooLarge,
    ReadFailed,
}

pub(super) fn read_bounded(
    reader: &mut impl Read,
    output: &mut [u8],
) -> Result<usize, ReadBodyError> {
    let mut len = 0_usize;
    while len < output.len() {
        let target = output.get_mut(len..).ok_or(ReadBodyError::ReadFailed)?;
        let read = reader.read(target).map_err(|_| ReadBodyError::ReadFailed)?;
        if read == 0 {
            return Ok(len);
        }
        len = len.checked_add(read).ok_or(ReadBodyError::TooLarge)?;
    }

    let mut probe = [0_u8; 1];
    let extra = reader
        .read(&mut probe)
        .map_err(|_| ReadBodyError::ReadFailed)?;
    if extra == 0 {
        Ok(len)
    } else {
        Err(ReadBodyError::TooLarge)
    }
}
