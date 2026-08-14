use core::fmt::{Display, Write};

/// Compares a displayed value with expected text without allocating.
pub(super) fn display_matches(expected: &str, value: impl Display) -> bool {
    let mut comparison = CanonicalText::new(expected);
    write!(&mut comparison, "{value}").is_ok() && comparison.complete()
}

struct CanonicalText<'a> {
    expected: &'a [u8],
    offset: usize,
}

impl<'a> CanonicalText<'a> {
    const fn new(expected: &'a str) -> Self {
        Self {
            expected: expected.as_bytes(),
            offset: 0,
        }
    }

    const fn complete(&self) -> bool {
        self.offset == self.expected.len()
    }
}

impl Write for CanonicalText<'_> {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self
            .offset
            .checked_add(value.len())
            .ok_or(core::fmt::Error)?;
        if self.expected.get(self.offset..end) != Some(value.as_bytes()) {
            return Err(core::fmt::Error);
        }
        self.offset = end;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use core::fmt;

    use super::display_matches;

    struct Fragmented;

    impl fmt::Display for Fragmented {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("canonical")?;
            formatter.write_str(" text")
        }
    }

    #[test]
    fn comparison_requires_the_complete_exact_display_without_allocation() {
        assert!(display_matches("canonical text", Fragmented));
        assert!(!display_matches("canonical", Fragmented));
        assert!(!display_matches("canonical text suffix", Fragmented));
        assert!(!display_matches("Canonical text", Fragmented));
    }
}
