//! Cleanup ownership for ordinary strings during fallible model parsing.

use alloc::string::String;
use alloc::vec::Vec;

use cloud_sdk_sanitization::sanitize_string;

use super::ResponseModelError;

/// Owns one string until cleanup responsibility moves into a final model.
pub(super) struct WipeString(String);

impl WipeString {
    pub(super) const fn new(value: String) -> Self {
        Self(value)
    }

    pub(super) fn as_str(&self) -> &str {
        &self.0
    }

    pub(super) fn into_inner(mut self) -> String {
        core::mem::take(&mut self.0)
    }
}

impl Drop for WipeString {
    fn drop(&mut self) {
        sanitize_string(&mut self.0);
    }
}

/// Owns partially parsed string collections until a final model takes them.
pub(super) struct WipeStrings(Vec<String>);

impl WipeStrings {
    pub(super) fn with_capacity(capacity: usize) -> Result<Self, ResponseModelError> {
        let mut values = Vec::new();
        values
            .try_reserve_exact(capacity)
            .map_err(|_| ResponseModelError::Allocation)?;
        Ok(Self(values))
    }

    pub(super) fn push(&mut self, value: WipeString) {
        self.0.push(value.into_inner());
    }

    pub(super) fn into_inner(mut self) -> Vec<String> {
        core::mem::take(&mut self.0)
    }
}

impl Drop for WipeStrings {
    fn drop(&mut self) {
        for value in &mut self.0 {
            sanitize_string(value);
        }
    }
}

#[cfg(test)]
mod tests {
    use alloc::string::String;

    use super::{WipeString, WipeStrings};

    #[test]
    fn ownership_transfer_preserves_text_without_a_second_copy() {
        let original = String::from("dns-topology");
        let pointer = original.as_ptr();
        let guarded = WipeString::new(original);
        let transferred = guarded.into_inner();

        assert_eq!(transferred, "dns-topology");
        assert_eq!(transferred.as_ptr(), pointer);
    }

    #[test]
    fn collection_transfer_preserves_guarded_allocations() {
        let original = String::from("ns1.example.test");
        let pointer = original.as_ptr();
        let mut guarded = WipeStrings::with_capacity(1)
            .unwrap_or_else(|_| unreachable!("cleanup fixture allocation failed"));
        guarded.push(WipeString::new(original));
        let transferred = guarded.into_inner();

        let Some(value) = transferred.first() else {
            unreachable!("cleanup fixture value disappeared")
        };
        assert_eq!(value, "ns1.example.test");
        assert_eq!(value.as_ptr(), pointer);
    }
}
