//! Validation for manual query and body operation compatibility overrides.

use syn::{
    Expr, ExprMatch, FnArg, ImplItem, ImplItemFn, ItemImpl, Lit, Pat, ReceiverKind, ReturnType,
    Safety, Type,
};

use crate::evidence::{
    operation_expression, require_unattributed_evidence, require_unattributed_signature,
};
pub(crate) fn accepted_operation_keys(
    implementation: &ItemImpl,
) -> Result<Option<Vec<String>>, String> {
    let Some(method) = implementation.items.iter().find_map(|item| match item {
        ImplItem::Fn(method) if method.sig.ident == "accepts_operation" => Some(method),
        _ => None,
    }) else {
        return Ok(None);
    };
    require_unattributed_evidence(&method.attrs)?;
    require_accepts_signature(method)?;
    let expression = operation_expression(&method.block)?;
    let Expr::Match(mapping) = expression else {
        return Err("accepts_operation must contain one explicit match expression".to_owned());
    };
    Ok(Some(accepted_match_keys(mapping)?))
}

fn accepted_match_keys(mapping: &ExprMatch) -> Result<Vec<String>, String> {
    let canonical_scrutinee = matches!(
        mapping.expr.as_ref(),
        Expr::Path(path)
            if path.attrs.is_empty()
                && path.qself.is_none()
                && path.path.leading_colon.is_none()
                && path.path.is_ident("operation_key")
    );
    if !canonical_scrutinee {
        return Err("accepts_operation must match the operation_key parameter".to_owned());
    }
    let [accepted, fallback] = mapping.arms.as_slice() else {
        return Err(canonical_mapping_error());
    };
    if !accepted.attrs.is_empty()
        || matches!(accepted.pat, Pat::Guard(_))
        || !bool_literal(accepted.body.as_ref(), true)
        || !fallback.attrs.is_empty()
        || matches!(fallback.pat, Pat::Guard(_))
        || !matches!(&fallback.pat, Pat::Wild(wildcard) if wildcard.attrs.is_empty())
        || !bool_literal(fallback.body.as_ref(), false)
    {
        return Err(canonical_mapping_error());
    }
    let mut keys = Vec::new();
    collect_literal_patterns(&accepted.pat, &mut keys)?;
    if keys.is_empty() {
        return Err(canonical_mapping_error());
    }
    Ok(keys)
}

fn collect_literal_patterns(pattern: &Pat, keys: &mut Vec<String>) -> Result<(), String> {
    match pattern {
        Pat::Lit(literal) if literal.attrs.is_empty() => {
            let Lit::Str(value) = &literal.lit else {
                return Err(canonical_mapping_error());
            };
            keys.push(value.value());
            Ok(())
        }
        Pat::Or(patterns) if patterns.attrs.is_empty() => {
            for pattern in &patterns.cases {
                collect_literal_patterns(pattern, keys)?;
            }
            Ok(())
        }
        _ => Err(canonical_mapping_error()),
    }
}

fn bool_literal(expression: &Expr, expected: bool) -> bool {
    matches!(
        expression,
        Expr::Lit(literal)
            if literal.attrs.is_empty()
                && matches!(&literal.lit, Lit::Bool(value) if value.value == expected)
    )
}

fn canonical_mapping_error() -> String {
    "accepts_operation must match string literals to true and _ to false".to_owned()
}

fn require_accepts_signature(method: &ImplItemFn) -> Result<(), String> {
    require_unattributed_signature(&method.sig)?;
    let signature = &method.sig;
    if signature.constness.is_some()
        || signature.asyncness.is_some()
        || !matches!(signature.safety, Safety::Default)
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
    if !matches!(receiver.kind, ReceiverKind::Value)
        || receiver.mutability.is_some()
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
