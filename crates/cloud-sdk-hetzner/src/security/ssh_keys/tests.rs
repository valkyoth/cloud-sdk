use super::{
    SecurityLabels, SshKeyCreateRequest, SshKeyEndpoint, SshKeyId, SshKeyListRequest, SshKeyName,
    SshKeySortField, SshPublicKey,
};
use crate::EndpointGroup;
use crate::labels::{LabelKey, LabelSelector, LabelValue};
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;
use crate::security::shared::SecurityRequestError;
use core::fmt::Write;

#[test]
fn ssh_key_paths_match_api_matrix() {
    let id = SshKeyId::new(42);
    let mut output = [0u8; 32];
    if let Some(id) = id {
        assert_eq!(SshKeyEndpoint::List.write_path(&mut output), Ok(9));
        assert_eq!(SshKeyEndpoint::Get(id).write_path(&mut output), Ok(12));
        let path = output
            .get(..12)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(path, Some("/ssh_keys/42"));
        assert_eq!(SshKeyEndpoint::Delete(id).method().as_str(), "DELETE");
        assert_eq!(
            SshKeyEndpoint::Update(id).api_base_url(),
            ApiBaseUrl::CloudV1
        );
        assert_eq!(
            SshKeyEndpoint::Create.endpoint_group(),
            EndpointGroup::SshKeys
        );
    }
}

#[test]
fn ssh_key_list_query_writes_filters_pagination_and_sorting() {
    let name = SshKeyName::new("deploy key");
    let selector = LabelSelector::new("env=prod");
    let page = Page::new(2);
    let per_page = PerPage::new(25);
    let mut output = [0u8; 128];
    if let (Ok(name), Ok(selector), Ok(page), Ok(per_page)) = (name, selector, page, per_page) {
        let request = SshKeyListRequest::new()
            .with_name(name)
            .with_label_selector(selector)
            .with_page(page)
            .with_per_page(per_page)
            .with_sort(SshKeySortField::Name, SortDirection::Asc);
        assert_eq!(request.write_query(&mut output), Ok(78));
        let query = output
            .get(..78)
            .and_then(|bytes| core::str::from_utf8(bytes).ok());
        assert_eq!(
            query,
            Some("label_selector=env%3Dprod&name=deploy%20key&page=2&per_page=25&sort=name%3Aasc")
        );
    }
}

#[test]
fn ssh_key_create_validates_required_fields_and_redacts_debug() {
    let name = SshKeyName::new("deploy");
    let public_key = SshPublicKey::new("ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIMockKey");
    if let (Ok(name), Ok(public_key)) = (name, public_key) {
        assert_eq!(
            SshKeyCreateRequest::try_new(None, Some(public_key)),
            Err(SecurityRequestError::MissingRequiredField)
        );
        let request = SshKeyCreateRequest::try_new(Some(name), Some(public_key));
        assert!(request.is_ok());
        if let Ok(request) = request {
            let mut debug = DebugBuffer::new();
            assert!(write!(&mut debug, "{request:?}").is_ok());
            let debug = debug.as_str();
            assert!(debug.contains("[redacted]"));
            assert!(!debug.contains("AAAAC3"));
        }
    }
}

struct DebugBuffer {
    bytes: [u8; 256],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0u8; 256],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

#[test]
fn ssh_key_validation_rejects_bad_inputs() {
    assert_eq!(
        SshKeyName::new("bad\n"),
        Err(SecurityRequestError::InvalidNameByte)
    );
    assert_eq!(
        SshPublicKey::new("not-a-key"),
        Err(SecurityRequestError::InvalidSshPublicKey)
    );
    assert_eq!(
        SshKeyListRequest::new().with_fingerprint("zz:zz"),
        Err(SecurityRequestError::InvalidSshFingerprint)
    );
    assert_eq!(
        SshKeyListRequest::new()
            .with_fingerprint("11:22:33:44:55:66:77:88:99:aa:bb:cc:dd:ee:ff:00:11:22:33:44:55:66"),
        Err(SecurityRequestError::InvalidSshFingerprint)
    );
    let key = LabelKey::new("z");
    let value = LabelValue::new("");
    if let (Ok(key), Ok(value)) = (key, value) {
        let labels = [(key, value)];
        assert!(SecurityLabels::new(&labels).is_ok());
    }
    let duplicate_key = LabelKey::new("a");
    let first = LabelValue::new("one");
    let second = LabelValue::new("two");
    if let (Ok(duplicate_key), Ok(first), Ok(second)) = (duplicate_key, first, second) {
        let labels = [(duplicate_key, first), (duplicate_key, second)];
        assert_eq!(
            SecurityLabels::new(&labels),
            Err(SecurityRequestError::InvalidLabel(
                crate::labels::LabelError::InvalidSelectorSyntax
            ))
        );
    }
}
