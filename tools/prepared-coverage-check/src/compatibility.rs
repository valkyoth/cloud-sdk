//! Validation for manual query and body operation compatibility overrides.

use syn::{Expr, FnArg, ImplItem, ImplItemFn, ItemImpl, Pat, ReturnType, Type};

use crate::evidence::{
    operation_expression, require_unattributed_evidence, require_unattributed_signature,
};
use crate::parse::AcceptedKeys;

pub(crate) fn accepted_operation_keys(
    implementation: &ItemImpl,
) -> Result<Option<Vec<syn::LitStr>>, String> {
    let Some(method) = implementation.items.iter().find_map(|item| match item {
        ImplItem::Fn(method) if method.sig.ident == "accepts_operation" => Some(method),
        _ => None,
    }) else {
        return Ok(None);
    };
    require_unattributed_evidence(&method.attrs)?;
    require_accepts_signature(method)?;
    let expression = operation_expression(&method.block)?;
    let Expr::Macro(expression_macro) = expression else {
        return Err("accepts_operation must contain one matches! expression".to_owned());
    };
    if !expression_macro.mac.path.is_ident("matches") {
        return Err("accepts_operation must use matches!".to_owned());
    }
    let accepted = syn::parse2::<AcceptedKeys>(expression_macro.mac.tokens.clone())
        .map_err(|error| format!("invalid accepts_operation mapping: {error}"))?;
    Ok(Some(accepted.keys))
}

fn require_accepts_signature(method: &ImplItemFn) -> Result<(), String> {
    require_unattributed_signature(&method.sig)?;
    let signature = &method.sig;
    if signature.constness.is_some()
        || signature.asyncness.is_some()
        || signature.unsafety.is_some()
        || signature.abi.is_some()
        || signature.variadic.is_some()
        || !signature.generics.params.is_empty()
        || signature.generics.where_clause.is_some()
        || signature.inputs.len() != 2
    {
        return Err(canonical_signature_error());
    }
    let mut inputs = signature.inputs.iter();
    let Some(FnArg::Receiver(receiver)) = inputs.next() else {
        return Err(canonical_signature_error());
    };
    if receiver.reference.is_some()
        || receiver.mutability.is_some()
        || receiver.colon_token.is_some()
        || !receiver.attrs.is_empty()
    {
        return Err(canonical_signature_error());
    }
    let Some(FnArg::Typed(argument)) = inputs.next() else {
        return Err(canonical_signature_error());
    };
    let Pat::Ident(pattern) = argument.pat.as_ref() else {
        return Err(canonical_signature_error());
    };
    let Type::Reference(reference) = argument.ty.as_ref() else {
        return Err(canonical_signature_error());
    };
    let parameter_ok = argument.attrs.is_empty()
        && pattern.ident == "operation_key"
        && pattern.by_ref.is_none()
        && pattern.mutability.is_none()
        && pattern.subpat.is_none()
        && reference.lifetime.is_none()
        && reference.mutability.is_none()
        && matches!(reference.elem.as_ref(), Type::Path(path) if path.qself.is_none() && path.path.is_ident("str"));
    let return_ok = matches!(
        &signature.output,
        ReturnType::Type(_, output)
            if matches!(output.as_ref(), Type::Path(path) if path.qself.is_none() && path.path.is_ident("bool"))
    );
    if !parameter_ok || !return_ok {
        return Err(canonical_signature_error());
    }
    Ok(())
}

fn canonical_signature_error() -> String {
    "accepts_operation must use the canonical signature".to_owned()
}
