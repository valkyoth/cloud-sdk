use cloud_sdk_sanitization::SecretBoxBytes;

pub(crate) struct ProtectedBoolean(SecretBoxBytes);

impl ProtectedBoolean {
    pub(super) fn try_new(value: u8) -> Result<Self, ()> {
        SecretBoxBytes::try_from_fn_bounded(1, 1, |_| Ok::<u8, core::convert::Infallible>(value))
            .map(Self)
            .map_err(|_| ())
    }

    pub(super) fn value(&self) -> bool {
        self.0.with_secret(|bytes| bytes == [1])
    }

    pub(super) fn copy_byte_to(&self, destination: &mut u8) {
        self.0.with_secret(|bytes| {
            *destination = bytes.first().copied().unwrap_or(0);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::ProtectedBoolean;

    #[test]
    fn protected_boolean_allocation_survives_owner_moves() {
        let value = ProtectedBoolean::try_new(1)
            .unwrap_or_else(|_| unreachable!("protected Boolean allocation failed"));
        let before = value.0.with_secret(<[u8]>::as_ptr);
        let moved = value;
        let after = moved.0.with_secret(<[u8]>::as_ptr);

        assert_eq!(before, after);
        assert!(moved.value());
    }
}
