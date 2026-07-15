//! Firewall JSON bodies.

use crate::cloud::firewalls::actions::{
    FirewallResourceIntent, FirewallResourcesRequest, FirewallSetRulesRequest,
};
use crate::cloud::firewalls::rules::{
    FirewallProtocol, FirewallRule, FirewallRuleSet, FirewallSelectors,
};
use crate::cloud::firewalls::{
    FirewallCreateRequest, FirewallEndpoint, FirewallResource, FirewallUpdateRequest,
};
use crate::prepared::{HetznerPreparationError, JsonWriter};

body_wire!(FirewallCreateRequest<'_>, request => FirewallEndpoint::Create, "create_firewall", write_create);
body_wire!(FirewallUpdateRequest<'_>, request => request.endpoint(), "update_firewall", write_update);
body_component!(
    FirewallSetRulesRequest<'_>,
    "set_firewall_rules",
    write_set_rules
);

impl crate::prepared::BodyWire for FirewallResourcesRequest<'_> {
    fn write_body(self, output: &mut [u8]) -> Result<usize, HetznerPreparationError> {
        write_resources_request(self, output)
    }

    fn operation_key(self) -> &'static str {
        match self.intent() {
            FirewallResourceIntent::Apply => "apply_firewall_to_resources",
            FirewallResourceIntent::Remove => "remove_firewall_from_resources",
        }
    }
}

fn write_create(
    request: FirewallCreateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(resources) = request.resources() {
            write_resources_field(writer, first, "apply_to", resources)?;
        }
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        writer.field_string(first, "name", request.name().as_str())?;
        if let Some(rules) = request.rules() {
            write_rules_field(writer, first, rules)?;
        }
        Ok(())
    })
}

fn write_update(
    request: FirewallUpdateRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        if let Some(labels) = request.labels() {
            writer.field_labels(first, "labels", labels)?;
        }
        if let Some(name) = request.name() {
            writer.field_string(first, "name", name.as_str())?;
        }
        Ok(())
    })
}

fn write_resources_request(
    request: FirewallResourcesRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    let field = match request.intent() {
        FirewallResourceIntent::Apply => "apply_to",
        FirewallResourceIntent::Remove => "remove_from",
    };
    object(output, |writer, first| {
        write_resources_field(writer, first, field, request.resources())
    })
}

fn write_set_rules(
    request: FirewallSetRulesRequest<'_>,
    output: &mut [u8],
) -> Result<usize, HetznerPreparationError> {
    object(output, |writer, first| {
        write_rules_field(writer, first, request.rules())
    })
}

fn write_resources_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    name: &str,
    resources: &[FirewallResource<'_>],
) -> Result<(), HetznerPreparationError> {
    writer.field(first, name)?;
    writer.begin_array()?;
    let mut item = true;
    for resource in resources {
        writer.value(&mut item)?;
        write_resource(writer, *resource)?;
    }
    writer.end_array()
}

fn write_resource(
    writer: &mut JsonWriter<'_>,
    resource: FirewallResource<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    let mut first = true;
    match resource {
        FirewallResource::Server(id) => {
            writer.field(&mut first, "server")?;
            writer.begin_object()?;
            let mut nested = true;
            writer.field_u64(&mut nested, "id", id.get())?;
            writer.end_object()?;
            writer.field_string(&mut first, "type", "server")?;
        }
        FirewallResource::LabelSelector(selector) => {
            writer.field(&mut first, "label_selector")?;
            writer.begin_object()?;
            let mut nested = true;
            writer.field_string(&mut nested, "selector", selector.as_str())?;
            writer.end_object()?;
            writer.field_string(&mut first, "type", "label_selector")?;
        }
    }
    writer.end_object()
}

fn write_rules_field(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    rules: FirewallRuleSet<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.field(first, "rules")?;
    writer.begin_array()?;
    let mut item = true;
    for rule in rules.rules() {
        writer.value(&mut item)?;
        write_rule(writer, *rule)?;
    }
    writer.end_array()
}

fn write_rule(
    writer: &mut JsonWriter<'_>,
    rule: FirewallRule<'_>,
) -> Result<(), HetznerPreparationError> {
    writer.begin_object()?;
    let mut first = true;
    if let Some(description) = rule.description() {
        writer.field_string(&mut first, "description", description.as_str())?;
    }
    match rule.selectors() {
        FirewallSelectors::Incoming(cidrs) => {
            writer.field_string(&mut first, "direction", "in")?;
            write_cidrs(writer, &mut first, "source_ips", cidrs)?;
        }
        FirewallSelectors::Outgoing(cidrs) => {
            writer.field_string(&mut first, "direction", "out")?;
            write_cidrs(writer, &mut first, "destination_ips", cidrs)?;
        }
    }
    if let Some(port) = rule.port() {
        writer.field_string(&mut first, "port", port.as_str())?;
    }
    writer.field_string(&mut first, "protocol", protocol(rule.protocol()))?;
    writer.end_object()
}

fn write_cidrs(
    writer: &mut JsonWriter<'_>,
    first: &mut bool,
    name: &str,
    cidrs: &[crate::cloud::ip::IpCidr<'_>],
) -> Result<(), HetznerPreparationError> {
    writer.field(first, name)?;
    writer.begin_array()?;
    let mut item = true;
    for cidr in cidrs {
        writer.value(&mut item)?;
        writer.string(cidr.as_str())?;
    }
    writer.end_array()
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

const fn protocol(value: FirewallProtocol) -> &'static str {
    match value {
        FirewallProtocol::Tcp => "tcp",
        FirewallProtocol::Udp => "udp",
        FirewallProtocol::Icmp => "icmp",
        FirewallProtocol::Esp => "esp",
        FirewallProtocol::Gre => "gre",
    }
}
