use std::io::Read;

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
