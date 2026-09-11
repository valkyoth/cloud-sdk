use super::schema_table::NODES;
use crate::discovery::{DiscoveryError as Error, DiscoveryValue as Value, Timestamp};

pub(super) enum Node {
    Any,
    Null,
    Bool,
    Integer(u64),
    Text,
    Timestamp,
    Enum(&'static [&'static str]),
    Array(usize),
    Nullable(usize),
    OneOf(&'static [usize]),
    Object(&'static [(&'static str, usize, bool)], Option<usize>),
}
pub(super) fn validate(value: &Value, id: usize, depth: usize) -> Result<(), Error> {
    if depth > 24 {
        return Err(Error::Limit);
    }
    let next = depth.checked_add(1).ok_or(Error::Limit)?;
    match NODES.get(id).ok_or(Error::Schema)? {
        Node::Any => (),
        Node::Null if value.is_null() => (),
        Node::Null => return Err(Error::Schema),
        Node::Bool => {
            value.boolean()?;
        }
        Node::Integer(maximum) => {
            value.count(*maximum)?;
        }
        Node::Text => {
            value.with_text(|_| ())?;
        }
        Node::Timestamp => {
            Timestamp::parse(value)?;
        }
        Node::Enum(allowed) => {
            if !value.with_text(|v| allowed.contains(&v))? {
                return Err(Error::Value);
            }
        }
        Node::Array(child) => {
            for entry in value.array()? {
                validate(entry, *child, next)?;
            }
        }
        Node::Nullable(child) => {
            if !value.is_null() {
                validate(value, *child, next)?;
            }
        }
        Node::OneOf(choices) => {
            let mut matches = 0_usize;
            for child in *choices {
                match validate(value, *child, next) {
                    Ok(()) => matches = matches.checked_add(1).ok_or(Error::Limit)?,
                    Err(Error::Allocation | Error::Limit) => return Err(Error::Limit),
                    Err(_) => (),
                }
            }
            if matches != 1 {
                return Err(Error::Schema);
            }
        }
        Node::Object(fields, extra) => {
            value.object()?;
            for (name, child, required) in *fields {
                match value.get(name)? {
                    Some(v) => validate(v, *child, next)?,
                    None if *required => return Err(Error::Schema),
                    None => (),
                }
            }
            if let Some(child) = extra {
                value.visit_fields(|name, v| {
                    if !fields.iter().any(|(key, _, _)| *key == name) {
                        validate(v, *child, next)?;
                    }
                    Ok(())
                })?;
            }
        }
    }
    Ok(())
}
