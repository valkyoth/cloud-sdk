use std::env;
use std::ffi::OsStr;
use std::fmt;
use std::fs::{self, File, Metadata};
use std::io::Read;
use std::path::{Path, PathBuf};

use cloud_sdk_reqwest::blocking::{BearerToken, MAX_BEARER_TOKEN_BYTES};
use cloud_sdk_sanitization::SecretBuffer;

const LIVE_MODE_ENV: &str = "CLOUD_SDK_OVHCLOUD_LIVE_MODE";
const TOKEN_FILE_ENV: &str = "CLOUD_SDK_OVHCLOUD_TOKEN_FILE";
const DESTRUCTIVE_ENV: &str = "CLOUD_SDK_OVHCLOUD_ALLOW_DESTRUCTIVE";
const MAX_TOKEN_FILE_BYTES: u64 = 4_098;
const MAX_TOKEN_READ_BYTES: u64 = 4_099;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LiveConfigurationError {
    LiveModeRequired,
    DestructiveModeForbidden,
    TokenFileRequired,
    TokenFileMetadataUnavailable,
    TokenFileSymlink,
    TokenFileNotRegular,
    #[cfg(not(unix))]
    TokenFilePlatformUnsupported,
    TokenFilePermissionsTooBroad,
    TokenFileChangedDuringOpen,
    TokenFileOpenFailed,
    TokenFileTooLarge,
    TokenFileReadFailed,
    TokenRejected,
}

impl fmt::Display for LiveConfigurationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::LiveModeRequired => "exact read-only live mode is required",
            Self::DestructiveModeForbidden => "destructive live mode is forbidden",
            Self::TokenFileRequired => "a token file is required",
            Self::TokenFileMetadataUnavailable => "token file metadata is unavailable",
            Self::TokenFileSymlink => "token file symlinks are forbidden",
            Self::TokenFileNotRegular => "token file is not regular",
            #[cfg(not(unix))]
            Self::TokenFilePlatformUnsupported => {
                "token file security checks are unsupported on this platform"
            }
            Self::TokenFilePermissionsTooBroad => "token file permissions are too broad",
            Self::TokenFileChangedDuringOpen => "token file changed during open",
            Self::TokenFileOpenFailed => "token file could not be opened",
            Self::TokenFileTooLarge => "token file exceeds the size limit",
            Self::TokenFileReadFailed => "token file could not be read",
            Self::TokenRejected => "token bytes were rejected",
        })
    }
}

impl core::error::Error for LiveConfigurationError {}

pub fn load_read_only_token() -> Result<BearerToken, LiveConfigurationError> {
    require_read_only_mode()?;
    let path = token_file_path()?;
    read_token_file(&path)
}

fn require_read_only_mode() -> Result<(), LiveConfigurationError> {
    validate_live_mode(
        env::var_os(LIVE_MODE_ENV).as_deref(),
        env::var_os(DESTRUCTIVE_ENV).is_some(),
    )
}

fn validate_live_mode(
    live_mode: Option<&OsStr>,
    destructive_present: bool,
) -> Result<(), LiveConfigurationError> {
    if destructive_present {
        return Err(LiveConfigurationError::DestructiveModeForbidden);
    }
    if live_mode != Some(OsStr::new("read-only")) {
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

    let capacity = usize::try_from(MAX_TOKEN_READ_BYTES)
        .map_err(|_| LiveConfigurationError::TokenFileReadFailed)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| LiveConfigurationError::TokenFileReadFailed)?;
    read_token_from(file, &mut bytes)
}

fn read_token_from<R: Read>(
    reader: R,
    bytes: &mut Vec<u8>,
) -> Result<BearerToken, LiveConfigurationError> {
    let read_result = reader.take(MAX_TOKEN_READ_BYTES).read_to_end(bytes);
    let guarded = SecretBuffer::new(bytes.as_mut_slice());
    read_result.map_err(|_| LiveConfigurationError::TokenFileReadFailed)?;
    if guarded.as_slice().len() > usize::try_from(MAX_TOKEN_FILE_BYTES).unwrap_or(usize::MAX) {
        return Err(LiveConfigurationError::TokenFileTooLarge);
    }
    let token_len = normalized_token_len(guarded.as_slice())?;
    let token = guarded
        .as_slice()
        .get(..token_len)
        .ok_or(LiveConfigurationError::TokenRejected)?;
    if token.len() > MAX_BEARER_TOKEN_BYTES {
        return Err(LiveConfigurationError::TokenFileTooLarge);
    }
    let token = core::str::from_utf8(token).map_err(|_| LiveConfigurationError::TokenRejected)?;
    BearerToken::new(token).map_err(|_| LiveConfigurationError::TokenRejected)
}

fn normalized_token_len(bytes: &[u8]) -> Result<usize, LiveConfigurationError> {
    let without_lf = bytes.strip_suffix(b"\n").unwrap_or(bytes);
    let token = without_lf.strip_suffix(b"\r").unwrap_or(without_lf);
    if token.is_empty()
        || token.first().is_some_and(u8::is_ascii_whitespace)
        || token.last().is_some_and(u8::is_ascii_whitespace)
    {
        return Err(LiveConfigurationError::TokenRejected);
    }
    Ok(token.len())
}

fn validate_metadata(metadata: &Metadata) -> Result<(), LiveConfigurationError> {
    if !metadata.is_file() {
        return Err(LiveConfigurationError::TokenFileNotRegular);
    }
    validate_private_permissions(metadata)
}

#[cfg(unix)]
fn validate_private_permissions(metadata: &Metadata) -> Result<(), LiveConfigurationError> {
    use std::os::unix::fs::PermissionsExt;

    if metadata.permissions().mode() & 0o077 != 0 {
        return Err(LiveConfigurationError::TokenFilePermissionsTooBroad);
    }
    Ok(())
}

#[cfg(not(unix))]
fn validate_private_permissions(_metadata: &Metadata) -> Result<(), LiveConfigurationError> {
    Err(LiveConfigurationError::TokenFilePlatformUnsupported)
}

#[cfg(unix)]
fn same_opened_file(before: &Metadata, opened: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    before.dev() == opened.dev() && before.ino() == opened.ino()
}

#[cfg(not(unix))]
fn same_opened_file(_before: &Metadata, _opened: &Metadata) -> bool {
    false
}

#[cfg(test)]
mod tests {
    use std::fs::{self, DirBuilder, OpenOptions};
    use std::io::{self, Read, Write};
    use std::path::{Path, PathBuf};
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::{
        LiveConfigurationError, normalized_token_len, read_token_file, read_token_from,
        validate_live_mode,
    };

    static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> std::io::Result<Self> {
            let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let name = format!("cloud-sdk-ovhcloud-smoke-{}-{sequence}", std::process::id());
            let path = std::env::temp_dir().join(name);
            let mut builder = DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(&path)?;
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

    fn write_private(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
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
    fn mode_requires_exact_read_only_and_forbids_destructive_presence() {
        assert_eq!(
            validate_live_mode(Some(std::ffi::OsStr::new("read-only")), false),
            Ok(())
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
    fn token_normalization_accepts_one_line_ending_only() {
        assert_eq!(normalized_token_len(b"token"), Ok(5));
        assert_eq!(normalized_token_len(b"token\n"), Ok(5));
        assert_eq!(normalized_token_len(b"token\r\n"), Ok(5));
        for rejected in [b"".as_slice(), b" token", b"token ", b"token\n\n"] {
            assert_eq!(
                normalized_token_len(rejected),
                Err(LiveConfigurationError::TokenRejected)
            );
        }
    }

    struct PrefixThenError {
        prefix: Option<&'static [u8]>,
    }

    impl Read for PrefixThenError {
        fn read(&mut self, output: &mut [u8]) -> io::Result<usize> {
            let Some(prefix) = self.prefix.take() else {
                return Err(io::Error::other("injected token read failure"));
            };
            let length = prefix.len().min(output.len());
            let Some(destination) = output.get_mut(..length) else {
                unreachable!("bounded reader destination became invalid");
            };
            let Some(source) = prefix.get(..length) else {
                unreachable!("bounded reader source became invalid");
            };
            destination.copy_from_slice(source);
            Ok(length)
        }
    }

    #[test]
    fn partial_token_read_is_cleared_before_error_propagation() {
        let mut bytes = Vec::with_capacity(64);
        let result = read_token_from(
            PrefixThenError {
                prefix: Some(b"secret-token-prefix"),
            },
            &mut bytes,
        );
        assert!(matches!(
            result,
            Err(LiveConfigurationError::TokenFileReadFailed)
        ));
        assert_eq!(bytes, vec![0; b"secret-token-prefix".len()]);
    }

    #[cfg(unix)]
    #[test]
    fn private_regular_token_file_is_accepted_with_redacted_output() -> std::io::Result<()> {
        let directory = TempDirectory::new()?;
        let path = directory.path().join("token");
        write_private(&path, b"secret-token\n")?;
        let token = read_token_file(&path);
        assert!(token.is_ok());
        let Ok(token) = token else {
            return Ok(());
        };
        let diagnostic = format!("{token:?}");
        assert_eq!(diagnostic, "BearerToken([redacted])");
        assert!(!diagnostic.contains("secret-token"));
        Ok(())
    }

    #[cfg(not(unix))]
    #[test]
    fn token_file_loading_fails_closed_on_unsupported_platform() -> std::io::Result<()> {
        let directory = TempDirectory::new()?;
        let path = directory.path().join("token");
        write_private(&path, b"secret-token\n")?;

        assert!(matches!(
            read_token_file(&path),
            Err(LiveConfigurationError::TokenFilePlatformUnsupported)
        ));
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_and_broad_permissions_are_rejected() -> std::io::Result<()> {
        use std::os::unix::fs::{PermissionsExt, symlink};

        let directory = TempDirectory::new()?;
        let private = directory.path().join("private");
        write_private(&private, b"token")?;
        let link = directory.path().join("link");
        symlink(&private, &link)?;
        assert!(matches!(
            read_token_file(&link),
            Err(LiveConfigurationError::TokenFileSymlink)
        ));

        fs::set_permissions(&private, fs::Permissions::from_mode(0o644))?;
        assert!(matches!(
            read_token_file(&private),
            Err(LiveConfigurationError::TokenFilePermissionsTooBroad)
        ));
        Ok(())
    }
}
