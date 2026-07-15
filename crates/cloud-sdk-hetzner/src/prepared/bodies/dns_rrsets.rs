//! DNS RRSet JSON bodies.

use crate::dns::rrsets::{
    Record, RecordUpdate, RecordUpdates, Records, RrsetAddRecordsRequest, RrsetCreateRequest,
    RrsetProtectionRequest, RrsetRemoveRecordsRequest, RrsetSetRecordsRequest, RrsetTtl,
    RrsetTtlRequest, RrsetUpdateRecordsRequest, RrsetUpdateRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(RrsetCreateRequest<'_>, request => request.endpoint(), "create_zone_rrset", write_create);
body_wire!(RrsetUpdateRequest<'_>, request => request.endpoint(), "update_zone_rrset", write_update);
body_wire!(RrsetProtectionRequest<'_>, request => request.endpoint(), "change_zone_rrset_protection", write_protection);
body_wire!(RrsetTtlRequest<'_>, request => request.endpoint(), "change_zone_rrset_ttl", write_ttl_request);
body_wire!(RrsetSetRecordsRequest<'_>, request => request.endpoint(), "set_zone_rrset_records", write_set_records);
body_wire!(RrsetAddRecordsRequest<'_>, request => request.endpoint(), "add_zone_rrset_records", write_add_records);
body_wire!(RrsetRemoveRecordsRequest<'_>, request => request.endpoint(), "remove_zone_rrset_records", write_remove_records);
body_wire!(RrsetUpdateRecordsRequest<'_>, request => request.endpoint(), "update_zone_rrset_records", write_update_records);

fn write_create(
    request: RrsetCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        writer.field_string(first, "name", request.name().as_str())?;
        write_records_field(writer, first, request.records())?;
        if let Some(ttl) = request.ttl() {
            write_ttl_field(writer, first, ttl)?;
        }
        writer.field_string(first, "type", request.rr_type().as_api_str())
    })
}

fn write_update(
    request: RrsetUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        Ok(())
    })
}

fn write_protection(
    request: RrsetProtectionRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "change", request.change())
    })
}

fn write_ttl_request(
    request: RrsetTtlRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_ttl_field(writer, first, request.ttl())
    })
}

fn write_set_records(
    request: RrsetSetRecordsRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_records_field(writer, first, request.records())
    })
}

fn write_add_records(
    request: RrsetAddRecordsRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_records_field(writer, first, request.records())?;
        if let Some(ttl) = request.ttl() {
            write_ttl_field(writer, first, ttl)?;
        }
        Ok(())
    })
}

fn write_remove_records(
    request: RrsetRemoveRecordsRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_records_field(writer, first, request.records())
    })
}

fn write_update_records(
    request: RrsetUpdateRecordsRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_updates_field(writer, first, request.records())
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

fn write_ttl_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    ttl: RrsetTtl,
) -> Result<(), HetznerPreparationError> {
    match ttl {
        RrsetTtl::InheritZoneDefault => writer.field_null(first, "ttl"),
        RrsetTtl::Explicit(ttl) => writer.field_u64(first, "ttl", u64::from(ttl.get())),
    }
}

fn write_records_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    records: Records<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "records")?;
    writer.begin_array()?;
    let mut first_record = true;
    for record in records.entries() {
        writer.value(&mut first_record)?;
        write_record(writer, *record)?;
    }
    writer.end_array()
}

fn write_record(
    writer: &mut JsonWriter<'_>,
    record: Record<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    let mut first = true;
    writer.field_sensitive(&mut first, "value", |output| {
        record.value().write_json_string(output)
    })?;
    if let Some(comment) = record.comment() {
        writer.field_sensitive(&mut first, "comment", |output| {
            comment.write_json_string(output)
        })?;
    }
    writer.end_object()
}

fn write_updates_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    records: RecordUpdates<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "records")?;
    writer.begin_array()?;
    let mut first_record = true;
    for record in records.entries() {
        writer.value(&mut first_record)?;
        write_record_update(writer, *record)?;
    }
    writer.end_array()
}

fn write_record_update(
    writer: &mut JsonWriter<'_>,
    record: RecordUpdate<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    let mut first = true;
    writer.field_sensitive(&mut first, "value", |output| {
        record.value().write_json_string(output)
    })?;
    writer.field_sensitive(&mut first, "comment", |output| {
        record.comment().write_json_string(output)
    })?;
    writer.end_object()
}
