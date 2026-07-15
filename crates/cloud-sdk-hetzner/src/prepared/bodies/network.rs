//! Network, Floating IP, and Primary IP JSON bodies.

use crate::cloud::networks::actions::{
    NetworkAddSubnetRequest, NetworkChangeIpRangeRequest, NetworkDeleteSubnetRequest,
    NetworkProtectionRequest, NetworkRouteRequest,
};
use crate::cloud::networks::floating_ips::{
    FloatingIpAssignRequest, FloatingIpChangeDnsPtrRequest, FloatingIpCreatePlacement,
    FloatingIpCreateRequest, FloatingIpDnsPtrIntent, FloatingIpEndpoint,
    FloatingIpProtectionRequest, FloatingIpType, FloatingIpUpdateRequest,
};
use crate::cloud::networks::primary_ips::{
    PrimaryIpAssignRequest, PrimaryIpChangeDnsPtrRequest, PrimaryIpCreateRequest,
    PrimaryIpDnsPtrIntent, PrimaryIpEndpoint, PrimaryIpProtectionRequest, PrimaryIpType,
    PrimaryIpUpdateRequest,
};
use crate::cloud::networks::resources::{
    NetworkCreateRequest, NetworkRoute, NetworkSubnet, NetworkSubnetType, NetworkUpdateRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(NetworkCreateRequest<'_>, request => crate::cloud::networks::resources::NetworkEndpoint::Create, "create_network", write_network_create);
body_wire!(NetworkUpdateRequest<'_>, request => request.endpoint(), "update_network", write_network_update);
body_component!(
    NetworkAddSubnetRequest<'_>,
    "add_network_subnet",
    write_subnet_request
);
body_component!(
    NetworkChangeIpRangeRequest<'_>,
    "change_network_ip_range",
    write_change_range
);
body_component!(
    NetworkProtectionRequest,
    "change_network_protection",
    write_network_protection
);

body_wire!(FloatingIpCreateRequest<'_>, request => FloatingIpEndpoint::Create, "create_floating_ip", write_floating_create);
body_wire!(FloatingIpUpdateRequest<'_>, request => request.endpoint(), "update_floating_ip", write_floating_update);
body_component!(
    FloatingIpAssignRequest,
    "assign_floating_ip",
    write_floating_assign
);
body_component!(
    FloatingIpChangeDnsPtrRequest<'_>,
    "change_floating_ip_dns_ptr",
    write_floating_dns
);
body_component!(
    FloatingIpProtectionRequest,
    "change_floating_ip_protection",
    write_floating_protection
);

body_wire!(PrimaryIpCreateRequest<'_>, request => PrimaryIpEndpoint::Create, "create_primary_ip", write_primary_create);
body_wire!(PrimaryIpUpdateRequest<'_>, request => request.endpoint(), "update_primary_ip", write_primary_update);
body_component!(
    PrimaryIpAssignRequest,
    "assign_primary_ip",
    write_primary_assign
);
body_component!(
    PrimaryIpChangeDnsPtrRequest<'_>,
    "change_primary_ip_dns_ptr",
    write_primary_dns
);
body_component!(
    PrimaryIpProtectionRequest,
    "change_primary_ip_protection",
    write_primary_protection
);

// The same route shape is used by add and delete operations.
impl crate::prepared::BodyWire for NetworkDeleteSubnetRequest<'_> {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        object(output, |writer, first| {
            writer.field_string(first, "ip_range", self.ip_range().as_str())
        })
    }

    fn operation_key(self) -> &'static str {
        "delete_network_subnet"
    }
}

impl crate::prepared::BodyWire for NetworkRouteRequest<'_> {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        write_route_request(self, output)
    }

    fn operation_key(self) -> &'static str {
        "add_network_route"
    }

    fn accepts_operation(self, operation_key: &str) -> bool {
        match operation_key {
            "add_network_route" | "delete_network_route" => true,
            _ => false,
        }
    }
}

fn write_network_create(
    request: NetworkCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(
            first,
            "expose_routes_to_vswitch",
            request.expose_routes_to_vswitch(),
        )?;
        writer.field_string(first, "ip_range", request.ip_range().as_str())?;
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        writer.field_string(first, "name", request.name().as_str())?;
        if let Some(routes) = request.routes() {
            writer.field(first, "routes")?;
            writer.begin_array()?;
            let mut item = true;
            for route in routes {
                writer.value(&mut item)?;
                write_route(writer, *route)?;
            }
            writer.end_array()?;
        }
        if let Some(subnets) = request.subnets() {
            writer.field(first, "subnets")?;
            writer.begin_array()?;
            let mut item = true;
            for subnet in subnets {
                writer.value(&mut item)?;
                write_subnet(writer, *subnet)?;
            }
            writer.end_array()?;
        }
        Ok(())
    })
}

fn write_network_update(
    request: NetworkUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(expose) = request.expose_routes_to_vswitch() {
            writer.field_bool(first, "expose_routes_to_vswitch", expose)?;
        }
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        if let Some(name) = request.name() {
            writer.field_string(first, "name", name.as_str())?;
        }
        Ok(())
    })
}

fn write_route_request(
    request: NetworkRouteRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_route_fields(writer, request.route())
    })
}

fn write_route(
    writer: &mut JsonWriter<'_>,
    route: NetworkRoute<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    write_route_fields(writer, route)?;
    writer.end_object()
}

fn write_route_fields(
    writer: &mut JsonWriter<'_>,
    route: NetworkRoute<'_>,
) -> Result<(), HetznerPreparationError> {
    let mut first = true;
    writer.field_string(&mut first, "destination", route.destination().as_str())?;
    writer.field_string(&mut first, "gateway", route.gateway().as_str())
}

fn write_subnet_request(
    request: NetworkAddSubnetRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, _| {
        write_subnet_fields(writer, request.subnet())
    })
}

fn write_subnet(
    writer: &mut JsonWriter<'_>,
    subnet: NetworkSubnet<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    write_subnet_fields(writer, subnet)?;
    writer.end_object()
}

fn write_subnet_fields(
    writer: &mut JsonWriter<'_>,
    subnet: NetworkSubnet<'_>,
) -> Result<(), HetznerPreparationError> {
    let mut first = true;
    if let Some(ip_range) = subnet.ip_range() {
        writer.field_string(&mut first, "ip_range", ip_range.as_str())?;
    }
    writer.field_string(&mut first, "network_zone", subnet.network_zone().as_str())?;
    writer.field_string(&mut first, "type", subnet_type(subnet.subnet_type()))?;
    if let Some(vswitch) = subnet.vswitch_id() {
        writer.field_u64(&mut first, "vswitch_id", vswitch.get())?;
    }
    Ok(())
}

fn write_change_range(
    request: NetworkChangeIpRangeRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_string(first, "ip_range", request.ip_range().as_str())
    })
}

fn write_network_protection(
    request: NetworkProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_floating_create(
    request: FloatingIpCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(description) = request.description {
            writer.field_string(first, "description", description.as_str())?;
        }
        match request.placement() {
            FloatingIpCreatePlacement::Server(server) => {
                writer.field_u64(first, "server", server.get())?
            }
            FloatingIpCreatePlacement::HomeLocation(location) => {
                writer.field_string(first, "home_location", location.as_str())?
            }
        }
        if let Some(labels) = request.labels {
            writer.field_labels(first, "labels", labels)?;
        }
        if let Some(name) = request.name {
            writer.field_string(first, "name", name.as_str())?;
        }
        writer.field_string(first, "type", floating_type(request.ip_type()))
    })
}

fn write_floating_update(
    request: FloatingIpUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_resource_update(
        output,
        request.description.map(|v| v.as_str()),
        request.labels,
        request.name.map(|v| v.as_str()),
    )
}

fn write_floating_assign(
    request: FloatingIpAssignRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "server", request.server().get())
    })
}

fn write_floating_dns(
    request: FloatingIpChangeDnsPtrRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let value = match request.dns_ptr() {
            FloatingIpDnsPtrIntent::Set(value) => Some(value.as_str()),
            FloatingIpDnsPtrIntent::Reset => None,
        };
        write_dns_ptr(writer, first, value)?;
        writer.field_string(first, "ip", request.ip().as_str())
    })
}

fn write_floating_protection(
    request: FloatingIpProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
    })
}

fn write_primary_create(
    request: PrimaryIpCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(id) = request.assignee_id {
            writer.field_u64(first, "assignee_id", id.get())?;
            writer.field_string(first, "assignee_type", "server")?;
        }
        writer.field_bool(first, "auto_delete", request.auto_delete)?;
        if let Some(labels) = request.labels {
            writer.field_labels(first, "labels", labels)?;
        }
        if let Some(location) = request.location {
            writer.field_string(first, "location", location.as_str())?;
        }
        if let Some(name) = request.name {
            writer.field_string(first, "name", name.as_str())?;
        }
        writer.field_string(first, "type", primary_type(request.ip_type()))
    })
}

fn write_primary_update(
    request: PrimaryIpUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    write_resource_update(
        output,
        None,
        request.labels,
        request.name.map(|v| v.as_str()),
    )
}

fn write_primary_assign(
    request: PrimaryIpAssignRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_u64(first, "assignee_id", request.assignee_id().get())?;
        writer.field_string(first, "assignee_type", "server")
    })
}

fn write_primary_dns(
    request: PrimaryIpChangeDnsPtrRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        let value = match request.dns_ptr() {
            PrimaryIpDnsPtrIntent::Set(value) => Some(value.as_str()),
            PrimaryIpDnsPtrIntent::Reset => None,
        };
        write_dns_ptr(writer, first, value)?;
        writer.field_string(first, "ip", request.ip().as_str())
    })
}

fn write_primary_protection(
    request: PrimaryIpProtectionRequest,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        writer.field_bool(first, "delete", request.delete())
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

fn write_resource_update(
    output: &mut [u8],
    description: Option<&str>,
    labels: Option<crate::cloud::shared::CloudLabels<'_>>,
    name: Option<&str>,
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(value) = description {
            writer.field_string(first, "description", value)?;
        }
        if let Some(value) = labels {
            writer.field_labels(first, "labels", value)?;
        }
        if let Some(value) = name {
            writer.field_string(first, "name", value)?;
        }
        Ok(())
    })
}

fn write_dns_ptr(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    value: Option<&str>,
) -> Result<(), HetznerPreparationError> {
    match value {
        Some(value) => writer.field_string(first, "dns_ptr", value),
        None => writer.field_null(first, "dns_ptr"),
    }
}

const fn floating_type(value: FloatingIpType) -> &'static str {
    match value {
        FloatingIpType::Ipv4 => "ipv4",
        FloatingIpType::Ipv6 => "ipv6",
    }
}
const fn primary_type(value: PrimaryIpType) -> &'static str {
    match value {
        PrimaryIpType::Ipv4 => "ipv4",
        PrimaryIpType::Ipv6 => "ipv6",
    }
}
const fn subnet_type(value: NetworkSubnetType) -> &'static str {
    match value {
        NetworkSubnetType::Cloud => "cloud",
        NetworkSubnetType::Vswitch => "vswitch",
    }
}
