use cloud_sdk::operation::RequestBodySensitivity::{Public, Sensitive};

use crate::cloud::servers::actions::ServerActionRequest;
use crate::cloud::servers::{ServerCreateRequest, ServerName, ServerReference, UserData};
use crate::dns::rrsets::{Record, RecordValue, Records, RrsetCreateRequest, RrsetName, RrsetType};
use crate::dns::zones::{
    PrimaryNameserver, PrimaryNameservers, TsigAlgorithm, TsigCredentials, TsigKey, ZoneCreateMode,
    ZoneCreateRequest, ZoneFile, ZoneFileImportRequest, ZoneName, ZonePrimaryNameserversRequest,
    ZoneReference,
};
use crate::prepared::BodyWire;
use crate::storage::storage_boxes::{
    StorageBoxCreateRequest, StorageBoxLocation, StorageBoxName, StorageBoxPassword,
    StorageBoxResetPasswordRequest, StorageBoxTypeRef,
};

#[test]
fn server_user_data_controls_body_sensitivity() {
    let name = ServerName::new("web-1").unwrap_or_else(|_| unreachable!());
    let server_type = ServerReference::new("cpx22").unwrap_or_else(|_| unreachable!());
    let image = ServerReference::new("ubuntu-24.04").unwrap_or_else(|_| unreachable!());
    let public = ServerCreateRequest::new(name, server_type, image);
    assert_eq!(public.sensitivity(), Public);

    let user_data = UserData::new("#cloud-config\n").unwrap_or_else(|_| unreachable!());
    assert_eq!(public.with_user_data(user_data).sensitivity(), Sensitive);
    assert_eq!(
        ServerActionRequest::Rebuild {
            image,
            user_data: None,
        }
        .sensitivity(),
        Public
    );
    assert_eq!(
        ServerActionRequest::Rebuild {
            image,
            user_data: Some(user_data),
        }
        .sensitivity(),
        Sensitive
    );
}

#[test]
fn storage_password_bodies_are_sensitive() {
    let name = StorageBoxName::new("backup").unwrap_or_else(|_| unreachable!());
    let location = StorageBoxLocation::new("fsn1").unwrap_or_else(|_| unreachable!());
    let kind = StorageBoxTypeRef::new("bx20").unwrap_or_else(|_| unreachable!());
    let password = StorageBoxPassword::new("reviewed-password").unwrap_or_else(|_| unreachable!());
    let request = StorageBoxCreateRequest::new(name, location, kind, password);
    assert_eq!(request.sensitivity(), Sensitive);
    assert_eq!(
        StorageBoxResetPasswordRequest::new(password).sensitivity(),
        Sensitive
    );
}

#[test]
fn zonefile_and_tsig_bodies_are_sensitive() {
    let name = ZoneName::new("example.com").unwrap_or_else(|_| unreachable!());
    assert_eq!(
        ZoneCreateRequest::new(name, ZoneCreateMode::Primary).sensitivity(),
        Public
    );

    let key = TsigKey::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=")
        .unwrap_or_else(|_| unreachable!());
    let nameserver = PrimaryNameserver::new("1.1.1.1")
        .unwrap_or_else(|_| unreachable!())
        .with_tsig(TsigCredentials::new(key, TsigAlgorithm::HmacSha256));
    let entries = [nameserver];
    let nameservers = PrimaryNameservers::new(&entries).unwrap_or_else(|_| unreachable!());
    assert_eq!(
        ZoneCreateRequest::new(name, ZoneCreateMode::Secondary(nameservers)).sensitivity(),
        Sensitive
    );
    assert_eq!(
        ZonePrimaryNameserversRequest::new(ZoneReference::Name(name), nameservers).sensitivity(),
        Sensitive
    );

    let zonefile =
        ZoneFile::new("example.com. 60 IN A 192.0.2.1").unwrap_or_else(|_| unreachable!());
    assert_eq!(
        ZoneCreateRequest::new(name, ZoneCreateMode::Primary)
            .with_zonefile(zonefile)
            .unwrap_or_else(|_| unreachable!())
            .sensitivity(),
        Sensitive
    );
    assert_eq!(
        ZoneFileImportRequest::new(ZoneReference::Name(name), zonefile).sensitivity(),
        Sensitive
    );
}

#[test]
fn rrset_record_bodies_are_sensitive() {
    let zone = ZoneName::new("example.com").unwrap_or_else(|_| unreachable!());
    let name = RrsetName::new("www").unwrap_or_else(|_| unreachable!());
    let value = RecordValue::new("192.0.2.1").unwrap_or_else(|_| unreachable!());
    let entries = [Record::new(value)];
    let records = Records::new(&entries).unwrap_or_else(|_| unreachable!());
    let request = RrsetCreateRequest::new(ZoneReference::Name(zone), name, RrsetType::A, records);
    assert_eq!(request.sensitivity(), Sensitive);
}
