//! AST inspection for concrete endpoint and body adapter declarations.

use std::collections::BTreeSet;

use syn::{Attribute, Expr, ImplItem, Item, ItemImpl, Lit, Stmt};

use crate::parse::{AcceptedKeys, BodyComponentArgs, BodyWireArgs, EndpointWireArgs};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegistryKind {
    Endpoint,
    Body,
}

impl RegistryKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Endpoint => "endpoint",
            Self::Body => "body",
        }
    }
}

pub(crate) fn inspect_source(source: &str, kind: RegistryKind) -> Result<Vec<String>, String> {
    let file = syn::parse_file(source).map_err(|error| format!("Rust parse failed: {error}"))?;
    let mut keys = Vec::new();
    inspect_items(&file.items, kind, &mut keys)?;
    Ok(keys)
}

fn inspect_items(items: &[Item], kind: RegistryKind, keys: &mut Vec<String>) -> Result<(), String> {
    for item in items {
        reject_conditional(item_attrs(item))?;
        match item {
            Item::Macro(item_macro) => {
                let name = item_macro
                    .mac
                    .path
                    .segments
                    .last()
                    .map(|segment| &segment.ident);
                match (kind, name.map(ToString::to_string).as_deref()) {
                    (RegistryKind::Endpoint, Some("endpoint_wire")) => {
                        let arguments =
                            syn::parse2::<EndpointWireArgs>(item_macro.mac.tokens.clone())
                                .map_err(|error| {
                                    format!("invalid endpoint_wire declaration: {error}")
                                })?;
                        keys.extend(strict_endpoint_mapping(&arguments.mapping)?);
                    }
                    (RegistryKind::Body, Some("body_wire")) => {
                        let arguments = syn::parse2::<BodyWireArgs>(item_macro.mac.tokens.clone())
                            .map_err(|error| format!("invalid body_wire declaration: {error}"))?;
                        keys.push(checked_key(&arguments.key)?);
                    }
                    (RegistryKind::Body, Some("body_component")) => {
                        let arguments =
                            syn::parse2::<BodyComponentArgs>(item_macro.mac.tokens.clone())
                                .map_err(|error| {
                                    format!("invalid body_component declaration: {error}")
                                })?;
                        keys.push(checked_key(&arguments.key)?);
                    }
                    _ => {}
                }
            }
            Item::Impl(item_impl) if implements(item_impl, kind.label_wire()) => {
                inspect_implementation(item_impl, kind, keys)?;
            }
            Item::Mod(item_mod) => {
                if let Some((_, nested)) = &item_mod.content {
                    inspect_items(nested, kind, keys)?;
                }
            }
            _ => {}
        }
    }
    Ok(())
}

impl RegistryKind {
    const fn label_wire(self) -> &'static str {
        match self {
            Self::Endpoint => "EndpointWire",
            Self::Body => "BodyWire",
        }
    }
}

fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(item) => &item.attrs,
        Item::Enum(item) => &item.attrs,
        Item::ExternCrate(item) => &item.attrs,
        Item::Fn(item) => &item.attrs,
        Item::ForeignMod(item) => &item.attrs,
        Item::Impl(item) => &item.attrs,
        Item::Macro(item) => &item.attrs,
        Item::Mod(item) => &item.attrs,
        Item::Static(item) => &item.attrs,
        Item::Struct(item) => &item.attrs,
        Item::Trait(item) => &item.attrs,
        Item::TraitAlias(item) => &item.attrs,
        Item::Type(item) => &item.attrs,
        Item::Union(item) => &item.attrs,
        Item::Use(item) => &item.attrs,
        _ => &[],
    }
}

fn reject_conditional(attributes: &[Attribute]) -> Result<(), String> {
    if attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("cfg") || attribute.path().is_ident("cfg_attr"))
    {
        return Err("conditionally compiled prepared evidence is forbidden".to_owned());
    }
    Ok(())
}

fn implements(item: &ItemImpl, expected: &str) -> bool {
    item.trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .is_some_and(|segment| segment.ident == expected)
}

fn inspect_implementation(
    implementation: &ItemImpl,
    kind: RegistryKind,
    keys: &mut Vec<String>,
) -> Result<(), String> {
    let operation = implementation
        .items
        .iter()
        .find_map(|item| match item {
            ImplItem::Fn(method) if method.sig.ident == "operation_key" => Some(method),
            _ => None,
        })
        .ok_or_else(|| format!("{} implementation has no operation_key", kind.label_wire()))?;
    reject_conditional(&operation.attrs)?;
    let mut implementation_keys = BTreeSet::new();
    implementation_keys.extend(strict_operation_expression(
        block_tail(&operation.block)?,
        kind == RegistryKind::Body,
    )?);

    if kind == RegistryKind::Body
        && let Some(accepted) = implementation.items.iter().find_map(|item| match item {
            ImplItem::Fn(method) if method.sig.ident == "accepts_operation" => Some(method),
            _ => None,
        })
    {
        reject_conditional(&accepted.attrs)?;
        let expression = block_tail(&accepted.block)?;
        let Expr::Macro(expression_macro) = expression else {
            return Err("accepts_operation must contain one matches! expression".to_owned());
        };
        if !expression_macro.mac.path.is_ident("matches") {
            return Err("accepts_operation must use matches!".to_owned());
        }
        let accepted = syn::parse2::<AcceptedKeys>(expression_macro.mac.tokens.clone())
            .map_err(|error| format!("invalid accepts_operation mapping: {error}"))?;
        for key in accepted.keys {
            implementation_keys.insert(checked_key(&key)?);
        }
    }
    keys.extend(implementation_keys);
    Ok(())
}

fn block_tail(block: &syn::Block) -> Result<&Expr, String> {
    match block.stmts.last() {
        Some(Stmt::Expr(expression, None)) => Ok(expression),
        _ => Err("operation mapping must be the method's tail expression".to_owned()),
    }
}

fn strict_operation_expression(
    expression: &Expr,
    allow_empty_sentinel: bool,
) -> Result<Vec<String>, String> {
    match expression {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(value) => literal_key(value, allow_empty_sentinel),
            _ => Err("operation mapping literal must be a string".to_owned()),
        },
        Expr::Match(expression_match) => {
            let mut keys = Vec::new();
            for arm in &expression_match.arms {
                reject_conditional(&arm.attrs)?;
                if arm.guard.is_some() {
                    return Err("operation match arms cannot have guards".to_owned());
                }
                let Expr::Lit(literal) = arm.body.as_ref() else {
                    return Err("operation match arms must return string literals".to_owned());
                };
                let Lit::Str(value) = &literal.lit else {
                    return Err("operation match arms must return string literals".to_owned());
                };
                keys.extend(literal_key(value, allow_empty_sentinel)?);
            }
            if keys.is_empty() {
                return Err("operation match cannot be empty".to_owned());
            }
            Ok(keys)
        }
        _ => Err("operation mapping must be a string literal or explicit match".to_owned()),
    }
}

fn strict_endpoint_mapping(expression: &Expr) -> Result<Vec<String>, String> {
    if !matches!(expression, Expr::Match(_)) {
        return Err("endpoint operation mapping must be an explicit match".to_owned());
    }
    strict_operation_expression(expression, false)
}

fn literal_key(value: &syn::LitStr, allow_empty_sentinel: bool) -> Result<Vec<String>, String> {
    if allow_empty_sentinel && value.value().is_empty() {
        return Ok(Vec::new());
    }
    Ok(vec![checked_key(value)?])
}

fn checked_key(value: &syn::LitStr) -> Result<String, String> {
    let key = value.value();
    let mut bytes = key.bytes();
    if !bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        || !bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
    {
        return Err("invalid operation key literal".to_owned());
    }
    Ok(key)
}
