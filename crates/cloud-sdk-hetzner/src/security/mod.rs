//! Certificate and SSH key resource modules.

pub mod certificates;
pub mod ssh_keys;

use crate::EndpointGroup;

/// Endpoint groups owned by the security API module.
pub const SECURITY_ENDPOINT_GROUPS: &[EndpointGroup] = &[
    EndpointGroup::Certificates,
    EndpointGroup::CertificateActions,
    EndpointGroup::SshKeys,
];

#[cfg(test)]
mod tests {
    use super::SECURITY_ENDPOINT_GROUPS;
    use crate::EndpointGroup;

    #[test]
    fn includes_certificate_and_ssh_key_groups() {
        assert!(SECURITY_ENDPOINT_GROUPS.contains(&EndpointGroup::Certificates));
        assert!(SECURITY_ENDPOINT_GROUPS.contains(&EndpointGroup::SshKeys));
    }
}
