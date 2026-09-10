//! Inspection of a transport-owned identifying user-agent.

/// A transport with one stable, explicitly configured user-agent.
///
/// Implementations must return the exact bytes applied to every request and
/// must not change them through interior mutability between verification and
/// dispatch. This is a trusted-adapter contract, like [`super::BoundTransport`].
/// Providers can require an identifying value without allowing ordinary
/// request headers to override transport-owned configuration.
pub trait BoundUserAgent {
    /// Exact configured value. It must be non-secret; Debug need not expose it.
    fn configured_user_agent(&self) -> &[u8];
}
