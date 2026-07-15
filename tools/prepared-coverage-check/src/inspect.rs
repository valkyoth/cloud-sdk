//! AST inspection for concrete endpoint and body adapter declarations.

use std::collections::BTreeSet;

use syn::{Attribute, Expr, ImplItem, Item, ItemImpl, Lit, Path, PathArguments, UseTree};

use crate::compatibility::accepted_operation_keys;
use crate::definitions::validate_adapter_definitions;
use crate::evidence::{
    operation_expression, reject_nested_expression, reject_nested_items, reject_nested_path,
    reject_nested_type, require_unattributed_evidence, require_unattributed_expression,
    require_unattributed_path, require_unattributed_type,
};
use crate::parse::{
    BodyComponentArgs, BodyWireArgs, EndpointPrepareArgs, EndpointWireArgs, QueryWireArgs,
};

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
    require_doc_only(&file.attrs)?;
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
        require_unattributed_evidence(item_attrs(item))?;
        reject_nested_items(item)?;
        match item {
            Item::Macro(item_macro) => {
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
                        validate_macro_type(&arguments.ty)?;
                        validate_macro_expression(&arguments.shape)?;
                        validate_macro_expression(&arguments.response)?;
                        validate_macro_expression(&arguments.destructive)?;
                        validate_macro_expression(&arguments.cost)?;
                        keys.extend(strict_endpoint_mapping(&arguments.mapping)?);
                    }
                    (RegistryKind::Body, Some("body_wire"))
                        if item_macro.mac.path.is_ident("body_wire") =>
                    {
                        let arguments = syn::parse2::<BodyWireArgs>(item_macro.mac.tokens.clone())
                            .map_err(|error| format!("invalid body_wire declaration: {error}"))?;
                        validate_macro_type(&arguments.ty)?;
                        validate_macro_expression(&arguments.endpoint)?;
                        validate_macro_path(&arguments.writer)?;
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
                        validate_macro_type(&arguments.ty)?;
                        validate_macro_path(&arguments.writer)?;
                        keys.push(checked_key(&arguments.key)?);
                    }
                    (RegistryKind::Endpoint, Some("query_wire"))
                        if item_macro.mac.path.is_ident("query_wire") =>
                    {
                        let arguments = syn::parse2::<QueryWireArgs>(item_macro.mac.tokens.clone())
                            .map_err(|error| format!("invalid query_wire declaration: {error}"))?;
                        validate_macro_type(&arguments.ty)?;
                        validate_macro_expression(&arguments.endpoint)?;
                    }
                    (RegistryKind::Endpoint, Some("impl_endpoint_prepare"))
                        if adapter_definition_root
                            && item_macro.mac.path.is_ident("impl_endpoint_prepare") =>
                    {
                        let arguments =
                            syn::parse2::<EndpointPrepareArgs>(item_macro.mac.tokens.clone())
                                .map_err(|error| {
                                    format!("invalid impl_endpoint_prepare declaration: {error}")
                                })?;
                        for ty in &arguments.types {
                            validate_macro_type(ty)?;
                        }
                    }
                    _ => {
                        return Err(
                            "unreviewed module-scope macro invocation is forbidden".to_owned()
                        );
                    }
                }
            }
            Item::Impl(item_impl) if implements_reserved(item_impl, kind) => {
                validate_impl_items(item_impl)?;
                if !implements_canonical(item_impl, kind) {
                    return Err(format!(
                        "{} implementations must use crate::prepared::{}",
                        kind.label_wire(),
                        kind.label_wire()
                    ));
                }
                inspect_implementation(item_impl, kind, keys)?;
            }
            Item::Impl(item_impl)
                if kind == RegistryKind::Endpoint && implements_named(item_impl, "QueryWire") =>
            {
                validate_impl_items(item_impl)?;
                if !implements_canonical_named(item_impl, "QueryWire") {
                    return Err(
                        "QueryWire implementations must use crate::prepared::QueryWire".to_owned(),
                    );
                }
                if let Some(accepted) = accepted_operation_keys(item_impl)? {
                    for key in accepted {
                        checked_key(&key)?;
                    }
                }
            }
            Item::Use(item_use) => reject_reserved_use(&item_use.tree, kind)?,
            Item::Trait(item_trait)
                if reserved_trait(kind, item_trait.ident.to_string().as_str()) =>
            {
                return Err(format!(
                    "local {} trait definitions are forbidden",
                    item_trait.ident
                ));
            }
            Item::Type(item_type) if reserved_trait(kind, item_type.ident.to_string().as_str()) => {
                return Err(format!("local {} aliases are forbidden", item_type.ident));
            }
            Item::Mod(module) if module.content.is_some() => {
                return Err("inline modules are forbidden in prepared evidence".to_owned());
            }
            Item::Mod(_) if !adapter_definition_root => {
                return Err("nested prepared module declarations are forbidden".to_owned());
            }
            Item::Mod(_) => {}
            Item::Verbatim(_) => {
                return Err("unparsed prepared module items are forbidden".to_owned());
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
    if attributes
        .iter()
        .any(|attribute| attribute.path().is_ident("macro_use"))
    {
        return Err("macro imports are forbidden in prepared evidence".to_owned());
    }
    Ok(())
}

fn require_doc_only(attributes: &[Attribute]) -> Result<(), String> {
    if attributes
        .iter()
        .all(|attribute| attribute.path().is_ident("doc"))
    {
        Ok(())
    } else {
        Err("only documentation attributes are allowed on prepared files".to_owned())
    }
}

fn implements_reserved(item: &ItemImpl, kind: RegistryKind) -> bool {
    implements_named(item, kind.label_wire())
}

fn validate_impl_items(implementation: &ItemImpl) -> Result<(), String> {
    for item in &implementation.items {
        match item {
            ImplItem::Fn(method) if method.attrs.is_empty() => {}
            ImplItem::Fn(_) | ImplItem::Const(_) | ImplItem::Type(_)
                if !impl_attrs(item).is_empty() =>
            {
                return Err(
                    "attributes inside prepared wire implementations are forbidden".to_owned(),
                );
            }
            ImplItem::Macro(_) => {
                return Err("macros inside prepared wire implementations are forbidden".to_owned());
            }
            ImplItem::Verbatim(_) => {
                return Err("unparsed prepared wire implementation items are forbidden".to_owned());
            }
            _ => {
                return Err("unsupported prepared wire implementation item".to_owned());
            }
        }
    }
    Ok(())
}

fn impl_attrs(item: &ImplItem) -> &[Attribute] {
    match item {
        ImplItem::Const(item) => &item.attrs,
        ImplItem::Fn(item) => &item.attrs,
        ImplItem::Type(item) => &item.attrs,
        _ => &[],
    }
}

fn implements_named(item: &ItemImpl, name: &str) -> bool {
    item.trait_
        .as_ref()
        .and_then(|(_, path, _)| path.segments.last())
        .is_some_and(|segment| segment.ident == name)
}

fn implements_canonical(item: &ItemImpl, kind: RegistryKind) -> bool {
    implements_canonical_named(item, kind.label_wire())
}

fn implements_canonical_named(item: &ItemImpl, name: &str) -> bool {
    item.trait_
        .as_ref()
        .is_some_and(|(_, path, _)| canonical_trait(path, name))
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

fn reserved_trait(kind: RegistryKind, name: &str) -> bool {
    name == kind.label_wire() || kind == RegistryKind::Endpoint && name == "QueryWire"
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
                || reserved_trait(kind, path.ident.to_string().as_str())
                || use_tree_has_reserved(&path.tree, kind)
        }
        UseTree::Name(name) => {
            reserved_macro(kind, name.ident.to_string().as_str())
                || reserved_trait(kind, name.ident.to_string().as_str())
        }
        UseTree::Rename(rename) => {
            reserved_macro(kind, rename.ident.to_string().as_str())
                || reserved_macro(kind, rename.rename.to_string().as_str())
                || reserved_trait(kind, rename.ident.to_string().as_str())
                || reserved_trait(kind, rename.rename.to_string().as_str())
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
        && let Some(accepted) = accepted_operation_keys(implementation)?
    {
        for key in accepted {
            implementation_keys.insert(checked_key(&key)?);
        }
    }
    keys.extend(implementation_keys);
    Ok(())
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
    reject_nested_expression(expression)?;
    if !matches!(expression, Expr::Match(_)) {
        return Err("endpoint operation mapping must be an explicit match".to_owned());
    }
    strict_operation_expression(expression, false)
}

fn validate_macro_expression(expression: &Expr) -> Result<(), String> {
    require_unattributed_expression(expression)?;
    reject_nested_expression(expression)
}

fn validate_macro_type(ty: &syn::Type) -> Result<(), String> {
    require_unattributed_type(ty)?;
    reject_nested_type(ty)
}

fn validate_macro_path(path: &Path) -> Result<(), String> {
    require_unattributed_path(path)?;
    reject_nested_path(path)
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
