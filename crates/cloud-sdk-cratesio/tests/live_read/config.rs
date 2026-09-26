use cloud_sdk_cratesio::{
    credentials::{ApiToken, CredentialOrigin},
    wire::IdentifyingUserAgent,
};
use cloud_sdk_sanitization::SecretBuffer;
use std::io::Read;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Anonymous,
    Token,
}

pub fn mode(value: &str, ci: bool, acknowledgement: Option<&str>) -> Result<Mode, &'static str> {
    if ci {
        return Err("live crates.io execution is forbidden in CI");
    }
    match (value, acknowledgement) {
        ("anonymous", None) => Ok(Mode::Anonymous),
        ("token", Some("isolated-account-read-requests")) => Ok(Mode::Token),
        _ => Err("explicit live mode and isolated-account acknowledgement required"),
    }
}

pub fn identity(value: &str) -> Result<IdentifyingUserAgent<'_>, &'static str> {
    IdentifyingUserAgent::new(value).map_err(|_| "identifying user agent required")
}

// Read at most one byte beyond the credential plus optional CRLF. The full
// fixed buffer clears even on reader errors; no immutable secret String exists.
pub fn read_token(reader: &mut impl Read) -> Result<ApiToken, &'static str> {
    let mut storage = [0; 1027];
    let mut guard = SecretBuffer::new(&mut storage);
    let mut len = 0_usize;
    loop {
        let remaining = guard
            .as_mut_slice()
            .get_mut(len..)
            .ok_or("token input limit")?;
        if remaining.is_empty() {
            return Err("token input limit");
        }
        match reader.read(remaining) {
            Ok(0) => break,
            Ok(read) => len = len.checked_add(read).ok_or("token input limit")?,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(_) => return Err("credential input failed"),
        }
    }
    let bytes = guard
        .as_mut_slice()
        .get_mut(..len)
        .ok_or("token input limit")?;
    let suffix = if bytes.ends_with(b"\r\n") {
        2
    } else {
        usize::from(bytes.ends_with(b"\n"))
    };
    let end = len.checked_sub(suffix).ok_or("token input limit")?;
    ApiToken::from_mut_bytes(
        CredentialOrigin::Production,
        bytes.get_mut(..end).ok_or("token input limit")?,
    )
    .map_err(|_| "invalid credential input")
}
