//! AST inspection for concrete endpoint and body adapter declarations.

use std::collections::BTreeSet;

use syn::visit::{self, Visit};
use syn::{Attribute, Expr, ImplItem, Item, ItemImpl, Lit, Path, PathArguments, Stmt, UseTree};

use crate::definitions::validate_adapter_definitions;
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

pub(crate) fn inspect_source(
    source: &str,
    kind: RegistryKind,
    adapter_definition_root: bool,
) -> Result<Vec<String>, String> {
    let file = syn::parse_file(source).map_err(|error| format!("Rust parse failed: {error}"))?;
    reject_conditional(&file.attrs)?;
    validate_adapter_definitions(&file.items, kind, adapter_definition_root)?;
    let mut keys = Vec::new();
    inspect_items(&file.items, kind, adapter_definition_root, &mut keys)?;
    Ok(keys)
}

fn inspect_items(
    items: &[Item],
    kind: RegistryKind,
    adapter_definition_root: bool,
    keys: &mut Vec<String>,
) -> Result<(), String> {
    for item in items {
        reject_conditional(item_attrs(item))?;
        match item {
            Item::Macro(item_macro) => {
                require_unattributed_evidence(&item_macro.attrs)?;
                if item_macro.mac.path.is_ident("macro_rules") {
                    continue;
                }
                let name = item_macro
                    .mac
                    .path
                    .segments
                    .last()
                    .map(|segment| &segment.ident);
                if name.is_some_and(|ident| reserved_macro(kind, ident.to_string().as_str()))
                    && !unqualified_adapter_macro(&item_macro.mac.path, kind)
                {
                    return Err("prepared adapter macros must use an unqualified path".to_owned());
                }
                match (kind, name.map(ToString::to_string).as_deref()) {
                    (RegistryKind::Endpoint, Some("endpoint_wire"))
                        if item_macro.mac.path.is_ident("endpoint_wire") =>
                    {
                        let arguments =
                            syn::parse2::<EndpointWireArgs>(item_macro.mac.tokens.clone())
                                .map_err(|error| {
                                    format!("invalid endpoint_wire declaration: {error}")
                                })?;
                        keys.extend(strict_endpoint_mapping(&arguments.mapping)?);
                    }
                    (RegistryKind::Body, Some("body_wire"))
                        if item_macro.mac.path.is_ident("body_wire") =>
                    {
                        let arguments = syn::parse2::<BodyWireArgs>(item_macro.mac.tokens.clone())
                            .map_err(|error| format!("invalid body_wire declaration: {error}"))?;
                        keys.push(checked_key(&arguments.key)?);
                    }
                    (RegistryKind::Body, Some("body_component"))
                        if item_macro.mac.path.is_ident("body_component") =>
                    {
                        let arguments =
                            syn::parse2::<BodyComponentArgs>(item_macro.mac.tokens.clone())
                                .map_err(|error| {
                                    format!("invalid body_component declaration: {error}")
                                })?;
                        keys.push(checked_key(&arguments.key)?);
                    }
                    (RegistryKind::Endpoint, Some("query_wire"))
                        if item_macro.mac.path.is_ident("query_wire") => {}
                    (RegistryKind::Endpoint, Some("impl_endpoint_prepare"))
                        if adapter_definition_root
                            && item_macro.mac.path.is_ident("impl_endpoint_prepare") => {}
                    _ => {
                        return Err(
                            "unreviewed module-scope macro invocation is forbidden".to_owned()
                        );
                    }
                }
            }
            Item::Impl(item_impl) if implements_reserved(item_impl, kind) => {
                require_unattributed_evidence(&item_impl.attrs)?;
                if !implements_canonical(item_impl, kind) {
                    return Err(format!(
                        "{} implementations must use crate::prepared::{}",
                        kind.label_wire(),
                        kind.label_wire()
                    ));
                }
                inspect_implementation(item_impl, kind, keys)?;
            }
            Item::Use(item_use) => reject_reserved_use(&item_use.tree, kind)?,
            Item::Trait(item_trait) if item_trait.ident == kind.label_wire() => {
                return Err(format!(
                    "local {} trait definitions are forbidden",
                    kind.label_wire()
                ));
            }
            Item::Type(item_type) if item_type.ident == kind.label_wire() => {
                return Err(format!("local {} aliases are forbidden", kind.label_wire()));
            }
            Item::Mod(module) if module.content.is_some() => {
                return Err("inline modules are forbidden in prepared evidence".to_owned());
            }
            Item::Mod(_) if !adapter_definition_root => {
                return Err("nested prepared module declarations are forbidden".to_owned());
            }
            Item::Mod(_) => {}
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
    if attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("macro_use"))
    {
        return Err("macro imports are forbidden in prepared evidence".to_owned());
    }
    Ok(())
}

fn require_unattributed_evidence(attributes: &[Attribute]) -> Result<(), String> {
    if attributes.is_empty() {
        Ok(())
    } else {
        Err("attributes on prepared-operation evidence are forbidden".to_owned())
    }
}

fn implements_reserved(item: &ItemImpl, kind: RegistryKind) -> bool {
    item.trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .is_some_and(|segment| segment.ident == kind.label_wire())
}

fn implements_canonical(item: &ItemImpl, kind: RegistryKind) -> bool {
    item.trait_
        .as_ref()
        .is_some_and(|(_, path, _)| canonical_trait(path, kind.label_wire()))
}

fn canonical_trait(path: &Path, trait_name: &str) -> bool {
    let expected = ["crate", "prepared", trait_name];
    path.leading_colon.is_none()
        && path.segments.len() == expected.len()
        && path.segments.iter().zip(expected).all(|(segment, name)| {
            segment.ident == name && matches!(segment.arguments, PathArguments::None)
        })
}

fn reserved_macro(kind: RegistryKind, name: &str) -> bool {
    match kind {
        RegistryKind::Endpoint => name == "endpoint_wire",
        RegistryKind::Body => name == "body_wire" || name == "body_component",
    }
}

fn unqualified_adapter_macro(path: &Path, kind: RegistryKind) -> bool {
    match kind {
        RegistryKind::Endpoint => path.is_ident("endpoint_wire"),
        RegistryKind::Body => path.is_ident("body_wire") || path.is_ident("body_component"),
    }
}

fn reject_reserved_use(tree: &UseTree, kind: RegistryKind) -> Result<(), String> {
    if use_tree_has_reserved(tree, kind) {
        return Err("prepared adapter imports and aliases are forbidden".to_owned());
    }
    Ok(())
}

fn use_tree_has_reserved(tree: &UseTree, kind: RegistryKind) -> bool {
    match tree {
        UseTree::Path(path) => {
            reserved_macro(kind, path.ident.to_string().as_str())
                || path.ident == kind.label_wire()
                || use_tree_has_reserved(&path.tree, kind)
        }
        UseTree::Name(name) => {
            reserved_macro(kind, name.ident.to_string().as_str()) || name.ident == kind.label_wire()
        }
        UseTree::Rename(rename) => {
            reserved_macro(kind, rename.ident.to_string().as_str())
                || reserved_macro(kind, rename.rename.to_string().as_str())
                || rename.ident == kind.label_wire()
                || rename.rename == kind.label_wire()
        }
        UseTree::Group(group) => group
            .items
            .iter()
            .any(|item| use_tree_has_reserved(item, kind)),
        UseTree::Glob(_) => true,
    }
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
    require_unattributed_evidence(&operation.attrs)?;
    let mut implementation_keys = BTreeSet::new();
    implementation_keys.extend(strict_operation_expression(
        operation_expression(&operation.block)?,
        kind == RegistryKind::Body,
    )?);

    if kind == RegistryKind::Body
        && let Some(accepted) = implementation.items.iter().find_map(|item| match item {
            ImplItem::Fn(method) if method.sig.ident == "accepts_operation" => Some(method),
            _ => None,
        })
    {
        require_unattributed_evidence(&accepted.attrs)?;
        let expression = operation_expression(&accepted.block)?;
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

fn operation_expression(block: &syn::Block) -> Result<&Expr, String> {
    let [Stmt::Expr(expression, None)] = block.stmts.as_slice() else {
        return Err("operation mapping must contain exactly one tail expression".to_owned());
    };
    require_unattributed_expression(expression)?;
    Ok(expression)
}

struct AttributeDetector {
    found: bool,
}

impl<'ast> Visit<'ast> for AttributeDetector {
    fn visit_attribute(&mut self, _attribute: &'ast Attribute) {
        self.found = true;
    }
}

fn require_unattributed_expression(expression: &Expr) -> Result<(), String> {
    let mut detector = AttributeDetector { found: false };
    visit::visit_expr(&mut detector, expression);
    if detector.found {
        Err("attributes inside operation evidence are forbidden".to_owned())
    } else {
        Ok(())
    }
}

fn strict_operation_expression(
    expression: &Expr,
    allow_empty_sentinel: bool,
) -> Result<Vec<String>, String> {
    require_unattributed_expression(expression)?;
    match expression {
        Expr::Lit(literal) => match &literal.lit {
            Lit::Str(value) => literal_key(value, allow_empty_sentinel),
            _ => Err("operation mapping literal must be a string".to_owned()),
        },
        Expr::Match(expression_match) => {
            let mut keys = Vec::new();
            for arm in &expression_match.arms {
                require_unattributed_evidence(&arm.attrs)?;
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
