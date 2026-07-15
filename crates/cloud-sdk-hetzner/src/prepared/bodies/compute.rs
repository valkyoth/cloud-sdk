//! Server, image, placement-group, and volume JSON bodies.

use crate::cloud::images::{ImageProtectionRequest, ImageUpdateRequest};
use crate::cloud::servers::actions::{
    DnsPtrIntent, RescueType, ServerActionKind as Kind, ServerActionRequest, ServerImageType,
};
use crate::cloud::servers::placement_groups::{
    PlacementGroupCreateRequest, PlacementGroupType, PlacementGroupUpdateRequest,
};
use crate::cloud::servers::{
    PrimaryIpSelection, ServerCreateRequest, ServerPublicNet, ServerUpdateRequest,
};
use crate::cloud::volumes::{
    VolumeAttachRequest, VolumeCreatePlacement, VolumeCreateRequest, VolumeProtectionRequest,
    VolumeResizeRequest, VolumeUpdateRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(ServerCreateRequest<'_>, request => request.endpoint(), "create_server", write_server_create);
body_wire!(ServerUpdateRequest<'_>, request => request.endpoint(), "update_server", write_server_update);
body_wire!(ImageUpdateRequest<'_>, request => request.endpoint(), "update_image", write_image_update);
body_component!(
    ImageProtectionRequest,
    "change_image_protection",
    write_image_protection
);
body_wire!(PlacementGroupCreateRequest<'_>, request => request.endpoint(), "create_placement_group", write_placement_create);
body_wire!(PlacementGroupUpdateRequest<'_>, request => request.endpoint(), "update_placement_group", write_placement_update);
body_wire!(VolumeCreateRequest<'_>, request => request.endpoint(), "create_volume", write_volume_create);
body_wire!(VolumeUpdateRequest<'_>, request => request.endpoint(), "update_volume", write_volume_update);
body_component!(VolumeAttachRequest, "attach_volume", write_volume_attach);
body_component!(
    VolumeProtectionRequest,
    "change_volume_protection",
    write_volume_protection
);
body_component!(VolumeResizeRequest, "resize_volume", write_volume_resize);

impl crate::prepared::BodyWire for ServerActionRequest<'_> {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        write_server_action(self, output)
    }

    fn operation_key(self) -> &'static str {
        match self {
            Self::AddToPlacementGroup { .. } => "add_server_to_placement_group",
            Self::AttachIso { .. } => "attach_server_iso",
            Self::AttachToNetwork { .. } => "attach_server_to_network",
            Self::ChangeAliasIps { .. } => "change_server_alias_ips",
            Self::ChangeDnsPtr { .. } => "change_server_dns_ptr",
            Self::ChangeProtection { .. } => "change_server_protection",
            Self::ChangeType { .. } => "change_server_type",
            Self::CreateImage { .. } => "create_server_image",
            Self::DetachFromNetwork { .. } => "detach_server_from_network",
            Self::EnableRescue { .. } => "enable_server_rescue",
            Self::Rebuild { .. } => "rebuild_server",
            Self::Empty(Kind::DetachIso) => "detach_server_iso",
            Self::Empty(Kind::DisableBackup) => "disable_server_backup",
            Self::Empty(Kind::DisableRescue) => "disable_server_rescue",
            Self::Empty(Kind::EnableBackup) => "enable_server_backup",
            Self::Empty(Kind::Poweroff) => "poweroff_server",
            Self::Empty(Kind::Poweron) => "poweron_server",
            Self::Empty(Kind::Reboot) => "reboot_server",
            Self::Empty(Kind::RemoveFromPlacementGroup) => "remove_server_from_placement_group",
            Self::Empty(Kind::RequestConsole) => "request_server_console",
            Self::Empty(Kind::Reset) => "reset_server",
            Self::Empty(Kind::ResetPassword) => "reset_server_password",
            Self::Empty(Kind::Shutdown) => "shutdown_server",
            Self::Empty(_) => "",
        }
    }
}

fn write_server_create(
    request: ServerCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(first, "image", request.image.as_str())?;
        if let Some(location) = request.location {
            writer.field_string(first, "location", location.as_str())?;
        }
        writer.field_string(first, "name", request.name.as_str())?;
        if let Some(public_net) = request.public_net {
            write_public_net(writer, first, public_net)?;
        }
        writer.field_string(first, "server_type", request.server_type.as_str())?;
        if let Some(user_data) = request.user_data {
            writer.field_sensitive(first, "user_data", |output| {
                user_data.write_json_string(output)
            })?;
        }
        Ok(())
    })
}

fn write_public_net(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    public_net: ServerPublicNet,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "public_net")?;
    writer.begin_object()?;
    let mut nested = true;
    writer.field_bool(&mut nested, "enable_ipv4", public_net.enable_ipv4)?;
    writer.field_bool(&mut nested, "enable_ipv6", public_net.enable_ipv6)?;
    write_primary_ip(writer, &mut nested, "ipv4", public_net.ipv4)?;
    write_primary_ip(writer, &mut nested, "ipv6", public_net.ipv6)?;
    writer.end_object()
}

fn write_primary_ip(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    name: &str,
    value: PrimaryIpSelection,
) -> Result<(), HetznerPreparationError> {
    match value {
        PrimaryIpSelection::Auto => Ok(()),
        PrimaryIpSelection::Id(id) => writer.field_u64(first, name, id.get()),
        PrimaryIpSelection::Null => writer.field_null(first, name),
    }
}

fn write_server_update(
    request: ServerUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(name) = request.name {
            writer.field_string(first, "name", name.as_str())?;
        }
        Ok(())
    })
}

fn write_server_action(
    request: ServerActionRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| match request {
        ServerActionRequest::AddToPlacementGroup { placement_group } => {
            writer.field_u64(first, "placement_group", placement_group.get())
        }
        ServerActionRequest::AttachIso { iso } => writer.field_string(first, "iso", iso.as_str()),
        ServerActionRequest::AttachToNetwork { network, ip } => {
            if let Some(ip) = ip {
                writer.field_string(first, "ip", ip.as_str())?;
            }
            writer.field_u64(first, "network", network.get())
        }
        ServerActionRequest::ChangeAliasIps { network, alias_ips } => {
            writer.field(first, "alias_ips")?;
            writer.begin_array()?;
            let mut item = true;
            for ip in alias_ips {
                writer.value(&mut item)?;
                writer.string(ip.as_str())?;
            }
            writer.end_array()?;
            writer.field_u64(first, "network", network.get())
        }
        ServerActionRequest::ChangeDnsPtr { ip, dns_ptr } => {
            match dns_ptr {
                DnsPtrIntent::Set(value) => {
                    writer.field_string(first, "dns_ptr", value.as_str())?;
                }
                DnsPtrIntent::Reset => writer.field_null(first, "dns_ptr")?,
            }
            writer.field_string(first, "ip", ip.as_str())
        }
        ServerActionRequest::ChangeProtection { delete, rebuild } => {
            writer.field_bool(first, "delete", delete)?;
            writer.field_bool(first, "rebuild", rebuild)
        }
        ServerActionRequest::ChangeType {
            server_type,
            upgrade_disk,
        } => {
            writer.field_string(first, "server_type", server_type.as_str())?;
            writer.field_bool(first, "upgrade_disk", upgrade_disk)
        }
        ServerActionRequest::CreateImage {
            description,
            image_type,
        } => {
            if let Some(description) = description {
                writer.field_string(first, "description", description.as_str())?;
            }
            writer.field_string(first, "type", image_type_value(image_type))
        }
        ServerActionRequest::DetachFromNetwork { network } => {
            writer.field_u64(first, "network", network.get())
        }
        ServerActionRequest::EnableRescue {
            rescue_type,
            ssh_keys,
        } => {
            writer.field(first, "ssh_keys")?;
            writer.begin_array()?;
            let mut item = true;
            for key in ssh_keys {
                writer.value(&mut item)?;
                writer.u64(key.get())?;
            }
            writer.end_array()?;
            writer.field_string(first, "type", rescue_type_value(rescue_type))
        }
        ServerActionRequest::Rebuild { image, user_data } => {
            writer.field_string(first, "image", image.as_str())?;
            if let Some(user_data) = user_data {
                writer.field_sensitive(first, "user_data", |output| {
                    user_data.write_json_string(output)
                })?;
            }
            Ok(())
        }
        ServerActionRequest::Empty(_) => Err(HetznerPreparationError::UnexpectedBody),
    })
}

fn write_image_update(
    request: ImageUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    let (description, labels) = request.prepared_parts();
    write_optional_text_labels(
        output,
        "description",
        description.map(|value| value.as_str()),
        labels,
    )
}

fn write_image_protection(
    request: ImageProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_placement_create(
    request: PlacementGroupCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let (name, labels) = request.prepared_parts();
        if let Some(labels) = labels {
            writer.field_labels(first, "labels", labels)?;
        }
        writer.field_string(first, "name", name.as_str())?;
        writer.field_string(
            first,
            "type",
            placement_type(request.placement_group_type()),
        )
    })
}

fn write_placement_update(
    request: PlacementGroupUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    let (name, labels) = request.prepared_parts();
    write_optional_text_labels(output, "name", name.map(|value| value.as_str()), labels)
}

fn write_volume_create(
    request: VolumeCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(format) = request.format {
            writer.field_string(first, "format", format.as_str())?;
        }
        if let Some(labels) = request.labels {
            writer.field_labels(first, "labels", labels)?;
        }
        match request.placement() {
            VolumeCreatePlacement::Server { server, automount } => {
                writer.field_bool(first, "automount", automount)?;
                writer.field_u64(first, "server", server.get())?;
            }
            VolumeCreatePlacement::Location(location) => {
                writer.field_string(first, "location", location.as_str())?;
            }
        }
        writer.field_string(first, "name", request.name.as_str())?;
        writer.field_u64(first, "size", u64::from(request.size().get()))
    })
}

fn write_volume_update(
    request: VolumeUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_optional_text_labels(
        output,
        "name",
        request.name.map(|value| value.as_str()),
        request.labels,
    )
}

fn write_volume_attach(
    request: VolumeAttachRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "automount", request.automount())?;
        writer.field_u64(first, "server", request.server().get())
    })
}

fn write_volume_protection(
    request: VolumeProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_volume_resize(
    request: VolumeResizeRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "size", u64::from(request.size().get()))
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

fn write_optional_text_labels(
    output: &mut [u8],
    field_name: &str,
    value: Option<&str>,
    labels: Option<crate::cloud::shared::CloudLabels<'_>>,
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = labels {
            writer.field_labels(first, "labels", labels)?;
        }
        if let Some(value) = value {
            writer.field_string(first, field_name, value)?;
        }
        Ok(())
    })
}

const fn placement_type(value: PlacementGroupType) -> &'static str {
    match value {
        PlacementGroupType::Spread => "spread",
    }
}

const fn image_type_value(value: ServerImageType) -> &'static str {
    match value {
        ServerImageType::Snapshot => "snapshot",
        ServerImageType::Backup => "backup",
    }
}

const fn rescue_type_value(value: RescueType) -> &'static str {
    match value {
        RescueType::Linux64 => "linux64",
    }
}
