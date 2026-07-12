use cloud_sdk::Method;

use super::*;
use crate::EndpointGroup;
use crate::cloud::shared::CloudResourceId;
use crate::dns::zones::{ZoneName, ZoneReference, ZoneTtl};
use crate::labels::LabelSelector;
use crate::pagination::{Page, PerPage, SortDirection};
use crate::request::ApiBaseUrl;

struct DebugBuffer {
    bytes: [u8; 128],
    len: usize,
}

impl core::fmt::Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> core::fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(core::fmt::Error)?;
        let target = self.bytes.get_mut(self.len..end).ok_or(core::fmt::Error)?;
        target.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn debug_text(value: &impl core::fmt::Debug) -> DebugBuffer {
    let mut output = DebugBuffer {
        bytes: [0; 128],
        len: 0,
    };
    assert!(core::fmt::write(&mut output, format_args!("{value:?}")).is_ok());
    output
}

impl DebugBuffer {
    fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.bytes.get(..self.len)?).ok()
    }
}

macro_rules! valid {
    ($expression:expr) => {
        match $expression {
            Ok(value) => value,
            Err(error) => panic!("expected valid value, got {error:?}"),
        }
    };
}

fn zone() -> ZoneReference<'static> {
    ZoneReference::Name(valid!(ZoneName::new("example.com")))
}

fn name(value: &'static str) -> RrsetName<'static> {
    valid!(RrsetName::new(value))
}

fn value(value: &'static str) -> RecordValue<'static> {
    valid!(RecordValue::new(value))
}

fn records<'a>(values: &'a [Record<'a>]) -> Records<'a> {
    valid!(Records::new(values))
}

fn rrset(name_value: &'static str, rr_type: RrsetType) -> RrsetReference<'static> {
    RrsetReference::new(zone(), name(name_value), rr_type)
}

fn written_path(endpoint: RrsetEndpoint<'_>) -> ([u8; 256], usize) {
    let mut output = [0_u8; 256];
    let len = valid!(endpoint.write_path(&mut output));
    (output, len)
}

fn written_action_path(endpoint: RrsetActionEndpoint<'_>) -> ([u8; 256], usize) {
    let mut output = [0_u8; 256];
    let len = valid!(endpoint.write_path(&mut output));
    (output, len)
}

#[test]
fn dns_rrsets_names_are_relative_lowercase_and_path_safe() {
    for invalid in [
        "", ".www", "www.", "WWW", "bad/name", "bad name", "foo.*", "-bad", "bad-",
    ] {
        assert_eq!(RrsetName::new(invalid), Err(RrsetRequestError::InvalidName));
    }
    assert!(RrsetName::new("@").is_ok());
    assert!(RrsetName::new("_sip._tcp").is_ok());
    assert!(RrsetName::new("*.apps").is_ok());
    assert!(RrsetName::new("xn--4bi").is_ok());

    let endpoint = RrsetEndpoint::Get(rrset("@", RrsetType::Aaaa));
    let (output, len) = written_path(endpoint);
    assert_eq!(
        output.get(..len),
        Some(b"/zones/example.com/rrsets/%40/AAAA".as_slice())
    );
    let endpoint = RrsetEndpoint::Get(rrset("*.apps", RrsetType::Txt));
    let (output, len) = written_path(endpoint);
    assert_eq!(
        output.get(..len),
        Some(b"/zones/example.com/rrsets/%2A.apps/TXT".as_slice())
    );
}

#[test]
fn dns_rrsets_type_surface_is_exhaustive_and_source_locked() {
    let values = [
        (RrsetType::A, "A"),
        (RrsetType::Aaaa, "AAAA"),
        (RrsetType::Caa, "CAA"),
        (RrsetType::Cname, "CNAME"),
        (RrsetType::Ds, "DS"),
        (RrsetType::Hinfo, "HINFO"),
        (RrsetType::Https, "HTTPS"),
        (RrsetType::Mx, "MX"),
        (RrsetType::Ns, "NS"),
        (RrsetType::Ptr, "PTR"),
        (RrsetType::Rp, "RP"),
        (RrsetType::Soa, "SOA"),
        (RrsetType::Srv, "SRV"),
        (RrsetType::Svcb, "SVCB"),
        (RrsetType::Tlsa, "TLSA"),
        (RrsetType::Txt, "TXT"),
    ];
    assert_eq!(values.len(), RRSET_TYPE_COUNT);
    for (rr_type, expected) in values {
        assert_eq!(rr_type.as_api_str(), expected);
    }
    assert_eq!(
        RrsetTypeFilter::new(&[]),
        Err(RrsetRequestError::InvalidTypeFilter)
    );
    assert_eq!(
        RrsetTypeFilter::new(&[RrsetType::A, RrsetType::A]),
        Err(RrsetRequestError::InvalidTypeFilter)
    );
    let too_many = [RrsetType::A; RRSET_TYPE_COUNT + 1];
    assert_eq!(
        RrsetTypeFilter::new(&too_many),
        Err(RrsetRequestError::InvalidTypeFilter)
    );
}

#[test]
fn dns_rrsets_record_values_are_bounded_escaped_and_unique() {
    assert_eq!(
        RecordValue::new(""),
        Err(RrsetRequestError::InvalidRecordValue)
    );
    assert_eq!(
        RecordValue::new("value\n"),
        Err(RrsetRequestError::InvalidRecordValue)
    );
    assert_eq!(
        RecordValue::new("safe\u{202e}spoof"),
        Err(RrsetRequestError::InvalidRecordValue)
    );
    assert_eq!(
        RecordComment::new("comment\u{2066}"),
        Err(RrsetRequestError::InvalidRecordComment)
    );
    let txt = value("\"quoted\" \\ value");
    assert_eq!(debug_text(&txt).as_str(), Some("RecordValue([redacted])"));
    let comment = valid!(RecordComment::new("internal note"));
    assert_eq!(
        debug_text(&comment).as_str(),
        Some("RecordComment([redacted])")
    );
    let mut output = [0_u8; 64];
    let escaped_comment = valid!(RecordComment::new("quoted \"comment\""));
    let comment_len = valid!(escaped_comment.write_json_string(&mut output));
    assert_eq!(
        output.get(..comment_len),
        Some(b"\"quoted \\\"comment\\\"\"".as_slice())
    );
    let len = valid!(txt.write_json_string(&mut output));
    assert_eq!(
        output.get(..len),
        Some(b"\"\\\"quoted\\\" \\\\ value\"".as_slice())
    );
    let mut small = [b'x'; 4];
    assert_eq!(
        txt.write_json_string(&mut small),
        Err(RrsetRequestError::BodyBufferTooSmall)
    );
    assert_eq!(small, [b'x'; 4]);

    let duplicates = [
        Record::new(value("192.0.2.1")),
        Record::new(value("192.0.2.1")).with_comment(valid!(RecordComment::new("other"))),
    ];
    assert_eq!(
        Records::new(&duplicates),
        Err(RrsetRequestError::DuplicateRecord)
    );
    assert_eq!(Records::new(&[]), Err(RrsetRequestError::EmptyRecords));
    let too_many = [Record::new(value("192.0.2.1")); MAX_RECORDS_PER_REQUEST + 1];
    assert_eq!(
        Records::new(&too_many),
        Err(RrsetRequestError::TooManyRecords)
    );
    let too_long_value = "x".repeat(MAX_RECORD_VALUE_BYTES + 1);
    assert_eq!(
        RecordValue::new(&too_long_value),
        Err(RrsetRequestError::InvalidRecordValue)
    );
    let too_long_comment = "x".repeat(MAX_RECORD_COMMENT_BYTES + 1);
    assert_eq!(
        RecordComment::new(&too_long_comment),
        Err(RrsetRequestError::InvalidRecordComment)
    );
}

#[test]
fn dns_rrsets_resource_paths_and_methods_match_source_lock() {
    let reference = rrset("www", RrsetType::A);
    let cases = [
        (
            RrsetEndpoint::List(zone()),
            Method::Get,
            "/zones/example.com/rrsets",
        ),
        (
            RrsetEndpoint::Create(zone()),
            Method::Post,
            "/zones/example.com/rrsets",
        ),
        (
            RrsetEndpoint::Get(reference),
            Method::Get,
            "/zones/example.com/rrsets/www/A",
        ),
        (
            RrsetEndpoint::Update(reference),
            Method::Put,
            "/zones/example.com/rrsets/www/A",
        ),
        (
            RrsetEndpoint::Delete(reference),
            Method::Delete,
            "/zones/example.com/rrsets/www/A",
        ),
    ];
    for (endpoint, method, path) in cases {
        assert_eq!(endpoint.method(), method);
        assert_eq!(endpoint.endpoint_group(), EndpointGroup::ZoneRrsets);
        assert_eq!(endpoint.api_base_url(), ApiBaseUrl::CloudV1);
        let (output, len) = written_path(endpoint);
        assert_eq!(
            core::str::from_utf8(output.get(..len).unwrap_or_default()),
            Ok(path)
        );
    }
    let id = CloudResourceId::new(42);
    assert!(id.is_some());
    let Some(id) = id else {
        return;
    };
    let id_zone = ZoneReference::Id(id);
    let endpoint = RrsetEndpoint::List(id_zone);
    let (output, len) = written_path(endpoint);
    assert_eq!(output.get(..len), Some(b"/zones/42/rrsets".as_slice()));
}

#[test]
fn dns_rrsets_list_query_is_deterministic_and_repeats_types() {
    let types = [RrsetType::A, RrsetType::Aaaa, RrsetType::Txt];
    let request = RrsetListRequest::new(zone())
        .with_name(name("www"))
        .with_types(valid!(RrsetTypeFilter::new(&types)))
        .with_label_selector(valid!(LabelSelector::new("environment=prod")))
        .with_page(valid!(Page::new(2)), valid!(PerPage::new(100)))
        .with_sort(RrsetSortField::Created, SortDirection::Desc);
    assert_eq!(request.endpoint(), RrsetEndpoint::List(zone()));
    let mut output = [0_u8; 256];
    let len = valid!(request.write_query(&mut output));
    assert_eq!(
        output.get(..len),
        Some(
            b"label_selector=environment%3Dprod&name=www&page=2&per_page=100&sort=created%3Adesc&type=A&type=AAAA&type=TXT"
                .as_slice()
        )
    );
    let mut small = [0_u8; 8];
    assert!(matches!(
        request.write_query(&mut small),
        Err(RrsetRequestError::Cloud(_))
    ));
}

#[test]
fn dns_rrsets_create_requires_fields_and_preserves_ttl_intent() {
    let entries = [Record::new(value("192.0.2.1"))];
    let entries = records(&entries);
    assert_eq!(
        RrsetCreateRequest::try_new(zone(), None, Some(RrsetType::A), Some(entries)),
        Err(RrsetRequestError::MissingRequiredField)
    );
    let request = valid!(RrsetCreateRequest::try_new(
        zone(),
        Some(name("www")),
        Some(RrsetType::A),
        Some(entries)
    ));
    assert_eq!(request.ttl(), None);
    assert_eq!(
        request.with_ttl(RrsetTtl::InheritZoneDefault).ttl(),
        Some(RrsetTtl::InheritZoneDefault)
    );
    let ttl = valid!(ZoneTtl::new(3600));
    assert_eq!(
        request.with_ttl(RrsetTtl::Explicit(ttl)).ttl(),
        Some(RrsetTtl::Explicit(ttl))
    );
    let label_entries = [];
    let empty_labels = valid!(RrsetLabels::new(&label_entries));
    let update = RrsetUpdateRequest::new(rrset("www", RrsetType::A)).with_labels(empty_labels);
    assert_eq!(
        update.endpoint(),
        RrsetEndpoint::Update(rrset("www", RrsetType::A))
    );
    assert_eq!(update.labels(), Some(empty_labels));
}

#[test]
fn dns_rrsets_actions_cover_all_mutation_semantics() {
    let reference = rrset("www", RrsetType::A);
    let cases = [
        (
            RrsetActionEndpoint::ChangeProtection(reference),
            "/zones/example.com/rrsets/www/A/actions/change_protection",
        ),
        (
            RrsetActionEndpoint::ChangeTtl(reference),
            "/zones/example.com/rrsets/www/A/actions/change_ttl",
        ),
        (
            RrsetActionEndpoint::SetRecords(reference),
            "/zones/example.com/rrsets/www/A/actions/set_records",
        ),
        (
            RrsetActionEndpoint::AddRecords(reference),
            "/zones/example.com/rrsets/www/A/actions/add_records",
        ),
        (
            RrsetActionEndpoint::RemoveRecords(reference),
            "/zones/example.com/rrsets/www/A/actions/remove_records",
        ),
        (
            RrsetActionEndpoint::UpdateRecords(reference),
            "/zones/example.com/rrsets/www/A/actions/update_records",
        ),
    ];
    for (endpoint, path) in cases {
        assert_eq!(endpoint.method(), Method::Post);
        assert_eq!(endpoint.endpoint_group(), EndpointGroup::ZoneRrsetActions);
        assert_eq!(endpoint.api_base_url(), ApiBaseUrl::CloudV1);
        let (output, len) = written_action_path(endpoint);
        assert_eq!(
            core::str::from_utf8(output.get(..len).unwrap_or_default()),
            Ok(path)
        );
    }

    let entries = [Record::new(value("192.0.2.1"))];
    let entries = records(&entries);
    assert_eq!(
        RrsetTtlRequest::new(reference, RrsetTtl::InheritZoneDefault).ttl(),
        RrsetTtl::InheritZoneDefault
    );
    assert_eq!(RrsetAddRecordsRequest::new(reference, entries).ttl(), None);
    assert_eq!(
        RrsetAddRecordsRequest::new(reference, entries)
            .with_ttl(RrsetTtl::InheritZoneDefault)
            .ttl(),
        Some(RrsetTtl::InheritZoneDefault)
    );
    assert!(RrsetProtectionRequest::new(reference, true).change());
    assert_eq!(
        RrsetProtectionRequest::new(reference, true).endpoint(),
        RrsetActionEndpoint::ChangeProtection(reference)
    );
    assert_eq!(
        RrsetSetRecordsRequest::new(reference, entries).records(),
        entries
    );
    assert_eq!(
        RrsetRemoveRecordsRequest::new(reference, entries).records(),
        entries
    );
    assert_eq!(
        RrsetSetRecordsRequest::new(reference, entries).endpoint(),
        RrsetActionEndpoint::SetRecords(reference)
    );
    assert_eq!(
        RrsetAddRecordsRequest::new(reference, entries).endpoint(),
        RrsetActionEndpoint::AddRecords(reference)
    );
    assert_eq!(
        RrsetRemoveRecordsRequest::new(reference, entries).endpoint(),
        RrsetActionEndpoint::RemoveRecords(reference)
    );
}

#[test]
fn dns_rrsets_updates_require_comments_and_unique_values() {
    let first = RecordUpdate::new(value("192.0.2.1"), valid!(RecordComment::new("primary")));
    let duplicate = RecordUpdate::new(value("192.0.2.1"), valid!(RecordComment::new("new")));
    assert_eq!(
        RecordUpdates::new(&[first, duplicate]),
        Err(RrsetRequestError::DuplicateRecord)
    );
    let updates = [first];
    let updates = valid!(RecordUpdates::new(&updates));
    let request = RrsetUpdateRecordsRequest::new(rrset("www", RrsetType::A), updates);
    assert_eq!(request.records(), updates);
    assert_eq!(
        request
            .records()
            .entries()
            .first()
            .map(|item| item.comment()),
        Some(first.comment())
    );
    assert_eq!(
        request.endpoint(),
        RrsetActionEndpoint::UpdateRecords(rrset("www", RrsetType::A))
    );
    assert_eq!(
        RecordUpdates::new(&[]),
        Err(RrsetRequestError::EmptyRecords)
    );
}

#[test]
fn dns_rrsets_paths_fail_closed_on_small_buffers() {
    let mut output = [0_u8; 8];
    assert!(matches!(
        RrsetEndpoint::Get(rrset("www", RrsetType::A)).write_path(&mut output),
        Err(RrsetRequestError::Cloud(_))
    ));
    assert!(matches!(
        RrsetActionEndpoint::SetRecords(rrset("www", RrsetType::A)).write_path(&mut output),
        Err(RrsetRequestError::Cloud(_))
    ));
}
