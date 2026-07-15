//! Compute, image, placement-group, and volume endpoint adapters.

use cloud_sdk::operation::CostIntent;

use crate::cloud::images::{ImageActionEndpoint, ImageEndpoint, ImageListRequest};
use crate::cloud::servers::actions::{ServerActionEndpoint, ServerActionKind};
use crate::cloud::servers::placement_groups::{PlacementGroupEndpoint, PlacementGroupListRequest};
use crate::cloud::servers::{ServerEndpoint, ServerListRequest, ServerMetricsRequest};
use crate::cloud::volumes::{VolumeActionEndpoint, VolumeEndpoint, VolumeListRequest};

use super::super::{RequestShape, ResponseProfile};

endpoint_wire!(
    ServerEndpoint,
    endpoint => match endpoint {
        ServerEndpoint::List => RequestShape::OptionalQuery,
        ServerEndpoint::Create | ServerEndpoint::Update(_) => RequestShape::RequiredJson,
        ServerEndpoint::Metrics(_) => RequestShape::RequiredQuery,
        ServerEndpoint::Get(_) | ServerEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        ServerEndpoint::Create => ResponseProfile::JsonCreated,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        ServerEndpoint::List => "list_servers",
        ServerEndpoint::Create => "create_server",
        ServerEndpoint::Get(_) => "get_server",
        ServerEndpoint::Update(_) => "update_server",
        ServerEndpoint::Delete(_) => "delete_server",
        ServerEndpoint::Metrics(_) => "get_server_metrics",
    },
    matches!(endpoint, ServerEndpoint::Delete(_)),
    if matches!(endpoint, ServerEndpoint::Create) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

query_wire!(ServerListRequest<'_>, request => {
    let _ = request;
    ServerEndpoint::List
});
query_wire!(ServerMetricsRequest<'_>, request => request.endpoint());

endpoint_wire!(
    ServerActionEndpoint,
    endpoint => match endpoint {
        ServerActionEndpoint::ListAll | ServerActionEndpoint::ListForServer(_) => {
            RequestShape::OptionalQuery
        }
        ServerActionEndpoint::Get(_) => RequestShape::None,
        ServerActionEndpoint::Start(_, kind) if server_action_requires_body(kind) => {
            RequestShape::RequiredJson
        }
        ServerActionEndpoint::Start(_, _) => RequestShape::None,
    },
    match endpoint {
        ServerActionEndpoint::ListAll
        | ServerActionEndpoint::Get(_)
        | ServerActionEndpoint::ListForServer(_) => ResponseProfile::JsonOk,
        ServerActionEndpoint::Start(_, _) => ResponseProfile::JsonCreated,
    },
    match endpoint {
        ServerActionEndpoint::ListAll => "list_servers_actions",
        ServerActionEndpoint::Get(_) => "get_servers_action",
        ServerActionEndpoint::ListForServer(_) => "list_server_actions",
        ServerActionEndpoint::Start(_, kind) => match kind {
            ServerActionKind::AddToPlacementGroup => "add_server_to_placement_group",
            ServerActionKind::AttachIso => "attach_server_iso",
            ServerActionKind::AttachToNetwork => "attach_server_to_network",
            ServerActionKind::ChangeAliasIps => "change_server_alias_ips",
            ServerActionKind::ChangeDnsPtr => "change_server_dns_ptr",
            ServerActionKind::ChangeProtection => "change_server_protection",
            ServerActionKind::ChangeType => "change_server_type",
            ServerActionKind::CreateImage => "create_server_image",
            ServerActionKind::DetachFromNetwork => "detach_server_from_network",
            ServerActionKind::DetachIso => "detach_server_iso",
            ServerActionKind::DisableBackup => "disable_server_backup",
            ServerActionKind::DisableRescue => "disable_server_rescue",
            ServerActionKind::EnableBackup => "enable_server_backup",
            ServerActionKind::EnableRescue => "enable_server_rescue",
            ServerActionKind::Poweroff => "poweroff_server",
            ServerActionKind::Poweron => "poweron_server",
            ServerActionKind::Reboot => "reboot_server",
            ServerActionKind::Rebuild => "rebuild_server",
            ServerActionKind::RemoveFromPlacementGroup => "remove_server_from_placement_group",
            ServerActionKind::RequestConsole => "request_server_console",
            ServerActionKind::Reset => "reset_server",
            ServerActionKind::ResetPassword => "reset_server_password",
            ServerActionKind::Shutdown => "shutdown_server",
        },
    },
    matches!(
        endpoint,
        ServerActionEndpoint::Start(
            _,
            ServerActionKind::DetachFromNetwork
                | ServerActionKind::DetachIso
                | ServerActionKind::DisableBackup
                | ServerActionKind::DisableRescue
                | ServerActionKind::ChangeProtection
                | ServerActionKind::Poweroff
                | ServerActionKind::Rebuild
                | ServerActionKind::RemoveFromPlacementGroup
                | ServerActionKind::Reset
                | ServerActionKind::ResetPassword
                | ServerActionKind::Shutdown
        )
    ),
    if matches!(
        endpoint,
        ServerActionEndpoint::Start(
            _,
            ServerActionKind::ChangeType
                | ServerActionKind::CreateImage
                | ServerActionKind::EnableBackup
        )
    ) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

endpoint_wire!(
    ImageEndpoint,
    endpoint => match endpoint {
        ImageEndpoint::List => RequestShape::OptionalQuery,
        ImageEndpoint::Update(_) => RequestShape::RequiredJson,
        ImageEndpoint::Get(_) | ImageEndpoint::Delete(_) => RequestShape::None,
    },
    if matches!(endpoint, ImageEndpoint::Delete(_)) {
        ResponseProfile::NoContent
    } else {
        ResponseProfile::JsonOk
    },
    match endpoint {
        ImageEndpoint::List => "list_images",
        ImageEndpoint::Get(_) => "get_image",
        ImageEndpoint::Update(_) => "update_image",
        ImageEndpoint::Delete(_) => "delete_image",
    },
    matches!(endpoint, ImageEndpoint::Delete(_)),
    CostIntent::NoKnownCost
);

query_wire!(ImageListRequest, request => {
    let _ = request;
    ImageEndpoint::List
});

endpoint_wire!(
    ImageActionEndpoint,
    endpoint => match endpoint {
        ImageActionEndpoint::ListAll | ImageActionEndpoint::ListForImage(_) => {
            RequestShape::OptionalQuery
        }
        ImageActionEndpoint::Get(_) => RequestShape::None,
        ImageActionEndpoint::ChangeProtection(_) => RequestShape::RequiredJson,
    },
    if matches!(endpoint, ImageActionEndpoint::ChangeProtection(_)) {
        ResponseProfile::JsonCreated
    } else {
        ResponseProfile::JsonOk
    },
    match endpoint {
        ImageActionEndpoint::ListAll => "list_images_actions",
        ImageActionEndpoint::Get(_) => "get_images_action",
        ImageActionEndpoint::ListForImage(_) => "list_image_actions",
        ImageActionEndpoint::ChangeProtection(_) => "change_image_protection",
    },
    matches!(endpoint, ImageActionEndpoint::ChangeProtection(_)),
    CostIntent::NoKnownCost
);

endpoint_wire!(
    PlacementGroupEndpoint,
    endpoint => match endpoint {
        PlacementGroupEndpoint::List => RequestShape::OptionalQuery,
        PlacementGroupEndpoint::Create | PlacementGroupEndpoint::Update(_) => {
            RequestShape::RequiredJson
        }
        PlacementGroupEndpoint::Get(_) | PlacementGroupEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        PlacementGroupEndpoint::Create => ResponseProfile::JsonCreated,
        PlacementGroupEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        PlacementGroupEndpoint::List => "list_placement_groups",
        PlacementGroupEndpoint::Create => "create_placement_group",
        PlacementGroupEndpoint::Get(_) => "get_placement_group",
        PlacementGroupEndpoint::Update(_) => "update_placement_group",
        PlacementGroupEndpoint::Delete(_) => "delete_placement_group",
    },
    matches!(endpoint, PlacementGroupEndpoint::Delete(_)),
    CostIntent::NoKnownCost
);

query_wire!(PlacementGroupListRequest<'_>, request => {
    let _ = request;
    PlacementGroupEndpoint::List
});

endpoint_wire!(
    VolumeEndpoint,
    endpoint => match endpoint {
        VolumeEndpoint::List => RequestShape::OptionalQuery,
        VolumeEndpoint::Create | VolumeEndpoint::Update(_) => RequestShape::RequiredJson,
        VolumeEndpoint::Get(_) | VolumeEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        VolumeEndpoint::Create => ResponseProfile::JsonCreated,
        VolumeEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        VolumeEndpoint::List => "list_volumes",
        VolumeEndpoint::Create => "create_volume",
        VolumeEndpoint::Get(_) => "get_volume",
        VolumeEndpoint::Update(_) => "update_volume",
        VolumeEndpoint::Delete(_) => "delete_volume",
    },
    matches!(endpoint, VolumeEndpoint::Delete(_)),
    if matches!(endpoint, VolumeEndpoint::Create) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

query_wire!(VolumeListRequest<'_>, request => {
    let _ = request;
    VolumeEndpoint::List
});

endpoint_wire!(
    VolumeActionEndpoint,
    endpoint => match endpoint {
        VolumeActionEndpoint::ListAll | VolumeActionEndpoint::ListForVolume(_) => {
            RequestShape::OptionalQuery
        }
        VolumeActionEndpoint::Get(_) | VolumeActionEndpoint::Detach(_) => RequestShape::None,
        VolumeActionEndpoint::Attach(_)
        | VolumeActionEndpoint::Resize(_)
        | VolumeActionEndpoint::ChangeProtection(_) => RequestShape::RequiredJson,
    },
    match endpoint {
        VolumeActionEndpoint::ListAll
        | VolumeActionEndpoint::Get(_)
        | VolumeActionEndpoint::ListForVolume(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        VolumeActionEndpoint::ListAll => "list_volumes_actions",
        VolumeActionEndpoint::Get(_) => "get_volumes_action",
        VolumeActionEndpoint::ListForVolume(_) => "list_volume_actions",
        VolumeActionEndpoint::Attach(_) => "attach_volume",
        VolumeActionEndpoint::ChangeProtection(_) => "change_volume_protection",
        VolumeActionEndpoint::Detach(_) => "detach_volume",
        VolumeActionEndpoint::Resize(_) => "resize_volume",
    },
    matches!(
        endpoint,
        VolumeActionEndpoint::ChangeProtection(_) | VolumeActionEndpoint::Detach(_)
    ),
    if matches!(endpoint, VolumeActionEndpoint::Resize(_)) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

const fn server_action_requires_body(kind: ServerActionKind) -> bool {
    !matches!(
        kind,
        ServerActionKind::DetachIso
            | ServerActionKind::DisableBackup
            | ServerActionKind::DisableRescue
            | ServerActionKind::EnableBackup
            | ServerActionKind::Poweroff
            | ServerActionKind::Poweron
            | ServerActionKind::Reboot
            | ServerActionKind::RemoveFromPlacementGroup
            | ServerActionKind::RequestConsole
            | ServerActionKind::Reset
            | ServerActionKind::ResetPassword
            | ServerActionKind::Shutdown
    )
}
