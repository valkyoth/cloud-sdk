use alloc::{borrow::Cow, format, string::String, vec::Vec};

use super::prepare::Kind;
use super::request::{RobotFirewallReplaceIntent, RobotFirewallRequestError};
use super::value::{RobotFirewallRule, RobotFirewallRules};
use crate::robot::{RobotForm, RobotFormField};

pub(super) fn write_form(
    kind: Kind<'_>,
    output: &mut [u8],
) -> Result<usize, RobotFirewallRequestError> {
    let mut values: Vec<(String, Cow<'_, str>)> = Vec::new();
    match kind {
        Kind::Replace(
            _,
            RobotFirewallReplaceIntent::Inline {
                status,
                filter_ipv6,
                whitelist_hos,
                rules,
            },
        ) => {
            push(&mut values, "status", status.as_str())?;
            push_optional_bool(&mut values, "filter_ipv6", filter_ipv6)?;
            push(&mut values, "whitelist_hos", bool_text(whitelist_hos))?;
            push_rules(&mut values, rules)?;
        }
        Kind::Replace(
            _,
            RobotFirewallReplaceIntent::Template {
                status,
                filter_ipv6,
                template_id,
            },
        ) => {
            push(&mut values, "status", status.as_str())?;
            push_optional_bool(&mut values, "filter_ipv6", filter_ipv6)?;
            push_owned(&mut values, "template_id", format!("{}", template_id.get()))?;
        }
        Kind::TemplateCreate(config) | Kind::TemplateUpdate(_, config) => {
            let (name, filter_ipv6, whitelist_hos, is_default, rules) = config.parts();
            push(&mut values, "name", name.as_str())?;
            push_optional_bool(&mut values, "filter_ipv6", filter_ipv6)?;
            push(&mut values, "whitelist_hos", bool_text(whitelist_hos))?;
            push(&mut values, "is_default", bool_text(is_default))?;
            push_rules(&mut values, rules)?;
        }
        Kind::Get(_)
        | Kind::Delete(_)
        | Kind::TemplateList
        | Kind::TemplateGet(_)
        | Kind::TemplateDelete(_) => return Ok(0),
    }
    let mut fields = Vec::new();
    fields
        .try_reserve_exact(values.len())
        .map_err(|_| RobotFirewallRequestError::Allocation)?;
    for (name, value) in &values {
        fields.push(
            RobotFormField::sensitive(name, value.as_ref())
                .map_err(RobotFirewallRequestError::Form)?,
        );
    }
    RobotForm::new(&fields)
        .and_then(|form| form.write_prepared(output))
        .map_err(RobotFirewallRequestError::Form)
}

fn push_rules<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    rules: RobotFirewallRules<'a>,
) -> Result<(), RobotFirewallRequestError> {
    for (direction, rules) in [("input", rules.input()), ("output", rules.output())] {
        for (index, rule) in rules.iter().copied().enumerate() {
            push_rule(values, direction, index, rule)?;
        }
    }
    Ok(())
}

fn push_rule<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    direction: &str,
    index: usize,
    rule: RobotFirewallRule<'a>,
) -> Result<(), RobotFirewallRequestError> {
    let fields = rule
        .validate()
        .map_err(RobotFirewallRequestError::Rule)?
        .fields();
    macro_rules! optional {
        ($field:literal, $value:expr) => {
            if let Some(value) = $value {
                push_dynamic(values, direction, index, $field, value)?;
            }
        };
    }
    optional!("name", fields.name);
    optional!("ip_version", fields.ip_version.map(|value| value.as_str()));
    optional!("dst_ip", fields.destination_ip.map(|value| value.as_str()));
    optional!("src_ip", fields.source_ip.map(|value| value.as_str()));
    optional!(
        "dst_port",
        fields.destination_port.map(|value| value.as_str())
    );
    optional!("src_port", fields.source_port.map(|value| value.as_str()));
    optional!("protocol", fields.protocol.map(|value| value.as_str()));
    optional!("tcp_flags", fields.tcp_flags.map(|value| value.as_str()));
    push_dynamic(values, direction, index, "action", fields.action.as_str())
}

fn push_dynamic<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    direction: &str,
    index: usize,
    field: &str,
    value: &'a str,
) -> Result<(), RobotFirewallRequestError> {
    push_pair(
        values,
        format!("rules[{direction}][{index}][{field}]"),
        Cow::Borrowed(value),
    )
}

fn push<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    name: &str,
    value: &'a str,
) -> Result<(), RobotFirewallRequestError> {
    push_pair(values, String::from(name), Cow::Borrowed(value))
}

fn push_owned<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    name: &str,
    value: String,
) -> Result<(), RobotFirewallRequestError> {
    push_pair(values, String::from(name), Cow::Owned(value))
}

fn push_pair<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    name: String,
    value: Cow<'a, str>,
) -> Result<(), RobotFirewallRequestError> {
    values
        .try_reserve(1)
        .map_err(|_| RobotFirewallRequestError::Allocation)?;
    values.push((name, value));
    Ok(())
}

fn push_optional_bool<'a>(
    values: &mut Vec<(String, Cow<'a, str>)>,
    name: &str,
    value: Option<bool>,
) -> Result<(), RobotFirewallRequestError> {
    if let Some(value) = value {
        push(values, name, bool_text(value))?;
    }
    Ok(())
}

const fn bool_text(value: bool) -> &'static str {
    if value { "true" } else { "false" }
}
