use std::env;
use std::ffi::OsStr;
use std::fs::{File, Metadata};
use std::io::Read;
use std::path::{Path, PathBuf};

use cloud_sdk_reqwest::blocking::{
    BasicCredential, BasicCredentialScope, HttpsEndpoint, MAX_BASIC_PASSWORD_BYTES,
    MAX_BASIC_USERNAME_BYTES,
};
use cloud_sdk_sanitization::sanitize_bytes;

use cloud_sdk_hetzner::{HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID};

const LIVE_MODE_ENV: &str = "CLOUD_SDK_HETZNER_LIVE_MODE";
const TOKEN_FILE_ENV: &str = "CLOUD_SDK_HETZNER_TOKEN_FILE";
const USERNAME_FILE_ENV: &str = "CLOUD_SDK_HETZNER_ROBOT_USERNAME_FILE";
const PASSWORD_FILE_ENV: &str = "CLOUD_SDK_HETZNER_ROBOT_PASSWORD_FILE";
const DESTRUCTIVE_ENV: &str = "CLOUD_SDK_HETZNER_ALLOW_DESTRUCTIVE";
const ROBOT_READ_ONLY_MODE: &str = "robot-read-only";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RobotLiveConfigurationError {
    LiveModeRequired,
    DestructiveModeForbidden,
    BearerTokenForbidden,
    CredentialFileRequired,
    CredentialFilesNotDistinct,
    #[cfg(not(unix))]
    UnsupportedPlatform,
    ParentDirectoryUntrusted,
    FileMetadataUnavailable,
    FileNotRegular,
    FileHasMultipleLinks,
    FileOwnerMismatch,
    FilePermissionsTooBroad,
    FileOpenFailed,
    FileTooLarge,
    FileReadFailed,
    CredentialRejected,
}

pub(super) fn load_read_only_robot_credential(
    endpoint: HttpsEndpoint,
) -> Result<BasicCredential, RobotLiveConfigurationError> {
    require_robot_read_only_mode()?;
    let username_path = credential_file_path(USERNAME_FILE_ENV)?;
    let password_path = credential_file_path(PASSWORD_FILE_ENV)?;
    if username_path == password_path {
        return Err(RobotLiveConfigurationError::CredentialFilesNotDistinct);
    }

    let scope = BasicCredentialScope::new(HETZNER_PROVIDER_ID, ROBOT_SERVICE_ID, endpoint);
    let mut username = read_secret_file(&username_path, MAX_BASIC_USERNAME_BYTES)?;
    let mut password = read_secret_file(&password_path, MAX_BASIC_PASSWORD_BYTES)?;
    if same_secret_file(&username, &password) {
        return Err(RobotLiveConfigurationError::CredentialFilesNotDistinct);
    }
    let username = normalized_secret(username.as_mut_slice())?;
    let password = normalized_secret(password.as_mut_slice())?;
    BasicCredential::from_mut_bytes(username, password, scope)
        .map_err(|_| RobotLiveConfigurationError::CredentialRejected)
}

fn require_robot_read_only_mode() -> Result<(), RobotLiveConfigurationError> {
    validate_live_mode(
        env::var_os(LIVE_MODE_ENV).as_deref(),
        env::var_os(DESTRUCTIVE_ENV).is_some(),
        env::var_os(TOKEN_FILE_ENV).is_some(),
    )
}

fn validate_live_mode(
    mode: Option<&OsStr>,
    destructive_present: bool,
    bearer_present: bool,
) -> Result<(), RobotLiveConfigurationError> {
    if destructive_present {
        return Err(RobotLiveConfigurationError::DestructiveModeForbidden);
    }
    if bearer_present {
        return Err(RobotLiveConfigurationError::BearerTokenForbidden);
    }
    if mode != Some(OsStr::new(ROBOT_READ_ONLY_MODE)) {
        return Err(RobotLiveConfigurationError::LiveModeRequired);
    }
    Ok(())
}

fn credential_file_path(name: &str) -> Result<PathBuf, RobotLiveConfigurationError> {
    let value = env::var_os(name).ok_or(RobotLiveConfigurationError::CredentialFileRequired)?;
    if value.is_empty() {
        return Err(RobotLiveConfigurationError::CredentialFileRequired);
    }
    Ok(PathBuf::from(value))
}

struct SecretFile {
    bytes: Vec<u8>,
    metadata: Metadata,
}

impl SecretFile {
    fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.bytes
    }
}

impl Drop for SecretFile {
    fn drop(&mut self) {
        sanitize_bytes(&mut self.bytes);
    }
}

fn read_secret_file(
    path: &Path,
    max_secret_bytes: usize,
) -> Result<SecretFile, RobotLiveConfigurationError> {
    let file = open_secret_file(path)?;
    let opened = file
        .metadata()
        .map_err(|_| RobotLiveConfigurationError::FileMetadataUnavailable)?;
    validate_metadata(&opened)?;

    let max_file_bytes = max_secret_bytes
        .checked_add(2)
        .ok_or(RobotLiveConfigurationError::FileTooLarge)?;
    if opened.len() > u64::try_from(max_file_bytes).unwrap_or(u64::MAX) {
        return Err(RobotLiveConfigurationError::FileTooLarge);
    }
    let read_limit = max_file_bytes
        .checked_add(1)
        .ok_or(RobotLiveConfigurationError::FileTooLarge)?;
    let mut secret = SecretFile {
        bytes: Vec::new(),
        metadata: opened,
    };
    secret
        .bytes
        .try_reserve_exact(read_limit)
        .map_err(|_| RobotLiveConfigurationError::FileReadFailed)?;
    file.take(u64::try_from(read_limit).unwrap_or(u64::MAX))
        .read_to_end(&mut secret.bytes)
        .map_err(|_| RobotLiveConfigurationError::FileReadFailed)?;
    if secret.bytes.len() > max_file_bytes {
        return Err(RobotLiveConfigurationError::FileTooLarge);
    }
    Ok(secret)
}

#[cfg(unix)]
fn open_secret_file(path: &Path) -> Result<File, RobotLiveConfigurationError> {
    use std::os::unix::fs::{MetadataExt, PermissionsExt};

    use rustix::fs::{Mode, OFlags, open, openat};
    use rustix::process::geteuid;

    let parent = path
        .parent()
        .ok_or(RobotLiveConfigurationError::ParentDirectoryUntrusted)?;
    let parent = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let file_name = path
        .file_name()
        .ok_or(RobotLiveConfigurationError::FileOpenFailed)?;
    let parent_descriptor = open(
        parent,
        OFlags::RDONLY | OFlags::DIRECTORY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|_| RobotLiveConfigurationError::ParentDirectoryUntrusted)?;
    let parent_directory = File::from(parent_descriptor);
    let parent_metadata = parent_directory
        .metadata()
        .map_err(|_| RobotLiveConfigurationError::ParentDirectoryUntrusted)?;
    let expected_owner = geteuid().as_raw();
    if !parent_metadata.is_dir()
        || parent_metadata.uid() != expected_owner
        || parent_metadata.permissions().mode() & 0o077 != 0
    {
        return Err(RobotLiveConfigurationError::ParentDirectoryUntrusted);
    }

    let descriptor = openat(
        &parent_directory,
        file_name,
        OFlags::RDONLY | OFlags::CLOEXEC | OFlags::NOFOLLOW,
        Mode::empty(),
    )
    .map_err(|_| RobotLiveConfigurationError::FileOpenFailed)?;
    let file = File::from(descriptor);
    let metadata = file
        .metadata()
        .map_err(|_| RobotLiveConfigurationError::FileMetadataUnavailable)?;
    if metadata.uid() != expected_owner {
        return Err(RobotLiveConfigurationError::FileOwnerMismatch);
    }
    Ok(file)
}

#[cfg(not(unix))]
fn open_secret_file(_path: &Path) -> Result<File, RobotLiveConfigurationError> {
    Err(RobotLiveConfigurationError::UnsupportedPlatform)
}

fn normalized_secret(bytes: &mut [u8]) -> Result<&mut [u8], RobotLiveConfigurationError> {
    let len = if bytes.ends_with(b"\r\n") {
        bytes.len().saturating_sub(2)
    } else if bytes.ends_with(b"\n") {
        bytes.len().saturating_sub(1)
    } else {
        bytes.len()
    };
    let secret = bytes
        .get_mut(..len)
        .ok_or(RobotLiveConfigurationError::CredentialRejected)?;
    if secret.is_empty() {
        return Err(RobotLiveConfigurationError::CredentialRejected);
    }
    Ok(secret)
}

fn validate_metadata(metadata: &Metadata) -> Result<(), RobotLiveConfigurationError> {
    if !metadata.is_file() {
        return Err(RobotLiveConfigurationError::FileNotRegular);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};

        if metadata.nlink() != 1 {
            return Err(RobotLiveConfigurationError::FileHasMultipleLinks);
        }
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(RobotLiveConfigurationError::FilePermissionsTooBroad);
        }
    }
    Ok(())
}

#[cfg(unix)]
fn same_secret_file(left: &SecretFile, right: &SecretFile) -> bool {
    use std::os::unix::fs::MetadataExt;

    left.metadata.dev() == right.metadata.dev() && left.metadata.ino() == right.metadata.ino()
}

#[cfg(not(unix))]
fn same_secret_file(_left: &SecretFile, _right: &SecretFile) -> bool {
    false
}

#[cfg(test)]
#[path = "robot_config/tests.rs"]
mod tests;
