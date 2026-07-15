//! Firewall, Load Balancer, network, and public-IP endpoint adapters.

use cloud_sdk::operation::CostIntent;

use crate::cloud::firewalls::actions::FirewallActionEndpoint;
use crate::cloud::firewalls::{FirewallEndpoint, FirewallListRequest};
use crate::cloud::load_balancers::{
    LoadBalancerActionEndpoint, LoadBalancerEndpoint, LoadBalancerListRequest,
    LoadBalancerMetricsRequest,
};
use crate::cloud::networks::actions::NetworkActionEndpoint;
use crate::cloud::networks::floating_ips::{
    FloatingIpActionEndpoint, FloatingIpEndpoint, FloatingIpListRequest,
};
use crate::cloud::networks::primary_ips::{
    PrimaryIpActionEndpoint, PrimaryIpEndpoint, PrimaryIpListRequest,
};
use crate::cloud::networks::resources::{NetworkEndpoint, NetworkListRequest};

use super::super::{RequestShape, ResponseProfile};

endpoint_wire!(
    FirewallEndpoint,
    endpoint => match endpoint {
        FirewallEndpoint::List => RequestShape::OptionalQuery,
        FirewallEndpoint::Create | FirewallEndpoint::Update(_) => RequestShape::RequiredJson,
        FirewallEndpoint::Get(_) | FirewallEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        FirewallEndpoint::Create => ResponseProfile::JsonCreated,
        FirewallEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        FirewallEndpoint::List => "list_firewalls",
        FirewallEndpoint::Create => "create_firewall",
        FirewallEndpoint::Get(_) => "get_firewall",
        FirewallEndpoint::Update(_) => "update_firewall",
        FirewallEndpoint::Delete(_) => "delete_firewall",
    },
    matches!(endpoint, FirewallEndpoint::Delete(_)),
    CostIntent::NoKnownCost
);

query_wire!(FirewallListRequest<'_>, request => {
    let _ = request;
    FirewallEndpoint::List
});

endpoint_wire!(
    FirewallActionEndpoint,
    endpoint => match endpoint {
        FirewallActionEndpoint::ListAll | FirewallActionEndpoint::ListForFirewall(_) => {
            RequestShape::OptionalQuery
        }
        FirewallActionEndpoint::Get(_) => RequestShape::None,
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        FirewallActionEndpoint::ListAll
        | FirewallActionEndpoint::Get(_)
        | FirewallActionEndpoint::ListForFirewall(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        FirewallActionEndpoint::ListAll => "list_firewalls_actions",
        FirewallActionEndpoint::Get(_) => "get_firewalls_action",
        FirewallActionEndpoint::ListForFirewall(_) => "list_firewall_actions",
        FirewallActionEndpoint::ApplyToResources(_) => "apply_firewall_to_resources",
        FirewallActionEndpoint::RemoveFromResources(_) => "remove_firewall_from_resources",
        FirewallActionEndpoint::SetRules(_) => "set_firewall_rules",
    },
    matches!(endpoint, FirewallActionEndpoint::RemoveFromResources(_)),
    CostIntent::NoKnownCost
);

endpoint_wire!(
    LoadBalancerEndpoint,
    endpoint => match endpoint {
        LoadBalancerEndpoint::List => RequestShape::OptionalQuery,
        LoadBalancerEndpoint::Create | LoadBalancerEndpoint::Update(_) => RequestShape::RequiredJson,
        LoadBalancerEndpoint::Metrics(_) => RequestShape::RequiredQuery,
        LoadBalancerEndpoint::Get(_) | LoadBalancerEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        LoadBalancerEndpoint::Create => ResponseProfile::JsonCreated,
        LoadBalancerEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        LoadBalancerEndpoint::List => "list_load_balancers",
        LoadBalancerEndpoint::Create => "create_load_balancer",
        LoadBalancerEndpoint::Get(_) => "get_load_balancer",
        LoadBalancerEndpoint::Update(_) => "update_load_balancer",
        LoadBalancerEndpoint::Delete(_) => "delete_load_balancer",
        LoadBalancerEndpoint::Metrics(_) => "get_load_balancer_metrics",
    },
    matches!(endpoint, LoadBalancerEndpoint::Delete(_)),
    if matches!(endpoint, LoadBalancerEndpoint::Create) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

query_wire!(LoadBalancerListRequest<'_>, request => {
    let _ = request;
    LoadBalancerEndpoint::List
});
query_wire!(LoadBalancerMetricsRequest<'_>, request => request.endpoint());

endpoint_wire!(
    LoadBalancerActionEndpoint,
    endpoint => match endpoint {
        LoadBalancerActionEndpoint::ListAll
        | LoadBalancerActionEndpoint::ListForLoadBalancer(_) => RequestShape::OptionalQuery,
        LoadBalancerActionEndpoint::Get(_)
        | LoadBalancerActionEndpoint::DisablePublicInterface(_)
        | LoadBalancerActionEndpoint::EnablePublicInterface(_) => RequestShape::None,
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        LoadBalancerActionEndpoint::ListAll
        | LoadBalancerActionEndpoint::Get(_)
        | LoadBalancerActionEndpoint::ListForLoadBalancer(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    load_balancer_action_key(endpoint),
    matches!(
        endpoint,
        LoadBalancerActionEndpoint::DeleteService(_)
            | LoadBalancerActionEndpoint::DetachFromNetwork(_)
            | LoadBalancerActionEndpoint::DisablePublicInterface(_)
            | LoadBalancerActionEndpoint::RemoveTarget(_)
    ),
    if matches!(endpoint, LoadBalancerActionEndpoint::ChangeType(_)) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

endpoint_wire!(
    NetworkEndpoint,
    endpoint => match endpoint {
        NetworkEndpoint::List => RequestShape::OptionalQuery,
        NetworkEndpoint::Create | NetworkEndpoint::Update(_) => RequestShape::RequiredJson,
        NetworkEndpoint::Get(_) | NetworkEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        NetworkEndpoint::Create => ResponseProfile::JsonCreated,
        NetworkEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        NetworkEndpoint::List => "list_networks",
        NetworkEndpoint::Create => "create_network",
        NetworkEndpoint::Get(_) => "get_network",
        NetworkEndpoint::Update(_) => "update_network",
        NetworkEndpoint::Delete(_) => "delete_network",
    },
    matches!(endpoint, NetworkEndpoint::Delete(_)),
    CostIntent::NoKnownCost
);

query_wire!(NetworkListRequest<'_>, request => {
    let _ = request;
    NetworkEndpoint::List
});

endpoint_wire!(
    NetworkActionEndpoint,
    endpoint => match endpoint {
        NetworkActionEndpoint::ListAll | NetworkActionEndpoint::ListForNetwork(_) => {
            RequestShape::OptionalQuery
        }
        NetworkActionEndpoint::Get(_) => RequestShape::None,
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        NetworkActionEndpoint::ListAll
        | NetworkActionEndpoint::Get(_)
        | NetworkActionEndpoint::ListForNetwork(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        NetworkActionEndpoint::ListAll => "list_networks_actions",
        NetworkActionEndpoint::Get(_) => "get_networks_action",
        NetworkActionEndpoint::ListForNetwork(_) => "list_network_actions",
        NetworkActionEndpoint::AddRoute(_) => "add_network_route",
        NetworkActionEndpoint::AddSubnet(_) => "add_network_subnet",
        NetworkActionEndpoint::ChangeIpRange(_) => "change_network_ip_range",
        NetworkActionEndpoint::ChangeProtection(_) => "change_network_protection",
        NetworkActionEndpoint::DeleteRoute(_) => "delete_network_route",
        NetworkActionEndpoint::DeleteSubnet(_) => "delete_network_subnet",
    },
    matches!(
        endpoint,
        NetworkActionEndpoint::DeleteRoute(_) | NetworkActionEndpoint::DeleteSubnet(_)
    ),
    CostIntent::NoKnownCost
);

endpoint_wire!(
    FloatingIpEndpoint,
    endpoint => match endpoint {
        FloatingIpEndpoint::List => RequestShape::OptionalQuery,
        FloatingIpEndpoint::Create | FloatingIpEndpoint::Update(_) => RequestShape::RequiredJson,
        FloatingIpEndpoint::Get(_) | FloatingIpEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        FloatingIpEndpoint::Create => ResponseProfile::JsonCreated,
        FloatingIpEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        FloatingIpEndpoint::List => "list_floating_ips",
        FloatingIpEndpoint::Create => "create_floating_ip",
        FloatingIpEndpoint::Get(_) => "get_floating_ip",
        FloatingIpEndpoint::Update(_) => "update_floating_ip",
        FloatingIpEndpoint::Delete(_) => "delete_floating_ip",
    },
    matches!(endpoint, FloatingIpEndpoint::Delete(_)),
    if matches!(endpoint, FloatingIpEndpoint::Create) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

query_wire!(FloatingIpListRequest<'_>, request => {
    let _ = request;
    FloatingIpEndpoint::List
});

endpoint_wire!(
    FloatingIpActionEndpoint,
    endpoint => match endpoint {
        FloatingIpActionEndpoint::ListAll
        | FloatingIpActionEndpoint::ListForFloatingIp(_) => RequestShape::OptionalQuery,
        FloatingIpActionEndpoint::Get(_) | FloatingIpActionEndpoint::Unassign(_) => {
            RequestShape::None
        }
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        FloatingIpActionEndpoint::ListAll
        | FloatingIpActionEndpoint::Get(_)
        | FloatingIpActionEndpoint::ListForFloatingIp(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        FloatingIpActionEndpoint::ListAll => "list_floating_ips_actions",
        FloatingIpActionEndpoint::Get(_) => "get_floating_ips_action",
        FloatingIpActionEndpoint::ListForFloatingIp(_) => "list_floating_ip_actions",
        FloatingIpActionEndpoint::Assign(_) => "assign_floating_ip",
        FloatingIpActionEndpoint::ChangeDnsPtr(_) => "change_floating_ip_dns_ptr",
        FloatingIpActionEndpoint::ChangeProtection(_) => "change_floating_ip_protection",
        FloatingIpActionEndpoint::Unassign(_) => "unassign_floating_ip",
    },
    matches!(endpoint, FloatingIpActionEndpoint::Unassign(_)),
    CostIntent::NoKnownCost
);

endpoint_wire!(
    PrimaryIpEndpoint,
    endpoint => match endpoint {
        PrimaryIpEndpoint::List => RequestShape::OptionalQuery,
        PrimaryIpEndpoint::Create | PrimaryIpEndpoint::Update(_) => RequestShape::RequiredJson,
        PrimaryIpEndpoint::Get(_) | PrimaryIpEndpoint::Delete(_) => RequestShape::None,
    },
    match endpoint {
        PrimaryIpEndpoint::Create => ResponseProfile::JsonCreated,
        PrimaryIpEndpoint::Delete(_) => ResponseProfile::NoContent,
        _ => ResponseProfile::JsonOk,
    },
    match endpoint {
        PrimaryIpEndpoint::List => "list_primary_ips",
        PrimaryIpEndpoint::Create => "create_primary_ip",
        PrimaryIpEndpoint::Get(_) => "get_primary_ip",
        PrimaryIpEndpoint::Update(_) => "update_primary_ip",
        PrimaryIpEndpoint::Delete(_) => "delete_primary_ip",
    },
    matches!(endpoint, PrimaryIpEndpoint::Delete(_)),
    if matches!(endpoint, PrimaryIpEndpoint::Create) {
        CostIntent::MayIncurCost
    } else {
        CostIntent::NoKnownCost
    }
);

query_wire!(PrimaryIpListRequest<'_>, request => {
    let _ = request;
    PrimaryIpEndpoint::List
});

endpoint_wire!(
    PrimaryIpActionEndpoint,
    endpoint => match endpoint {
        PrimaryIpActionEndpoint::ListAll | PrimaryIpActionEndpoint::ListForPrimaryIp(_) => {
            RequestShape::OptionalQuery
        }
        PrimaryIpActionEndpoint::Get(_) | PrimaryIpActionEndpoint::Unassign(_) => {
            RequestShape::None
        }
        _ => RequestShape::RequiredJson,
    },
    match endpoint {
        PrimaryIpActionEndpoint::ListAll
        | PrimaryIpActionEndpoint::Get(_)
        | PrimaryIpActionEndpoint::ListForPrimaryIp(_) => ResponseProfile::JsonOk,
        _ => ResponseProfile::JsonCreated,
    },
    match endpoint {
        PrimaryIpActionEndpoint::ListAll => "list_primary_ips_actions",
        PrimaryIpActionEndpoint::Get(_) => "get_primary_ips_action",
        PrimaryIpActionEndpoint::ListForPrimaryIp(_) => "list_primary_ip_actions",
        PrimaryIpActionEndpoint::Assign(_) => "assign_primary_ip",
        PrimaryIpActionEndpoint::ChangeDnsPtr(_) => "change_primary_ip_dns_ptr",
        PrimaryIpActionEndpoint::ChangeProtection(_) => "change_primary_ip_protection",
        PrimaryIpActionEndpoint::Unassign(_) => "unassign_primary_ip",
    },
    matches!(endpoint, PrimaryIpActionEndpoint::Unassign(_)),
    CostIntent::NoKnownCost
);

const fn load_balancer_action_key(endpoint: LoadBalancerActionEndpoint) -> &'static str {
    match endpoint {
        LoadBalancerActionEndpoint::ListAll => "list_load_balancers_actions",
        LoadBalancerActionEndpoint::Get(_) => "get_load_balancers_action",
        LoadBalancerActionEndpoint::ListForLoadBalancer(_) => "list_load_balancer_actions",
        LoadBalancerActionEndpoint::AddService(_) => "add_load_balancer_service",
        LoadBalancerActionEndpoint::AddTarget(_) => "add_load_balancer_target",
        LoadBalancerActionEndpoint::AttachToNetwork(_) => "attach_load_balancer_to_network",
        LoadBalancerActionEndpoint::ChangeAlgorithm(_) => "change_load_balancer_algorithm",
        LoadBalancerActionEndpoint::ChangeDnsPtr(_) => "change_load_balancer_dns_ptr",
        LoadBalancerActionEndpoint::ChangeProtection(_) => "change_load_balancer_protection",
        LoadBalancerActionEndpoint::ChangeType(_) => "change_load_balancer_type",
        LoadBalancerActionEndpoint::DeleteService(_) => "delete_load_balancer_service",
        LoadBalancerActionEndpoint::DetachFromNetwork(_) => "detach_load_balancer_from_network",
        LoadBalancerActionEndpoint::DisablePublicInterface(_) => {
            "disable_load_balancer_public_interface"
        }
        LoadBalancerActionEndpoint::EnablePublicInterface(_) => {
            "enable_load_balancer_public_interface"
        }
        LoadBalancerActionEndpoint::RemoveTarget(_) => "remove_load_balancer_target",
        LoadBalancerActionEndpoint::UpdateService(_) => "update_load_balancer_service",
    }
}
