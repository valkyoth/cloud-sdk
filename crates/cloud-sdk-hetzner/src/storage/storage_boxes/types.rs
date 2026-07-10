//! Storage Box request marker types.

use core::fmt;

use cloud_sdk::buffer;

use crate::cloud::shared::{CloudLabels, CloudName, CloudRequestError, CloudResourceId, CloudText};

/// Storage Box request error.
pub type StorageBoxRequestError = CloudRequestError;
/// Storage Box identifier.
pub type StorageBoxId = CloudResourceId;
/// Storage Box type identifier.
pub type StorageBoxTypeId = CloudResourceId;
/// Storage Box snapshot identifier.
pub type StorageBoxSnapshotId = CloudResourceId;
/// Storage Box subaccount identifier.
pub type StorageBoxSubaccountId = CloudResourceId;
/// Storage Box request name.
pub type StorageBoxName<'a> = CloudName<'a>;
/// Storage Box type name.
pub type StorageBoxTypeName<'a> = CloudName<'a>;
/// Storage Box type ID-or-name request marker.
pub type StorageBoxTypeRef<'a> = CloudName<'a>;
/// Storage Box location name.
pub type StorageBoxLocation<'a> = CloudName<'a>;
/// Storage Box snapshot name.
pub type StorageBoxSnapshotName<'a> = CloudName<'a>;
/// Storage Box snapshot ID-or-name request marker.
pub type StorageBoxSnapshotRef<'a> = CloudName<'a>;
/// Storage Box subaccount display name.
pub type StorageBoxSubaccountName<'a> = CloudName<'a>;
/// Storage Box subaccount username filter.
pub type StorageBoxSubaccountUsername<'a> = CloudName<'a>;
/// Storage Box bounded text value.
pub type StorageBoxText<'a> = CloudText<'a>;
/// Storage Box description value.
pub type StorageBoxDescription<'a> = CloudText<'a>;
/// Storage Box snapshot description.
pub type StorageBoxSnapshotDescription<'a> = CloudText<'a>;
/// Storage Box subaccount description.
pub type StorageBoxSubaccountDescription<'a> = CloudText<'a>;
/// Storage Box SSH public key marker.
pub type StorageBoxSshKey<'a> = CloudText<'a>;
/// Storage Box label map marker.
pub type StorageBoxLabels<'a> = CloudLabels<'a>;

/// Storage Box list sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
    /// Sort by raw size.
    StatsSize,
    /// Sort by filesystem size.
    StatsSizeFilesystem,
}

/// Storage Box type list sort marker. The source API exposes no sort parameter.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxTypeSortField {}

/// Storage Box action list sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxActionSortField {
    /// Sort by ID.
    Id,
    /// Sort by command.
    Command,
    /// Sort by status.
    Status,
    /// Sort by started timestamp.
    Started,
    /// Sort by finished timestamp.
    Finished,
}

/// Storage Box snapshot list sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxSnapshotSortField {
    /// Sort by ID.
    Id,
    /// Sort by name.
    Name,
    /// Sort by creation timestamp.
    Created,
}

/// Storage Box subaccount list sort fields admitted by the source-locked API.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StorageBoxSubaccountSortField {
    /// Sort by ID.
    Id,
    /// Sort by creation timestamp.
    Created,
}

/// Redacted Storage Box password value.
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct StorageBoxPassword<'a> {
    value: &'a str,
}

impl<'a> StorageBoxPassword<'a> {
    /// Creates a bounded JSON-string password marker.
    pub fn new(value: &'a str) -> Result<Self, StorageBoxRequestError> {
        if value.is_empty() || value.len() > 1024 || value.bytes().any(|byte| byte < 0x20) {
            return Err(StorageBoxRequestError::InvalidText);
        }
        Ok(Self { value })
    }

    /// Writes this password as a JSON string into a caller-owned buffer.
    ///
    /// # Security
    ///
    /// `output` contains the plaintext password after this call succeeds.
    /// Callers must overwrite the written bytes, for example with
    /// `output[..written].fill(0)`, once the request body has been sent. If the
    /// buffer is too small, the writer returns before modifying it.
    pub fn write_json_string(self, output: &mut [u8]) -> Result<usize, StorageBoxRequestError> {
        let mut len = 0;
        buffer::write_json_string(
            output,
            &mut len,
            self.value,
            StorageBoxRequestError::QueryBufferTooSmall,
        )?;
        Ok(len)
    }
}

impl fmt::Debug for StorageBoxPassword<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("StorageBoxPassword([redacted])")
    }
}

/// Storage Box subaccount home directory.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct StorageBoxHomeDirectory<'a> {
    value: &'a str,
}

impl<'a> StorageBoxHomeDirectory<'a> {
    /// Creates a bounded, relative home directory marker.
    pub fn new(value: &'a str) -> Result<Self, StorageBoxRequestError> {
        if value.is_empty()
            || value.len() > 999
            || value.starts_with('/')
            || value
                .split('/')
                .any(|segment| matches!(segment.trim(), "." | ".."))
            || !value.bytes().all(is_home_directory_byte)
        {
            return Err(StorageBoxRequestError::InvalidText);
        }
        Ok(Self { value })
    }

    /// Returns the directory value.
    #[must_use]
    pub const fn as_str(self) -> &'a str {
        self.value
    }
}

macro_rules! bounded_u8 {
    ($name:ident, $min:expr, $max:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(u8);

        impl $name {
            /// Creates a bounded value.
            pub const fn new(value: u8) -> Option<Self> {
                if value > $max || ($min > 0 && value < $min) {
                    return None;
                }
                Some(Self(value))
            }

            /// Returns the raw value.
            #[must_use]
            pub const fn get(self) -> u8 {
                self.0
            }
        }
    };
}

macro_rules! bounded_u8_max {
    ($name:ident, $max:expr, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct $name(u8);

        impl $name {
            /// Creates a bounded value.
            pub const fn new(value: u8) -> Option<Self> {
                if value > $max {
                    return None;
                }
                Some(Self(value))
            }

            /// Returns the raw value.
            #[must_use]
            pub const fn get(self) -> u8 {
                self.0
            }
        }
    };
}

bounded_u8_max!(SnapshotPlanMinute, 59, "Snapshot plan minute.");
bounded_u8_max!(SnapshotPlanHour, 23, "Snapshot plan hour.");
bounded_u8!(SnapshotPlanDayOfWeek, 1, 7, "Snapshot plan day of week.");
bounded_u8!(SnapshotPlanDayOfMonth, 1, 31, "Snapshot plan day of month.");

/// Snapshot plan maximum retained snapshots.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshotPlanMaxSnapshots(u16);

impl SnapshotPlanMaxSnapshots {
    /// Creates a nonzero snapshot retention count.
    pub const fn new(value: u16) -> Option<Self> {
        if value == 0 {
            return None;
        }
        Some(Self(value))
    }

    /// Returns the raw value.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

const fn is_home_directory_byte(byte: u8) -> bool {
    matches!(byte, b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b' ' | b'.' | b'/' | b'_' | b'-')
}
