//! DNS Zone JSON bodies.

use crate::dns::zones::{
    PrimaryNameserver, PrimaryNameservers, TsigCredentials, ZoneCreateMode, ZoneCreateRequest,
    ZoneFileImportRequest, ZonePrimaryNameserversRequest, ZoneProtectionRequest, ZoneTtlRequest,
    ZoneUpdateRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(ZoneCreateRequest<'_>, request => request.endpoint(), "create_zone", write_create, zone_create_sensitivity);
body_wire!(ZoneUpdateRequest<'_>, request => request.endpoint(), "update_zone", write_update, public);
body_wire!(ZonePrimaryNameserversRequest<'_>, request => request.endpoint(), "change_zone_primary_nameservers", write_nameservers_request, nameservers_sensitivity);
body_wire!(ZoneProtectionRequest<'_>, request => request.endpoint(), "change_zone_protection", write_protection, public);
body_wire!(ZoneTtlRequest<'_>, request => request.endpoint(), "change_zone_ttl", write_ttl, public);
body_wire!(ZoneFileImportRequest<'_>, request => request.endpoint(), "import_zone_zonefile", write_zonefile_import, sensitive_body);

fn zone_create_sensitivity(
    request: ZoneCreateRequest<'_>,
) -> cloud_sdk::operation::RequestBodySensitivity {
    if request.zonefile().is_some() {
        return cloud_sdk::operation::RequestBodySensitivity::Sensitive;
    }
    match request.mode() {
        ZoneCreateMode::Primary => cloud_sdk::operation::RequestBodySensitivity::Public,
        ZoneCreateMode::Secondary(nameservers) => nameserver_entries_sensitivity(nameservers),
    }
}

fn nameservers_sensitivity(
    request: ZonePrimaryNameserversRequest<'_>,
) -> cloud_sdk::operation::RequestBodySensitivity {
    nameserver_entries_sensitivity(request.nameservers())
}

fn nameserver_entries_sensitivity(
    nameservers: PrimaryNameservers<'_>,
) -> cloud_sdk::operation::RequestBodySensitivity {
    if nameservers
        .entries()
        .iter()
        .any(|nameserver| nameserver.tsig().is_some())
    {
        cloud_sdk::operation::RequestBodySensitivity::Sensitive
    } else {
        cloud_sdk::operation::RequestBodySensitivity::Public
    }
}

fn sensitive_body<T>(_: T) -> cloud_sdk::operation::RequestBodySensitivity {
    cloud_sdk::operation::RequestBodySensitivity::Sensitive
}

fn write_create(
    request: ZoneCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        writer.field_string(first, "mode", request.mode().mode().as_api_str())?;
        writer.field_string(first, "name", request.name().as_str())?;
        if let ZoneCreateMode::Secondary(nameservers) = request.mode() {
            write_nameservers_field(writer, first, nameservers)?;
        }
        if let Some(ttl) = request.ttl() {
            writer.field_u64(first, "ttl", u64::from(ttl.get()))?;
        }
        if let Some(zonefile) = request.zonefile() {
            writer.field_sensitive(first, "zonefile", zonefile)?;
        }
        Ok(())
    })
}

fn write_update(
    request: ZoneUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        Ok(())
    })
}

fn write_nameservers_request(
    request: ZonePrimaryNameserversRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_nameservers_field(writer, first, request.nameservers())
    })
}

fn write_protection(
    request: ZoneProtectionRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_ttl(
    request: ZoneTtlRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "ttl", u64::from(request.ttl().get()))
    })
}

fn write_zonefile_import(
    request: ZoneFileImportRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_sensitive(first, "zonefile", request.zonefile())
    })
}

fn object<F>(output: &mut [u8], write: F) -> Result<usize, HetznerPreparationError>
where
    F: Copy + Fn(&mut JsonWriter<'_, '_>, &mut bool) -> Result<(), HetznerPreparationError>,
{
    crate::prepared::encode_object(output, write)
}

fn write_nameservers_field(
    writer: &mut JsonWriter<'_, '_>,
    first: &mut bool,
    nameservers: PrimaryNameservers<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "primary_nameservers")?;
    writer.begin_array()?;
    let mut first_nameserver = true;
    for nameserver in nameservers.entries() {
        writer.value(&mut first_nameserver)?;
        write_nameserver(writer, *nameserver)?;
    }
    writer.end_array()
}

fn write_nameserver(
    writer: &mut JsonWriter<'_, '_>,
    nameserver: PrimaryNameserver<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    let mut first = true;
    writer.field_string(&mut first, "address", nameserver.as_str())?;
    writer.field_u64(&mut first, "port", u64::from(nameserver.port()))?;
    if let Some(tsig) = nameserver.tsig() {
        write_tsig(writer, &mut first, tsig)?;
    }
    writer.end_object()
}

fn write_tsig(
    writer: &mut JsonWriter<'_, '_>,
    first: &mut bool,
    tsig: TsigCredentials<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field_string(first, "tsig_algorithm", tsig.algorithm().as_api_str())?;
    writer.field_sensitive(first, "tsig_key", tsig.key())
}
