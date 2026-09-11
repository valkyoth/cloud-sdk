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
            one_of(choices.iter().map(|child| validate(value, *child, next)))?;
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

fn one_of(results: impl Iterator<Item = Result<(), Error>>) -> Result<(), Error> {
    let mut matches = 0_usize;
    for result in results {
        match result {
            Ok(()) => matches = matches.checked_add(1).ok_or(Error::Limit)?,
            Err(error @ (Error::Allocation | Error::Limit)) => return Err(error),
            Err(_) => (),
        }
    }
    if matches != 1 {
        return Err(Error::Schema);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{Error, one_of};

    #[test]
    fn one_of_preserves_operational_errors_before_and_after_matches() {
        for error in [Error::Allocation, Error::Limit] {
            for results in [
                [Err(error), Ok(())],
                [Ok(()), Err(error)],
                [Err(Error::Schema), Err(error)],
            ] {
                assert_eq!(one_of(results.into_iter()), Err(error));
            }
            let mut branches = 0;
            let result = one_of((0..2).map(|_| {
                branches += 1;
                Err(error)
            }));
            assert_eq!(result, Err(error));
            assert_eq!(branches, 1);
        }
    }

    #[test]
    fn one_of_still_requires_exactly_one_successful_branch() {
        assert_eq!(one_of([Ok(())].into_iter()), Ok(()));
        assert_eq!(one_of([Err(Error::Value), Ok(())].into_iter()), Ok(()));
        assert_eq!(one_of([Ok(()), Err(Error::Schema)].into_iter()), Ok(()));
        assert_eq!(one_of([Ok(()), Ok(())].into_iter()), Err(Error::Schema));
        assert_eq!(one_of([Err(Error::Value)].into_iter()), Err(Error::Schema));
        assert_eq!(one_of(core::iter::empty()), Err(Error::Schema));
    }
}
