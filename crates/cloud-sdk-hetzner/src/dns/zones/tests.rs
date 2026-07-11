use super::*;

use crate::actions::ActionStatus;
use crate::pagination::{Page, PerPage, SortDirection};

struct DebugBuffer {
    bytes: [u8; 128],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    fn as_str(&self) -> Option<&str> {
        core::str::from_utf8(self.bytes.get(..self.len)?).ok()
    }
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

fn debug_buffer(value: &impl core::fmt::Debug) -> DebugBuffer {
    let mut output = DebugBuffer::new();
    assert!(core::fmt::write(&mut output, format_args!("{value:?}")).is_ok());
    output
}

macro_rules! valid {
    ($value:expr) => {{
        let result = $value;
        assert!(result.is_ok());
        let Ok(value) = result else { return };
        value
    }};
}

macro_rules! some {
    ($value:expr) => {{
        let result = $value;
        assert!(result.is_some());
        let Some(value) = result else { return };
        value
    }};
}

macro_rules! id {
    ($value:expr) => {
        some!(ZoneId::new($value))
    };
}

macro_rules! action_id {
    ($value:expr) => {
        some!(ZoneActionId::new($value))
    };
}

macro_rules! name {
    ($value:expr) => {
        valid!(ZoneName::new($value))
    };
}

macro_rules! zone {
    ($value:expr) => {
        ZoneReference::Name(name!($value))
    };
}

macro_rules! ttl {
    ($value:expr) => {
        valid!(ZoneTtl::new($value))
    };
}

macro_rules! nameserver {
    ($value:expr) => {
        valid!(PrimaryNameserver::new($value))
    };
}

#[test]
fn dns_zones_resource_paths_and_methods_are_source_locked() {
    let cases = [
        (ZoneEndpoint::List, cloud_sdk::Method::Get, "/zones"),
        (ZoneEndpoint::Create, cloud_sdk::Method::Post, "/zones"),
        (
            ZoneEndpoint::Get(ZoneReference::Id(id!(42))),
            cloud_sdk::Method::Get,
            "/zones/42",
        ),
        (
            ZoneEndpoint::Update(zone!("example.com")),
            cloud_sdk::Method::Put,
            "/zones/example.com",
        ),
        (
            ZoneEndpoint::Delete(zone!("example.com")),
            cloud_sdk::Method::Delete,
            "/zones/example.com",
        ),
        (
            ZoneEndpoint::ExportZoneFile(zone!("example.com")),
            cloud_sdk::Method::Get,
            "/zones/example.com/zonefile",
        ),
    ];
    for (endpoint, method, expected) in cases {
        let mut output = [0_u8; 96];
        let len = valid!(endpoint.write_path(&mut output));
        assert_eq!(endpoint.method(), method);
        assert_eq!(endpoint.endpoint_group(), crate::EndpointGroup::Zones);
        assert_eq!(endpoint.api_base_url(), crate::request::ApiBaseUrl::CloudV1);
        assert_eq!(output.get(..len), Some(expected.as_bytes()));
    }
}

#[test]
fn dns_zones_action_paths_cover_nondeprecated_operations() {
    let cases = [
        (
            ZoneActionEndpoint::ListAll,
            cloud_sdk::Method::Get,
            "/zones/actions",
        ),
        (
            ZoneActionEndpoint::Get(action_id!(7)),
            cloud_sdk::Method::Get,
            "/zones/actions/7",
        ),
        (
            ZoneActionEndpoint::ListForZone(zone!("example.com")),
            cloud_sdk::Method::Get,
            "/zones/example.com/actions",
        ),
        (
            ZoneActionEndpoint::ChangePrimaryNameservers(zone!("example.com")),
            cloud_sdk::Method::Post,
            "/zones/example.com/actions/change_primary_nameservers",
        ),
        (
            ZoneActionEndpoint::ChangeProtection(zone!("example.com")),
            cloud_sdk::Method::Post,
            "/zones/example.com/actions/change_protection",
        ),
        (
            ZoneActionEndpoint::ChangeTtl(zone!("example.com")),
            cloud_sdk::Method::Post,
            "/zones/example.com/actions/change_ttl",
        ),
        (
            ZoneActionEndpoint::ImportZoneFile(zone!("example.com")),
            cloud_sdk::Method::Post,
            "/zones/example.com/actions/import_zonefile",
        ),
    ];
    for (endpoint, method, expected) in cases {
        let mut output = [0_u8; 96];
        let len = valid!(endpoint.write_path(&mut output));
        assert_eq!(endpoint.method(), method);
        assert_eq!(endpoint.endpoint_group(), crate::EndpointGroup::ZoneActions);
        assert_eq!(endpoint.api_base_url(), crate::request::ApiBaseUrl::CloudV1);
        assert_eq!(output.get(..len), Some(expected.as_bytes()));
    }
}

#[test]
fn dns_zones_paths_fail_closed_on_small_buffers() {
    let mut output = [0_u8; 4];
    assert_eq!(
        ZoneEndpoint::Get(zone!("example.com")).write_path(&mut output),
        Err(ZoneRequestError::Cloud(
            crate::cloud::shared::CloudRequestError::PathBufferTooSmall
        ))
    );
    assert_eq!(
        ZoneActionEndpoint::ChangeTtl(zone!("example.com")).write_path(&mut output),
        Err(ZoneRequestError::Cloud(
            crate::cloud::shared::CloudRequestError::PathBufferTooSmall
        ))
    );
}

#[test]
fn dns_zones_names_are_conservative_lowercase_domains() {
    for invalid in [
        "",
        "localhost",
        ".example.com",
        "example.com.",
        "Example.com",
        "-bad.example",
        "bad-.example",
        "bad_name.example",
        "münchen.de",
    ] {
        assert_eq!(
            ZoneName::new(invalid),
            Err(ZoneRequestError::InvalidZoneName)
        );
    }
    assert!(ZoneName::new("example.com").is_ok());
    assert!(ZoneName::new("xn--mnchen-3ya.de").is_ok());
}

#[test]
fn dns_zones_ttl_bounds_are_exact() {
    assert_eq!(ZoneTtl::new(59), Err(ZoneRequestError::InvalidTtl));
    assert_eq!(ZoneTtl::new(60).map(ZoneTtl::get), Ok(60));
    assert_eq!(
        ZoneTtl::new(2_147_483_647).map(ZoneTtl::get),
        Ok(2_147_483_647)
    );
    assert_eq!(
        ZoneTtl::new(2_147_483_648),
        Err(ZoneRequestError::InvalidTtl)
    );
}

#[test]
fn dns_zones_zonefile_is_bounded_redacted_and_atomically_escaped() {
    assert_eq!(ZoneFile::new(""), Err(ZoneRequestError::InvalidZoneFile));
    assert_eq!(
        ZoneFile::new("$ORIGIN example.com.\0"),
        Err(ZoneRequestError::InvalidZoneFile)
    );
    let too_large = "x".repeat(MAX_ZONE_FILE_BYTES + 1);
    assert_eq!(
        ZoneFile::new(&too_large),
        Err(ZoneRequestError::InvalidZoneFile)
    );

    let file = valid!(ZoneFile::new("$ORIGIN example.com.\n@ IN TXT \"value\""));
    assert_eq!(debug_buffer(&file).as_str(), Some("ZoneFile([redacted])"));
    let mut output = [0xaa_u8; 8];
    assert_eq!(
        file.write_json_string(&mut output),
        Err(ZoneRequestError::BodyBufferTooSmall)
    );
    assert_eq!(output, [0xaa_u8; 8]);
    let mut output = [0_u8; 96];
    let len = valid!(file.write_json_string(&mut output));
    assert_eq!(
        output.get(..len),
        Some(b"\"$ORIGIN example.com.\\n@ IN TXT \\\"value\\\"\"".as_slice())
    );
}

#[test]
fn dns_zones_primary_nameservers_require_public_unique_addresses() {
    for invalid in ["not-an-ip", "10.0.0.1", "127.0.0.1", "2001:db8::1"] {
        assert_eq!(
            PrimaryNameserver::new(invalid),
            Err(ZoneRequestError::InvalidNameserverAddress)
        );
    }
    let first = nameserver!("8.8.8.8");
    assert_eq!(
        first.with_port(0),
        Err(ZoneRequestError::InvalidNameserverPort)
    );
    assert_eq!(first.port(), 53);
    let duplicate = [first, valid!(first.with_port(5353))];
    assert_eq!(
        PrimaryNameservers::new(&duplicate),
        Err(ZoneRequestError::DuplicatePrimaryNameserver)
    );
    assert_eq!(
        PrimaryNameservers::new(&[]),
        Err(ZoneRequestError::EmptyPrimaryNameservers)
    );
    let too_many = [first; MAX_PRIMARY_NAMESERVERS + 1];
    assert_eq!(
        PrimaryNameservers::new(&too_many),
        Err(ZoneRequestError::TooManyPrimaryNameservers)
    );
}

#[test]
fn dns_zones_tsig_is_coherent_validated_and_redacted() {
    for invalid in ["", "abc", "YWJjZA=", "YW=JjZA=", "é=="] {
        assert_eq!(TsigKey::new(invalid), Err(ZoneRequestError::InvalidTsigKey));
    }
    let key = valid!(TsigKey::new("YWJjZA=="));
    let credentials = TsigCredentials::new(key, TsigAlgorithm::HmacSha256);
    let server = nameserver!("1.1.1.1").with_tsig(credentials);
    assert_eq!(server.tsig(), Some(credentials));
    assert!(
        !debug_buffer(&credentials)
            .as_str()
            .is_some_and(|value| value.contains("YWJjZA"))
    );
    assert!(
        !debug_buffer(&key)
            .as_str()
            .is_some_and(|value| value.contains("YWJjZA"))
    );
    let mut output = [0_u8; 16];
    let len = valid!(key.write_json_string(&mut output));
    assert_eq!(output.get(..len), Some(b"\"YWJjZA==\"".as_slice()));
}

#[test]
fn dns_zones_create_mode_prevents_conflicting_configuration() {
    assert_eq!(
        ZoneCreateRequest::try_new(None, Some(ZoneCreateMode::Primary)),
        Err(ZoneRequestError::MissingRequiredField)
    );
    let servers = [nameserver!("8.8.8.8"), nameserver!("1.1.1.1")];
    let servers = valid!(PrimaryNameservers::new(&servers));
    let secondary = valid!(ZoneCreateRequest::try_new(
        Some(name!("example.com")),
        Some(ZoneCreateMode::Secondary(servers))
    ));
    let file = valid!(ZoneFile::new("$ORIGIN example.com."));
    assert_eq!(
        secondary.with_zonefile(file),
        Err(ZoneRequestError::InvalidModeConfiguration)
    );
    let primary = valid!(ZoneCreateRequest::try_new(
        Some(name!("example.com")),
        Some(ZoneCreateMode::Primary)
    ));
    assert!(primary.with_zonefile(file).is_ok());
}

#[test]
fn dns_zones_list_query_is_encoded_and_deterministic() {
    let selector = valid!(crate::labels::LabelSelector::new("env=prod"));
    let page = valid!(Page::new(2));
    let per_page = valid!(PerPage::new(50));
    let request = ZoneListRequest::new()
        .with_label_selector(selector)
        .with_mode(ZoneMode::Secondary)
        .with_name(name!("example.com"))
        .with_page(page, per_page)
        .with_sort(ZoneSortField::Created, SortDirection::Desc);
    let mut output = [0_u8; 160];
    let len = valid!(request.write_query(&mut output));
    assert_eq!(
        output.get(..len),
        Some(
            b"label_selector=env%3Dprod&mode=secondary&name=example.com&page=2&per_page=50&sort=created%3Adesc"
                .as_slice()
        )
    );
}

#[test]
fn dns_zones_action_query_enforces_global_only_id_filter() {
    let page = valid!(Page::new(1));
    let per_page = valid!(PerPage::new(25));
    let request = ZoneActionListRequest::new()
        .with_id(action_id!(7))
        .with_status(ActionStatus::Running)
        .with_page(page, per_page)
        .with_sort(ZoneActionSortField::Started, SortDirection::Desc);
    let mut output = [0_u8; 128];
    assert_eq!(
        request.write_query(
            ZoneActionEndpoint::ListForZone(zone!("example.com")),
            &mut output
        ),
        Err(ZoneRequestError::InvalidActionFilter)
    );
    let len = valid!(request.write_query(ZoneActionEndpoint::ListAll, &mut output));
    assert_eq!(
        output.get(..len),
        Some(b"id=7&page=1&per_page=25&sort=started%3Adesc&status=running".as_slice())
    );
}

#[test]
fn dns_zones_action_bodies_preserve_explicit_intent() {
    let zone = zone!("example.com");
    let servers = [nameserver!("8.8.8.8")];
    let servers = valid!(PrimaryNameservers::new(&servers));
    let primary = ZonePrimaryNameserversRequest::new(zone, servers);
    assert_eq!(
        primary.endpoint(),
        ZoneActionEndpoint::ChangePrimaryNameservers(zone)
    );
    assert_eq!(primary.nameservers(), servers);

    let protection = ZoneProtectionRequest::new(zone, false);
    assert!(!protection.delete());
    assert_eq!(
        protection.endpoint(),
        ZoneActionEndpoint::ChangeProtection(zone)
    );

    let ttl_request = ZoneTtlRequest::new(zone, ttl!(3600));
    assert_eq!(ttl_request.ttl().get(), 3600);
    assert_eq!(ttl_request.endpoint(), ZoneActionEndpoint::ChangeTtl(zone));

    let file = valid!(ZoneFile::new("$ORIGIN example.com."));
    let import = ZoneFileImportRequest::new(zone, file);
    assert_eq!(import.zonefile(), file);
    assert_eq!(import.endpoint(), ZoneActionEndpoint::ImportZoneFile(zone));
}
