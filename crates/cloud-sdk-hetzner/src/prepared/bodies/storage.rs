//! Console Storage Box JSON bodies.

use crate::prepared::{HetznerPreparationError, JsonWriter};
use crate::storage::storage_boxes::{
    StorageBoxAccessSettingsRequest, StorageBoxChangeHomeDirectoryRequest,
    StorageBoxChangeTypeRequest, StorageBoxCreateRequest, StorageBoxProtectionRequest,
    StorageBoxResetPasswordRequest, StorageBoxRollbackSnapshotRequest,
    StorageBoxSnapshotCreateRequest, StorageBoxSnapshotPlanRequest,
    StorageBoxSnapshotUpdateRequest, StorageBoxSubaccountAccessSettingsRequest,
    StorageBoxSubaccountCreateRequest, StorageBoxSubaccountUpdateRequest, StorageBoxUpdateRequest,
};

body_wire!(StorageBoxCreateRequest<'_>, request => request.endpoint(), "create_storage_box", write_create);
body_wire!(StorageBoxUpdateRequest<'_>, request => request.endpoint(), "update_storage_box", write_update);
body_wire!(StorageBoxSnapshotCreateRequest<'_>, request => request.endpoint(), "create_storage_box_snapshot", write_snapshot_create);
body_wire!(StorageBoxSnapshotUpdateRequest<'_>, request => request.endpoint(), "update_storage_box_snapshot", write_snapshot_update);
body_wire!(StorageBoxSubaccountCreateRequest<'_>, request => request.endpoint(), "create_storage_box_subaccount", write_subaccount_create);
body_wire!(StorageBoxSubaccountUpdateRequest<'_>, request => request.endpoint(), "update_storage_box_subaccount", write_subaccount_update);

body_component!(
    StorageBoxProtectionRequest,
    "change_storage_box_protection",
    write_protection
);
body_component!(
    StorageBoxChangeTypeRequest<'_>,
    "change_storage_box_type",
    write_change_type
);
body_component!(
    StorageBoxRollbackSnapshotRequest<'_>,
    "rollback_storage_box_snapshot",
    write_rollback
);
body_component!(
    StorageBoxSnapshotPlanRequest,
    "enable_storage_box_snapshot_plan",
    write_snapshot_plan
);
body_component!(
    StorageBoxAccessSettingsRequest,
    "update_storage_box_access_settings",
    write_access_settings_body
);
body_component!(
    StorageBoxChangeHomeDirectoryRequest<'_>,
    "change_storage_box_subaccount_home_directory",
    write_home_directory
);
body_component!(
    StorageBoxSubaccountAccessSettingsRequest,
    "update_storage_box_subaccount_access_settings",
    write_subaccount_access_body
);

impl crate::prepared::BodyWire for StorageBoxResetPasswordRequest<'_> {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        write_password(self, output)
    }

    fn operation_key(self) -> &'static str {
        "reset_storage_box_password"
    }

    fn accepts_operation(self, operation_key: &str) -> bool {
        matches!(
            operation_key,
            "reset_storage_box_password" | "reset_storage_box_subaccount_password"
        )
    }
}

fn write_create(
    request: StorageBoxCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(settings) = request.access_settings {
            writer.field(first, "access_settings")?;
            write_access_settings(writer, settings)?;
        }
        if let Some(labels) = request.labels {
            writer.field_labels(first, "labels", labels)?;
        }
        writer.field_string(first, "location", request.location.as_str())?;
        writer.field_string(first, "name", request.name.as_str())?;
        writer.field_sensitive(first, "password", |output| {
            request.password.write_json_string(output)
        })?;
        if let Some(keys) = request.ssh_keys {
            writer.field(first, "ssh_keys")?;
            writer.begin_array()?;
            let mut item = true;
            for key in keys {
                writer.value(&mut item)?;
                writer.string(key.as_str())?;
            }
            writer.end_array()?;
        }
        writer.field_string(first, "storage_box_type", request.storage_box_type.as_str())
    })
}

fn write_update(
    request: StorageBoxUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_optional_resource(
        output,
        None,
        request.labels,
        request.name.map(|v| v.as_str()),
    )
}

fn write_snapshot_create(
    request: StorageBoxSnapshotCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_optional_resource(
        output,
        request.description.map(|v| v.as_str()),
        request.labels,
        None,
    )
}

fn write_snapshot_update(
    request: StorageBoxSnapshotUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_optional_resource(
        output,
        request.description.map(|v| v.as_str()),
        request.labels,
        None,
    )
}

fn write_subaccount_create(
    request: StorageBoxSubaccountCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(settings) = request.access_settings {
            writer.field(first, "access_settings")?;
            write_subaccount_access(writer, settings)?;
        }
        if let Some(value) = request.description {
            writer.field_string(first, "description", value.as_str())?;
        }
        writer.field_string(first, "home_directory", request.home_directory.as_str())?;
        if let Some(value) = request.labels {
            writer.field_labels(first, "labels", value)?;
        }
        if let Some(value) = request.name {
            writer.field_string(first, "name", value.as_str())?;
        }
        writer.field_sensitive(first, "password", |output| {
            request.password.write_json_string(output)
        })
    })
}

fn write_subaccount_update(
    request: StorageBoxSubaccountUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_optional_resource(
        output,
        request.description.map(|v| v.as_str()),
        request.labels,
        request.name.map(|v| v.as_str()),
    )
}

fn write_protection(
    request: StorageBoxProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_change_type(
    request: StorageBoxChangeTypeRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(
            first,
            "storage_box_type",
            request.storage_box_type().as_str(),
        )
    })
}

fn write_password(
    request: StorageBoxResetPasswordRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_sensitive(first, "password", |output| {
            request.password().write_json_string(output)
        })
    })
}

fn write_rollback(
    request: StorageBoxRollbackSnapshotRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(first, "snapshot", request.snapshot().as_str())
    })
}

fn write_snapshot_plan(
    request: StorageBoxSnapshotPlanRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        match request.day_of_month {
            Some(v) => writer.field_u64(first, "day_of_month", u64::from(v.get()))?,
            None => writer.field_null(first, "day_of_month")?,
        }
        match request.day_of_week {
            Some(v) => writer.field_u64(first, "day_of_week", u64::from(v.get()))?,
            None => writer.field_null(first, "day_of_week")?,
        }
        writer.field_u64(first, "hour", u64::from(request.hour.get()))?;
        writer.field_u64(
            first,
            "max_snapshots",
            u64::from(request.max_snapshots.get()),
        )?;
        writer.field_u64(first, "minute", u64::from(request.minute.get()))
    })
}

fn write_access_settings_body(
    request: StorageBoxAccessSettingsRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_access_settings_fields(writer, request)
    })
}

fn write_subaccount_access_body(
    request: StorageBoxSubaccountAccessSettingsRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_subaccount_access_fields(writer, request)
    })
}

fn write_home_directory(
    request: StorageBoxChangeHomeDirectoryRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(first, "home_directory", request.home_directory().as_str())
    })
}

fn write_access_settings(
    writer: &mut JsonWriter<'_>,
    request: StorageBoxAccessSettingsRequest,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    write_access_settings_fields(writer, request)?;
    writer.end_object()
}

fn write_access_settings_fields(
    writer: &mut JsonWriter<'_>,
    request: StorageBoxAccessSettingsRequest,
) -> Result<(), HetznerPreparationError> {
    let mut first = true;
    if let Some(v) = request.reachable_externally {
        writer.field_bool(&mut first, "reachable_externally", v)?;
    }
    if let Some(v) = request.samba_enabled {
        writer.field_bool(&mut first, "samba_enabled", v)?;
    }
    if let Some(v) = request.ssh_enabled {
        writer.field_bool(&mut first, "ssh_enabled", v)?;
    }
    if let Some(v) = request.webdav_enabled {
        writer.field_bool(&mut first, "webdav_enabled", v)?;
    }
    if let Some(v) = request.zfs_enabled {
        writer.field_bool(&mut first, "zfs_enabled", v)?;
    }
    Ok(())
}

fn write_subaccount_access(
    writer: &mut JsonWriter<'_>,
    request: StorageBoxSubaccountAccessSettingsRequest,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    write_subaccount_access_fields(writer, request)?;
    writer.end_object()
}

fn write_subaccount_access_fields(
    writer: &mut JsonWriter<'_>,
    request: StorageBoxSubaccountAccessSettingsRequest,
) -> Result<(), HetznerPreparationError> {
    let mut first = true;
    if let Some(v) = request.reachable_externally {
        writer.field_bool(&mut first, "reachable_externally", v)?;
    }
    if let Some(v) = request.readonly {
        writer.field_bool(&mut first, "readonly", v)?;
    }
    if let Some(v) = request.samba_enabled {
        writer.field_bool(&mut first, "samba_enabled", v)?;
    }
    if let Some(v) = request.ssh_enabled {
        writer.field_bool(&mut first, "ssh_enabled", v)?;
    }
    if let Some(v) = request.webdav_enabled {
        writer.field_bool(&mut first, "webdav_enabled", v)?;
    }
    Ok(())
}

fn write_optional_resource(
    output: &mut [u8],
    description: Option<&str>,
    labels: Option<crate::cloud::shared::CloudLabels<'_>>,
    name: Option<&str>,
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(v) = description {
            writer.field_string(first, "description", v)?;
        }
        if let Some(v) = labels {
            writer.field_labels(first, "labels", v)?;
        }
        if let Some(v) = name {
            writer.field_string(first, "name", v)?;
        }
        Ok(())
    })
}

fn object<F>(output: &mut [u8], write: F) -> Result<usize, HetznerPreparationError>
where
    F: FnOnce(&mut JsonWriter<'_>, &mut bool) -> Result<(), HetznerPreparationError>,
{
    let mut writer = JsonWriter::new(output);
    writer.begin_object()?;
    let mut first = true;
    write(&mut writer, &mut first)?;
    writer.end_object()?;
    Ok(writer.len())
}
