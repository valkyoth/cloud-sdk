//! Certificate and SSH key JSON bodies.

use crate::prepared::{HetznerPreparationError, JsonWriter};
use crate::security::certificates::{
    CertificateCreateMode, CertificateCreateRequest, CertificateType, CertificateUpdateRequest,
    SecurityLabels,
};
use crate::security::ssh_keys::{SshKeyCreateRequest, SshKeyUpdateRequest};

body_wire!(CertificateCreateRequest<'_>, request => request.endpoint(), "create_certificate", write_certificate_create);
body_wire!(CertificateUpdateRequest<'_>, request => request.endpoint(), "update_certificate", write_certificate_update);
body_wire!(SshKeyCreateRequest<'_>, request => request.endpoint(), "create_ssh_key", write_ssh_create);
body_wire!(SshKeyUpdateRequest<'_>, request => request.endpoint(), "update_ssh_key", write_ssh_update);

fn write_certificate_create(
    request: CertificateCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let (name, labels) = request.prepared_parts();
        match request.mode() {
            CertificateCreateMode::Uploaded {
                certificate,
                private_key,
            } => {
                writer.field_string(first, "certificate", certificate.as_str())?;
                writer.field_sensitive(first, "private_key", |output| {
                    private_key.write_json_string(output)
                })?;
            }
            CertificateCreateMode::Managed { domain_names } => {
                writer.field(first, "domain_names")?;
                writer.begin_array()?;
                let mut first_domain = true;
                for domain in domain_names {
                    writer.value(&mut first_domain)?;
                    writer.string(domain.as_str())?;
                }
                writer.end_array()?;
            }
        }
        if let Some(labels) = labels {
            write_labels(writer, first, labels)?;
        }
        writer.field_string(first, "name", name.as_str())?;
        writer.field_string(
            first,
            "type",
            certificate_type(request.mode().certificate_type()),
        )
    })
}

fn write_certificate_update(
    request: CertificateUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let (name, labels) = request.prepared_parts();
        if let Some(labels) = labels {
            write_labels(writer, first, labels)?;
        }
        if let Some(name) = name {
            writer.field_string(first, "name", name.as_str())?;
        }
        Ok(())
    })
}

fn write_ssh_create(
    request: SshKeyCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = request.labels() {
            write_labels(writer, first, labels)?;
        }
        writer.field_string(first, "name", request.name().as_str())?;
        writer.field_string(first, "public_key", request.public_key().as_str())
    })
}

fn write_ssh_update(
    request: SshKeyUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let (name, labels) = request.prepared_parts();
        if let Some(labels) = labels {
            write_labels(writer, first, labels)?;
        }
        if let Some(name) = name {
            writer.field_string(first, "name", name.as_str())?;
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

fn write_labels(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    labels: SecurityLabels<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field_label_entries(first, "labels", labels.entries())
}

const fn certificate_type(value: CertificateType) -> &'static str {
    match value {
        CertificateType::Uploaded => "uploaded",
        CertificateType::Managed => "managed",
    }
}
