use alloc::string::String;
use alloc::vec::Vec;
use core::fmt;

use cloud_sdk_sanitization::sanitize_string;

use super::super::{Labels, ResponseModelError, UtcTimestamp, required};
use super::common::{MAX_CONSOLE_ITEMS, parse_model_labels};
use super::parse::{
    boolean, object_mut, positive, take_text, take_text_allow_empty, take_timestamp,
};
use crate::serde::strict_json::Value;

/// Access settings for one Storage Box subaccount.
#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxSubaccountAccessSettings {
    /// Whether access outside Hetzner's network is allowed.
    pub reachable_externally: bool,
    /// Whether Samba is enabled.
    pub samba_enabled: bool,
    /// Whether SSH is enabled.
    pub ssh_enabled: bool,
    /// Whether WebDAV is enabled.
    pub webdav_enabled: bool,
    /// Whether the account is read-only.
    pub readonly: bool,
}

/// One source-complete Storage Box subaccount.
///
/// Whole-model equality is intentionally unavailable because dynamic provider
/// text must not acquire a variable-time comparison API.
///
/// ```compile_fail
/// use cloud_sdk_hetzner::serde::StorageBoxSubaccount;
/// fn compare(left: &StorageBoxSubaccount, right: &StorageBoxSubaccount) -> bool { left == right }
/// ```
#[non_exhaustive]
pub struct StorageBoxSubaccount {
    storage_box: u64,
    id: u64,
    name: String,
    home_directory: String,
    access_settings: StorageBoxSubaccountAccessSettings,
    description: String,
    labels: Labels,
    username: String,
    server: String,
    created: UtcTimestamp,
}

impl StorageBoxSubaccount {
    /// Returns the parent Storage Box identifier.
    #[must_use]
    pub const fn storage_box(&self) -> u64 {
        self.storage_box
    }

    /// Returns the subaccount identifier.
    #[must_use]
    pub const fn id(&self) -> u64 {
        self.id
    }

    /// Returns the subaccount name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the relative home directory.
    #[must_use]
    pub fn home_directory(&self) -> &str {
        &self.home_directory
    }

    /// Returns access settings.
    #[must_use]
    pub const fn access_settings(&self) -> StorageBoxSubaccountAccessSettings {
        self.access_settings
    }

    /// Returns the description.
    #[must_use]
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns user-defined labels.
    #[must_use]
    pub const fn labels(&self) -> &Labels {
        &self.labels
    }

    /// Returns the login username.
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    /// Returns the service FQDN.
    #[must_use]
    pub fn server(&self) -> &str {
        &self.server
    }

    /// Returns the canonical creation timestamp.
    #[must_use]
    pub fn created(&self) -> &str {
        self.created.as_str()
    }
}

impl fmt::Debug for StorageBoxSubaccount {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("StorageBoxSubaccount")
            .field("storage_box", &"[redacted]")
            .field("id", &"[redacted]")
            .field("identity", &"[redacted]")
            .field("home_directory", &"[redacted]")
            .field("labels", &self.labels)
            .finish()
    }
}

impl Drop for StorageBoxSubaccount {
    fn drop(&mut self) {
        for value in [
            &mut self.name,
            &mut self.home_directory,
            &mut self.description,
            &mut self.username,
            &mut self.server,
        ] {
            sanitize_string(value);
        }
    }
}

pub(crate) fn parse_storage_box_subaccount(
    value: &mut Value,
) -> Result<StorageBoxSubaccount, ResponseModelError> {
    let fields = object_mut(value)?;
    let home_directory = take_text(fields, "home_directory", 999)?;
    if home_directory.as_str().starts_with('/')
        || !home_directory.as_str().bytes().all(valid_home_byte)
    {
        return Err(ResponseModelError::InvalidText);
    }
    let storage_box = positive(fields, "storage_box")?;
    let id = positive(fields, "id")?;
    let name = take_text(fields, "name", 50)?;
    let access_settings = parse_access(required(fields, "access_settings")?)?;
    let description = take_text_allow_empty(fields, "description", 1_000)?;
    let labels = parse_model_labels(required(fields, "labels")?)?;
    let username = take_text(fields, "username", 256)?;
    let server = take_text(fields, "server", 512)?;
    let created = take_timestamp(fields, "created")?;
    Ok(StorageBoxSubaccount {
        storage_box,
        id,
        name: name.into_inner(),
        home_directory: home_directory.into_inner(),
        access_settings,
        description: description.into_inner(),
        labels,
        username: username.into_inner(),
        server: server.into_inner(),
        created,
    })
}

pub(crate) fn parse_storage_box_subaccounts(
    value: &mut Value,
) -> Result<Vec<StorageBoxSubaccount>, ResponseModelError> {
    let values = value.as_array_mut().ok_or(ResponseModelError::WrongType)?;
    if values.len() > MAX_CONSOLE_ITEMS {
        return Err(ResponseModelError::TooManyItems);
    }
    let mut subaccounts = Vec::new();
    subaccounts
        .try_reserve_exact(values.len())
        .map_err(|_| ResponseModelError::Allocation)?;
    for value in values {
        subaccounts.push(parse_storage_box_subaccount(value)?);
    }
    Ok(subaccounts)
}

fn parse_access(value: &Value) -> Result<StorageBoxSubaccountAccessSettings, ResponseModelError> {
    let fields = value.as_object().ok_or(ResponseModelError::WrongType)?;
    Ok(StorageBoxSubaccountAccessSettings {
        reachable_externally: boolean(fields, "reachable_externally")?,
        samba_enabled: boolean(fields, "samba_enabled")?,
        ssh_enabled: boolean(fields, "ssh_enabled")?,
        webdav_enabled: boolean(fields, "webdav_enabled")?,
        readonly: boolean(fields, "readonly")?,
    })
}

fn valid_home_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b' ' | b'.' | b'/' | b'_' | b'-')
}
