#[cfg(unix)]
use std::fs::{self, DirBuilder, OpenOptions};
#[cfg(unix)]
use std::io::Write;
use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};

#[cfg(unix)]
use cloud_sdk_hetzner::{ROBOT_API_BASE_URL, official_robot_endpoint_policy};
#[cfg(unix)]
use cloud_sdk_reqwest::blocking::HttpsEndpoint;

#[cfg(unix)]
use super::same_secret_file;
use super::{RobotLiveConfigurationError, normalized_secret, read_secret_file, validate_live_mode};

#[cfg(unix)]
static TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[cfg(unix)]
struct TempDirectory(PathBuf);

#[cfg(unix)]
impl TempDirectory {
    fn new() -> std::io::Result<Self> {
        let sequence = TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "cloud-sdk-robot-live-smoke-{}-{sequence}",
            std::process::id()
        ));
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

#[cfg(unix)]
impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[cfg(unix)]
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
fn requires_exact_robot_mode_and_no_other_credential_or_destructive_flag() {
    let mode = Some(std::ffi::OsStr::new("robot-read-only"));
    assert_eq!(validate_live_mode(mode, false, false), Ok(()));
    assert_eq!(
        validate_live_mode(None, false, false),
        Err(RobotLiveConfigurationError::LiveModeRequired)
    );
    assert_eq!(
        validate_live_mode(mode, true, false),
        Err(RobotLiveConfigurationError::DestructiveModeForbidden)
    );
    assert_eq!(
        validate_live_mode(mode, false, true),
        Err(RobotLiveConfigurationError::BearerTokenForbidden)
    );
}

#[test]
fn strips_only_one_terminal_line_ending() {
    for (mut value, expected) in [
        (b"robot-user".to_vec(), b"robot-user".as_slice()),
        (b"robot-user\n".to_vec(), b"robot-user".as_slice()),
        (b"robot-user\r\n".to_vec(), b"robot-user".as_slice()),
    ] {
        let normalized = normalized_secret(&mut value);
        assert!(normalized.is_ok());
        assert_eq!(normalized.ok().as_deref(), Some(expected));
    }
    let mut empty = b"\n".to_vec();
    assert_eq!(
        normalized_secret(&mut empty),
        Err(RobotLiveConfigurationError::CredentialRejected)
    );
}

#[cfg(unix)]
#[test]
fn private_distinct_files_build_a_redacted_basic_credential() -> std::io::Result<()> {
    let directory = TempDirectory::new()?;
    let username_path = directory.path().join("username");
    let password_path = directory.path().join("password");
    write_private_file(&username_path, b"robot-user\n")?;
    write_private_file(&password_path, b"secret-password\n")?;

    let mut username = read_secret_file(&username_path, 256)
        .unwrap_or_else(|_| unreachable!("private username fixture failed"));
    let mut password = read_secret_file(&password_path, 2048)
        .unwrap_or_else(|_| unreachable!("private password fixture failed"));
    assert!(!same_secret_file(&username, &password));
    let username = normalized_secret(username.as_mut_slice())
        .unwrap_or_else(|_| unreachable!("username normalization failed"));
    let password = normalized_secret(password.as_mut_slice())
        .unwrap_or_else(|_| unreachable!("password normalization failed"));
    let policy = official_robot_endpoint_policy()
        .unwrap_or_else(|_| unreachable!("official Robot policy failed"));
    let endpoint = HttpsEndpoint::new_with_policy(ROBOT_API_BASE_URL, policy)
        .unwrap_or_else(|_| unreachable!("official Robot endpoint failed"));
    let scope = cloud_sdk_reqwest::blocking::BasicCredentialScope::new(
        cloud_sdk_hetzner::HETZNER_PROVIDER_ID,
        cloud_sdk_hetzner::ROBOT_SERVICE_ID,
        endpoint,
    );
    let credential =
        cloud_sdk_reqwest::blocking::BasicCredential::from_mut_bytes(username, password, scope);
    assert!(credential.is_ok());
    let diagnostic = format!("{:?}", credential.ok());
    assert!(diagnostic.contains("[redacted]"));
    assert!(!diagnostic.contains("robot-user"));
    assert!(!diagnostic.contains("secret-password"));
    Ok(())
}

#[cfg(not(unix))]
#[test]
fn robot_credential_loading_fails_closed() {
    assert!(matches!(
        read_secret_file(Path::new("unused"), 64),
        Err(RobotLiveConfigurationError::UnsupportedPlatform)
    ));
}

#[cfg(unix)]
#[test]
fn rejects_symlinks_multiple_links_and_broad_permissions() -> std::io::Result<()> {
    use std::os::unix::fs::{PermissionsExt, symlink};

    let directory = TempDirectory::new()?;
    let path = directory.path().join("credential");
    write_private_file(&path, b"secret\n")?;
    let link = directory.path().join("symlink");
    symlink(&path, &link)?;
    assert!(matches!(
        read_secret_file(&link, 64),
        Err(RobotLiveConfigurationError::FileOpenFailed)
    ));

    let real_parent = directory.path().join("real-parent");
    fs::create_dir(&real_parent)?;
    fs::set_permissions(&real_parent, fs::Permissions::from_mode(0o700))?;
    let nested_credential = real_parent.join("credential");
    write_private_file(&nested_credential, b"secret\n")?;
    let parent_link = directory.path().join("parent-link");
    symlink(&real_parent, &parent_link)?;
    assert!(matches!(
        read_secret_file(&parent_link.join("credential"), 64),
        Err(RobotLiveConfigurationError::ParentDirectoryUntrusted)
    ));

    let hardlink = directory.path().join("hardlink");
    fs::hard_link(&path, &hardlink)?;
    assert!(matches!(
        read_secret_file(&path, 64),
        Err(RobotLiveConfigurationError::FileHasMultipleLinks)
    ));
    fs::remove_file(&hardlink)?;

    fs::set_permissions(&path, fs::Permissions::from_mode(0o640))?;
    assert!(matches!(
        read_secret_file(&path, 64),
        Err(RobotLiveConfigurationError::FilePermissionsTooBroad)
    ));

    fs::set_permissions(directory.path(), fs::Permissions::from_mode(0o750))?;
    assert!(matches!(
        read_secret_file(&path, 64),
        Err(RobotLiveConfigurationError::ParentDirectoryUntrusted)
    ));
    Ok(())
}
