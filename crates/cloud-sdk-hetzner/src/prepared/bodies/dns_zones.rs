//! DNS Zone JSON bodies.

use crate::dns::zones::{
    PrimaryNameserver, PrimaryNameservers, TsigCredentials, ZoneCreateMode, ZoneCreateRequest,
    ZoneFileImportRequest, ZonePrimaryNameserversRequest, ZoneProtectionRequest, ZoneTtlRequest,
    ZoneUpdateRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(ZoneCreateRequest<'_>, request => request.endpoint(), "create_zone", write_create);
body_wire!(ZoneUpdateRequest<'_>, request => request.endpoint(), "update_zone", write_update);
body_wire!(ZonePrimaryNameserversRequest<'_>, request => request.endpoint(), "change_zone_primary_nameservers", write_nameservers_request);
body_wire!(ZoneProtectionRequest<'_>, request => request.endpoint(), "change_zone_protection", write_protection);
body_wire!(ZoneTtlRequest<'_>, request => request.endpoint(), "change_zone_ttl", write_ttl);
body_wire!(ZoneFileImportRequest<'_>, request => request.endpoint(), "import_zone_zonefile", write_zonefile_import);

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
            writer.field_sensitive(first, "zonefile", |output| {
                zonefile.write_json_string(output)
            })?;
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
        writer.field_sensitive(first, "zonefile", |output| {
            request.zonefile().write_json_string(output)
        })
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

fn write_nameservers_field(
    writer: &mut JsonWriter<'_>,
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
    writer: &mut JsonWriter<'_>,
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
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    tsig: TsigCredentials<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field_string(first, "tsig_algorithm", tsig.algorithm().as_api_str())?;
    writer.field_sensitive(first, "tsig_key", |output| {
        tsig.key().write_json_string(output)
    })
}
