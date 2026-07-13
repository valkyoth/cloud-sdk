use std::env;
use std::ffi::OsStr;
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Path, PathBuf};

use cloud_sdk_reqwest::blocking::{BearerToken, MAX_BEARER_TOKEN_BYTES};
use cloud_sdk_sanitization::SecretBuffer;

const LIVE_MODE_ENV: &str = "CLOUD_SDK_HETZNER_LIVE_MODE";
const TOKEN_FILE_ENV: &str = "CLOUD_SDK_HETZNER_TOKEN_FILE";
const DESTRUCTIVE_ENV: &str = "CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE";
const READ_ONLY_MODE: &str = "read-only";
const MAX_TOKEN_FILE_BYTES: u64 = 4_098;
const MAX_TOKEN_READ_BYTES: u64 = 4_099;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum LiveConfigurationError {
    LiveModeRequired,
    DestructiveModeForbidden,
    TokenFileRequired,
    TokenFileMetadataUnavailable,
    TokenFileSymlink,
    TokenFileNotRegular,
    TokenFilePermissionsTooBroad,
    TokenFileChangedDuringOpen,
    TokenFileOpenFailed,
    TokenFileTooLarge,
    TokenFileReadFailed,
    TokenEncodingInvalid,
    TokenRejected,
}

pub(super) fn load_read_only_token() -> Result<BearerToken, LiveConfigurationError> {
    require_read_only_mode()?;
    let path = token_file_path()?;
    read_token_file(&path)
}

fn require_read_only_mode() -> Result<(), LiveConfigurationError> {
    let live_mode = env::var_os(LIVE_MODE_ENV);
    validate_live_mode(live_mode.as_deref(), env::var_os(DESTRUCTIVE_ENV).is_some())
}

fn validate_live_mode(
    live_mode: Option<&OsStr>,
    destructive_present: bool,
) -> Result<(), LiveConfigurationError> {
    if destructive_present {
        return Err(LiveConfigurationError::DestructiveModeForbidden);
    }
    if live_mode != Some(OsStr::new(READ_ONLY_MODE)) {
        return Err(LiveConfigurationError::LiveModeRequired);
    }
    Ok(())
}

fn token_file_path() -> Result<PathBuf, LiveConfigurationError> {
    let value = env::var_os(TOKEN_FILE_ENV).ok_or(LiveConfigurationError::TokenFileRequired)?;
    if value.is_empty() {
        return Err(LiveConfigurationError::TokenFileRequired);
    }
    Ok(PathBuf::from(value))
}

fn read_token_file(path: &Path) -> Result<BearerToken, LiveConfigurationError> {
    let before = fs::symlink_metadata(path)
        .map_err(|_| LiveConfigurationError::TokenFileMetadataUnavailable)?;
    if before.file_type().is_symlink() {
        return Err(LiveConfigurationError::TokenFileSymlink);
    }
    validate_metadata(&before)?;

    let file = File::open(path).map_err(|_| LiveConfigurationError::TokenFileOpenFailed)?;
    let opened = file
        .metadata()
        .map_err(|_| LiveConfigurationError::TokenFileMetadataUnavailable)?;
    validate_metadata(&opened)?;
    if !same_opened_file(&before, &opened) {
        return Err(LiveConfigurationError::TokenFileChangedDuringOpen);
    }
    if opened.len() > MAX_TOKEN_FILE_BYTES {
        return Err(LiveConfigurationError::TokenFileTooLarge);
    }

    let mut bytes = token_read_buffer()?;
    let read_result = file.take(MAX_TOKEN_READ_BYTES).read_to_end(&mut bytes);
    let guarded = SecretBuffer::new(bytes.as_mut_slice());
    read_result.map_err(|_| LiveConfigurationError::TokenFileReadFailed)?;
    if guarded.as_slice().len() > usize::try_from(MAX_TOKEN_FILE_BYTES).unwrap_or(usize::MAX) {
        return Err(LiveConfigurationError::TokenFileTooLarge);
    }

    let token_bytes = normalized_token(guarded.as_slice())?;
    let token = core::str::from_utf8(token_bytes)
        .map_err(|_| LiveConfigurationError::TokenEncodingInvalid)?;
    if token.len() > MAX_BEARER_TOKEN_BYTES {
        return Err(LiveConfigurationError::TokenFileTooLarge);
    }
    BearerToken::new(token).map_err(|_| LiveConfigurationError::TokenRejected)
}

fn token_read_buffer() -> Result<Vec<u8>, LiveConfigurationError> {
    let read_capacity = usize::try_from(MAX_TOKEN_READ_BYTES)
        .map_err(|_| LiveConfigurationError::TokenFileReadFailed)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(read_capacity)
        .map_err(|_| LiveConfigurationError::TokenFileReadFailed)?;
    Ok(bytes)
}

fn validate_metadata(metadata: &Metadata) -> Result<(), LiveConfigurationError> {
    if !metadata.is_file() {
        return Err(LiveConfigurationError::TokenFileNotRegular);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(LiveConfigurationError::TokenFilePermissionsTooBroad);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn same_opened_file(before: &Metadata, opened: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;

    before.dev() == opened.dev() && before.ino() == opened.ino()
}

#[cfg(not(unix))]
fn same_opened_file(_before: &Metadata, _opened: &Metadata) -> bool {
    true
}

fn normalized_token(bytes: &[u8]) -> Result<&[u8], LiveConfigurationError> {
    let without_lf = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let token = without_lf.strip_suffix(b"\r").unwrap_or(without_lf);
    if token.is_empty()
        || token.first().is_some_and(u8::is_ascii_whitespace)
        || token.last().is_some_and(u8::is_ascii_whitespace)
    {
        return Err(LiveConfigurationError::TokenRejected);
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use std::fs::{self, OpenOptions};
    use std::io::Write;
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{
        LiveConfigurationError, MAX_TOKEN_READ_BYTES, normalized_token, read_token_file,
        token_read_buffer, validate_live_mode,
    };

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> std::io::Result<Self> {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let name = format!("cloud-sdk-live-smoke-{}-{sequence}", std::process::id());
            let path = std::env::temp_dir().join(name);
            fs::create_dir(&path)?;
            Ok(Self(path))
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn write_private_file(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;

            options.mode(0o600);
        }
        let mut file = options.open(path)?;
        file.write_all(bytes)
    }

    #[test]
    fn accepts_exact_token_or_one_terminal_line_ending() {
        assert_eq!(normalized_token(b"abc-123"), Ok(b"abc-123".as_slice()));
        assert_eq!(normalized_token(b"abc-123\n"), Ok(b"abc-123".as_slice()));
        assert_eq!(normalized_token(b"abc-123\r\n"), Ok(b"abc-123".as_slice()));
    }

    #[test]
    fn rejects_empty_or_boundary_whitespace() {
        for value in [b"".as_slice(), b"\n", b" token", b"token ", b"token\n\n"] {
            assert_eq!(
                normalized_token(value),
                Err(LiveConfigurationError::TokenRejected)
            );
        }
    }

    #[test]
    fn requires_exact_read_only_mode_and_rejects_destructive_presence() {
        assert_eq!(
            validate_live_mode(Some(std::ffi::OsStr::new("read-only")), false),
            Ok(())
        );
        assert_eq!(
            validate_live_mode(None, false),
            Err(LiveConfigurationError::LiveModeRequired)
        );
        assert_eq!(
            validate_live_mode(Some(std::ffi::OsStr::new("READ-ONLY")), false),
            Err(LiveConfigurationError::LiveModeRequired)
        );
        assert_eq!(
            validate_live_mode(Some(std::ffi::OsStr::new("read-only")), true),
            Err(LiveConfigurationError::DestructiveModeForbidden)
        );
    }

    #[test]
    fn reserves_the_complete_token_read_bound_before_io() {
        let result = token_read_buffer();
        assert!(result.is_ok());
        let Ok(buffer) = result else { return };
        let Ok(required) = usize::try_from(MAX_TOKEN_READ_BYTES) else {
            return;
        };
        assert!(buffer.is_empty());
        assert!(buffer.capacity() >= required);
    }

    #[test]
    fn reads_a_private_regular_token_file_with_redacted_diagnostics() {
        let Ok(directory) = TempDirectory::new() else {
            return;
        };
        let path = directory.path().join("token");
        assert!(write_private_file(&path, b"secret-token\n").is_ok());
        let result = read_token_file(&path);
        assert!(result.is_ok());
        let Ok(token) = result else { return };
        let diagnostic = format!("{token:?}");
        assert_eq!(diagnostic, "BearerToken([redacted])");
        assert!(!diagnostic.contains("secret-token"));
    }

    #[test]
    fn rejects_non_regular_and_oversized_token_files() {
        let Ok(directory) = TempDirectory::new() else {
            return;
        };
        assert!(matches!(
            read_token_file(directory.path()),
            Err(LiveConfigurationError::TokenFileNotRegular)
        ));

        let path = directory.path().join("oversized-token");
        assert!(write_private_file(&path, &[b'a'; 4_099]).is_ok());
        assert!(matches!(
            read_token_file(&path),
            Err(LiveConfigurationError::TokenFileTooLarge)
        ));
    }

    #[cfg(unix)]
    #[test]
    fn rejects_symlinks_and_group_or_world_access() {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let Ok(directory) = TempDirectory::new() else {
            return;
        };
        let path = directory.path().join("token");
        assert!(write_private_file(&path, b"secret-token\n").is_ok());
        let link = directory.path().join("token-link");
        assert!(symlink(&path, &link).is_ok());
        assert!(matches!(
            read_token_file(&link),
            Err(LiveConfigurationError::TokenFileSymlink)
        ));

        assert!(fs::set_permissions(&path, fs::Permissions::from_mode(0o640)).is_ok());
        assert!(matches!(
            read_token_file(&path),
            Err(LiveConfigurationError::TokenFilePermissionsTooBroad)
        ));
    }
}
