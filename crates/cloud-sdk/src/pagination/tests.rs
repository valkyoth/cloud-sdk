use core::fmt::{self, Write};

mod link;
mod numbered_budget;
mod offset;
mod opaque;

fn assert_redacted(value: &impl fmt::Debug) {
    let mut output = DebugBuffer::new();
    assert!(write!(&mut output, "{value:?}").is_ok());
    assert!(output.as_str().contains("[redacted]"));
}

struct DebugBuffer {
    bytes: [u8; 128],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let output = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        output.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}
