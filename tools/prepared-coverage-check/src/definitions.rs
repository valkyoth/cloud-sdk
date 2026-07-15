//! Source-locked adapter macro definition validation.

use std::collections::BTreeMap;

use syn::{Item, ItemMacro, MacroDelimiter};

use crate::inspect::RegistryKind;

const ENDPOINT_LOCK: &str = include_str!("../locks/endpoints.rs");
const BODY_LOCK: &str = include_str!("../locks/bodies.rs");

pub(crate) fn validate_adapter_definitions(
    items: &[Item],
    kind: RegistryKind,
    definition_root: bool,
) -> Result<(), String> {
    let definitions = macro_definitions(items)?;
    if !definition_root {
        if definitions.is_empty() {
            return Ok(());
        }
        return Err("prepared adapter macro shadowing is forbidden".to_owned());
    }

    let expected_source = match kind {
        RegistryKind::Endpoint => ENDPOINT_LOCK,
        RegistryKind::Body => BODY_LOCK,
    };
    let expected_file = syn::parse_file(expected_source)
        .map_err(|error| format!("invalid built-in adapter definition lock: {error}"))?;
    let expected = macro_definitions(&expected_file.items)?;
    if definitions.keys().ne(expected.keys()) {
        return Err(format!(
            "{} root must contain exactly the source-locked adapter definitions",
            kind.label()
        ));
    }
    for (name, definition) in definitions {
        let locked = expected
            .get(&name)
            .ok_or_else(|| "adapter definition lock is incomplete".to_owned())?;
        if !same_definition(definition, locked) {
            return Err(format!("{name} definition differs from its source lock"));
        }
    }
    Ok(())
}

fn macro_definitions(items: &[Item]) -> Result<BTreeMap<String, &ItemMacro>, String> {
    let mut definitions = BTreeMap::new();
    for item in items {
        let Item::Macro(item_macro) = item else {
            continue;
        };
        if !item_macro.mac.path.is_ident("macro_rules") {
            continue;
        }
        let Some(name) = item_macro.ident.as_ref().map(ToString::to_string) else {
            continue;
        };
        if !item_macro.attrs.is_empty() {
            return Err(format!("{name} definition attributes are forbidden"));
        }
        if definitions.insert(name.clone(), item_macro).is_some() {
            return Err(format!("duplicate {name} definition"));
        }
    }
    Ok(definitions)
}

fn same_definition(actual: &ItemMacro, expected: &ItemMacro) -> bool {
    actual.ident == expected.ident
        && same_delimiter(&actual.mac.delimiter, &expected.mac.delimiter)
        && actual.mac.tokens.to_string() == expected.mac.tokens.to_string()
        && actual.semi_token.is_some() == expected.semi_token.is_some()
}

const fn same_delimiter(left: &MacroDelimiter, right: &MacroDelimiter) -> bool {
    matches!(
        (left, right),
        (MacroDelimiter::Paren(_), MacroDelimiter::Paren(_))
            | (MacroDelimiter::Brace(_), MacroDelimiter::Brace(_))
            | (MacroDelimiter::Bracket(_), MacroDelimiter::Bracket(_))
    )
}
