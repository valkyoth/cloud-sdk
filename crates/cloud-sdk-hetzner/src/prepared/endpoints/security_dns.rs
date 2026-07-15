//! Security and DNS endpoint adapters.

use cloud_sdk::operation::CostIntent;

use crate::dns::rrsets::{RrsetActionEndpoint, RrsetEndpoint, RrsetListRequest};
use crate::dns::zones::{ZoneActionEndpoint, ZoneActionListRequest, ZoneEndpoint, ZoneListRequest};
use crate::security::certificates::{
    CertificateActionEndpoint, CertificateActionListForCertificateRequest,
    CertificateActionListRequest, CertificateEndpoint, CertificateListRequest,
};
use crate::security::ssh_keys::{SshKeyEndpoint, SshKeyListRequest};

use super::super::{RequestShape, ResponseProfile};

endpoint_wire!(
    CertificateEndpoint,
    endpoint => match endpoint {
        CertificateEndpoint::List => RequestShape::OptionalQuery,
        CertificateEndpoint::Create | CertificateEndpoint::Update(_) => RequestShape::RequiredJson,
        CertificateEndpoint::Get(_)
        | CertificateEndpoint::Delete(_)
        | CertificateEndpoint::Retry(_) => RequestShape::None,
    },
    match endpoint {
        CertificateEndpoint::Create | CertificateEndpoint::Retry(_) => {
            ResponseProfile::JsonCreated
        }
        CertificateEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        CertificateEndpoint::List => "list_certificates",
        CertificateEndpoint::Create => "create_certificate",
        CertificateEndpoint::Get(_) => "get_certificate",
        CertificateEndpoint::Update(_) => "update_certificate",
        CertificateEndpoint::Delete(_) => "delete_certificate",
        CertificateEndpoint::Retry(_) => "retry_certificate",
    },
    match endpoint {
        CertificateEndpoint::Delete(_) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);

query_wire!(CertificateListRequest<'_>, request => {
    let _ = request;
    CertificateEndpoint::List
});

endpoint_wire!(
    CertificateActionEndpoint,
    endpoint => match endpoint {
        CertificateActionEndpoint::ListAll | CertificateActionEndpoint::ListForCertificate(_) => {
            RequestShape::OptionalQuery
        }
        CertificateActionEndpoint::Get(_) => RequestShape::None,
    },
    ResponseProfile::JsonOk,
    match endpoint {
        CertificateActionEndpoint::ListAll => "list_certificates_actions",
        CertificateActionEndpoint::Get(_) => "get_certificates_action",
        CertificateActionEndpoint::ListForCertificate(_) => "list_certificate_actions",
    },
    false,
    CostIntent::NoKnownCost
);

query_wire!(CertificateActionListRequest<'_>, request => {
    let _ = request;
    CertificateActionEndpoint::ListAll
});
query_wire!(CertificateActionListForCertificateRequest<'_>, request => request.endpoint());

endpoint_wire!(
    SshKeyEndpoint,
    endpoint => match endpoint {
        SshKeyEndpoint::List => RequestShape::OptionalQuery,
        SshKeyEndpoint::Create | SshKeyEndpoint::Update(_) => RequestShape::RequiredJson,
        SshKeyEndpoint::Get(_) | SshKeyEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        SshKeyEndpoint::Create => ResponseProfile::JsonCreated,
        SshKeyEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        SshKeyEndpoint::List => "list_ssh_keys",
        SshKeyEndpoint::Create => "create_ssh_key",
        SshKeyEndpoint::Get(_) => "get_ssh_key",
        SshKeyEndpoint::Update(_) => "update_ssh_key",
        SshKeyEndpoint::Delete(_) => "delete_ssh_key",
    },
    match endpoint {
        SshKeyEndpoint::Delete(_) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);

query_wire!(SshKeyListRequest<'_>, request => {
    let _ = request;
    SshKeyEndpoint::List
});

endpoint_wire!(
    ZoneEndpoint<'_>,
    endpoint => match endpoint {
        ZoneEndpoint::List => RequestShape::OptionalQuery,
        ZoneEndpoint::Create | ZoneEndpoint::Update(_) => RequestShape::RequiredJson,
        ZoneEndpoint::Get(_) | ZoneEndpoint::Delete(_) | ZoneEndpoint::ExportZoneFile(_) => {
            RequestShape::None
        }
    },
    match endpoint {
        ZoneEndpoint::Create | ZoneEndpoint::Delete(_) => ResponseProfile::JsonCreated,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        ZoneEndpoint::List => "list_zones",
        ZoneEndpoint::Create => "create_zone",
        ZoneEndpoint::Get(_) => "get_zone",
        ZoneEndpoint::Update(_) => "update_zone",
        ZoneEndpoint::Delete(_) => "delete_zone",
        ZoneEndpoint::ExportZoneFile(_) => "get_zone_zonefile",
    },
    match endpoint {
        ZoneEndpoint::Delete(_) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);

query_wire!(ZoneListRequest<'_>, request => {
    let _ = request;
    ZoneEndpoint::List
});

endpoint_wire!(
    ZoneActionEndpoint<'_>,
    endpoint => match endpoint {
        ZoneActionEndpoint::ListAll | ZoneActionEndpoint::ListForZone(_) => {
            RequestShape::OptionalQuery
        }
        ZoneActionEndpoint::Get(_) => RequestShape::None,
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        ZoneActionEndpoint::ListAll
        | ZoneActionEndpoint::Get(_)
        | ZoneActionEndpoint::ListForZone(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        ZoneActionEndpoint::ListAll => "list_zones_actions",
        ZoneActionEndpoint::Get(_) => "get_zones_action",
        ZoneActionEndpoint::ListForZone(_) => "list_zone_actions",
        ZoneActionEndpoint::ChangePrimaryNameservers(_) => "change_zone_primary_nameservers",
        ZoneActionEndpoint::ChangeProtection(_) => "change_zone_protection",
        ZoneActionEndpoint::ChangeTtl(_) => "change_zone_ttl",
        ZoneActionEndpoint::ImportZoneFile(_) => "import_zone_zonefile",
    },
    match endpoint {
        ZoneActionEndpoint::ChangeProtection(_) | ZoneActionEndpoint::ImportZoneFile(_) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);

impl crate::prepared::QueryWire for ZoneActionListRequest {
    fn write_query(
        self,
        output: &mut [u8],
    ) -> Result<usize, super::super::HetznerPreparationError> {
        self.write_query(ZoneActionEndpoint::ListAll, output)
            .map_err(|_| super::super::HetznerPreparationError::Query)
    }

    fn operation_key(self) -> &'static str {
        "list_zones_actions"
    }

    fn accepts_operation(self, operation_key: &str) -> bool {
        match operation_key {
            "list_zones_actions" | "list_zone_actions" => true,
            _ => false,
        }
    }
}

impl cloud_sdk::operation::PrepareOperation for ZoneActionListRequest {
    type Error = super::super::HetznerPreparationError;

    fn prepare<'storage>(
        &self,
        storage: cloud_sdk::operation::PreparationStorage<'storage>,
    ) -> Result<cloud_sdk::operation::PreparedRequest<'storage>, Self::Error> {
        super::super::HetznerPreparedOperation::query(ZoneActionEndpoint::ListAll, *self)
            .prepare(storage)
    }
}

endpoint_wire!(
    RrsetEndpoint<'_>,
    endpoint => match endpoint {
        RrsetEndpoint::List(_) => RequestShape::OptionalQuery,
        RrsetEndpoint::Create(_) | RrsetEndpoint::Update(_) => RequestShape::RequiredJson,
        RrsetEndpoint::Get(_) | RrsetEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        RrsetEndpoint::Create(_) | RrsetEndpoint::Delete(_) => ResponseProfile::JsonCreated,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        RrsetEndpoint::List(_) => "list_zone_rrsets",
        RrsetEndpoint::Create(_) => "create_zone_rrset",
        RrsetEndpoint::Get(_) => "get_zone_rrset",
        RrsetEndpoint::Update(_) => "update_zone_rrset",
        RrsetEndpoint::Delete(_) => "delete_zone_rrset",
    },
    match endpoint {
        RrsetEndpoint::Delete(_) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);

query_wire!(RrsetListRequest<'_>, request => request.endpoint());

endpoint_wire!(
    RrsetActionEndpoint<'_>,
    endpoint => RequestShape::RequiredJson,
    match endpoint {
        RrsetActionEndpoint::UpdateRecords(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        RrsetActionEndpoint::ChangeProtection(_) => "change_zone_rrset_protection",
        RrsetActionEndpoint::ChangeTtl(_) => "change_zone_rrset_ttl",
        RrsetActionEndpoint::SetRecords(_) => "set_zone_rrset_records",
        RrsetActionEndpoint::AddRecords(_) => "add_zone_rrset_records",
        RrsetActionEndpoint::RemoveRecords(_) => "remove_zone_rrset_records",
        RrsetActionEndpoint::UpdateRecords(_) => "update_zone_rrset_records",
    },
    match endpoint {
        RrsetActionEndpoint::ChangeProtection(_)
            | RrsetActionEndpoint::SetRecords(_)
            | RrsetActionEndpoint::RemoveRecords(_) => true,
        _ => false,
    },
    CostIntent::NoKnownCost
);
