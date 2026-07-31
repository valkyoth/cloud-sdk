use core::fmt;

/// Raw query state admitted only through a validated provider pagination link.
///
/// This value has no public constructor. It preserves provider URI syntax
/// exactly and cannot be passed to [`super::RequestTarget::assemble`].
///
/// ```compile_fail
/// use cloud_sdk::transport::ProviderLinkQuery;
/// let _query = ProviderLinkQuery("page=2");
/// ```
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct ProviderLinkQuery<'a>(pub(super) &'a str);

impl<'a> ProviderLinkQuery<'a> {
    /// Returns the exact provider-supplied query bytes as text.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

impl fmt::Debug for ProviderLinkQuery<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ProviderLinkQuery([redacted])")
    }
}
