use core::fmt;

/// Maximum identifying user-agent value accepted by the SDK.
pub const MAX_USER_AGENT_BYTES: usize = 256;

/// Explicit application identity, not a default SDK-only user agent.
///
/// Format: `product/version (contact)`. The contact is a trusted operator email
/// or HTTPS URL, not a credential. Syntax cannot establish its real ownership.
/// Email contacts use ASCII dot-atoms (at most 64 local-part bytes); both
/// contact forms require dotted DNS names with 1 through 63 byte labels.
/// Quoted local parts, address literals, and internationalized text are excluded.
#[derive(Clone, Copy)]
pub struct IdentifyingUserAgent<'a>(&'a str);

impl<'a> IdentifyingUserAgent<'a> {
    /// Validates a bounded, single-line identifying HTTP user agent.
    pub fn new(value: &'a str) -> Result<Self, UserAgentError> {
        if value.is_empty()
            || value.len() > MAX_USER_AGENT_BYTES
            || !value.bytes().all(|byte| (b' '..=b'~').contains(&byte))
        {
            return Err(UserAgentError);
        }
        let (product, contact) = value.split_once(" (").ok_or(UserAgentError)?;
        let contact = contact.strip_suffix(')').ok_or(UserAgentError)?;
        let (name, version) = product.split_once('/').ok_or(UserAgentError)?;
        let token = |text: &str| {
            !text.is_empty()
                && text.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+')
                })
        };
        if !token(name) || !token(version) || !valid_contact(contact) {
            return Err(UserAgentError);
        }
        Ok(Self(value))
    }

    /// Lends identifying text to a trusted header adapter. Never put secrets here.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.0
    }
}

fn valid_contact(contact: &str) -> bool {
    if contact.is_empty()
        || contact
            .bytes()
            .any(|byte| byte <= b' ' || matches!(byte, b'(' | b')' | b'\\' | b'"' | b'<' | b'>'))
    {
        return false;
    }
    if let Some(url) = contact.strip_prefix("https://") {
        let host = url.split('/').next().unwrap_or_default();
        return !url.contains(['@', '#', '?']) && valid_contact_domain(host);
    }
    valid_email_contact(contact)
}

fn valid_email_contact(contact: &str) -> bool {
    let Some((local, domain)) = contact.split_once('@') else {
        return false;
    };
    local.len() <= 64
        && local.split('.').all(|atom| {
            !atom.is_empty()
                && atom.bytes().all(|byte| {
                    byte.is_ascii_alphanumeric() || b"!#$%&'*+-/=?^_`{|}~".contains(&byte)
                })
        })
        && valid_contact_domain(domain)
}

fn valid_contact_domain(domain: &str) -> bool {
    domain.contains('.')
        && domain.split('.').all(|label| {
            label.len() <= 63
                && label
                    .as_bytes()
                    .first()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .as_bytes()
                    .last()
                    .is_some_and(u8::is_ascii_alphanumeric)
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
        })
}

impl fmt::Debug for IdentifyingUserAgent<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("IdentifyingUserAgent([redacted])")
    }
}

/// Missing, oversized, unsafe, or non-identifying user agent.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UserAgentError;

impl fmt::Display for UserAgentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("crates.io requires a bounded identifying user agent")
    }
}

impl core::error::Error for UserAgentError {}
