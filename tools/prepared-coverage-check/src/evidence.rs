//! Attribute and expression constraints shared by operation evidence.

use syn::visit::{self, Visit};
use syn::{Attribute, Expr, Signature, Stmt};

pub(crate) fn require_unattributed_evidence(attributes: &[Attribute]) -> Result<(), String> {
    if attributes.is_empty() {
        Ok(())
    } else {
        Err("attributes on prepared-operation evidence are forbidden".to_owned())
    }
}

pub(crate) fn operation_expression(block: &syn::Block) -> Result<&Expr, String> {
    let [Stmt::Expr(expression, None)] = block.stmts.as_slice() else {
        return Err("operation mapping must contain exactly one tail expression".to_owned());
    };
    require_unattributed_expression(expression)?;
    Ok(expression)
}

pub(crate) fn require_unattributed_expression(expression: &Expr) -> Result<(), String> {
    let mut detector = AttributeDetector { found: false };
    visit::visit_expr(&mut detector, expression);
    require_not_found(
        detector,
        "attributes inside operation evidence are forbidden",
    )
}

pub(crate) fn require_unattributed_signature(signature: &Signature) -> Result<(), String> {
    let mut detector = AttributeDetector { found: false };
    visit::visit_signature(&mut detector, signature);
    require_not_found(
        detector,
        "attributes inside accepts_operation signature are forbidden",
    )
}

struct AttributeDetector {
    found: bool,
}

impl<'ast> Visit<'ast> for AttributeDetector {
    fn visit_attribute(&mut self, _attribute: &'ast Attribute) {
        self.found = true;
    }
}

fn require_not_found(detector: AttributeDetector, message: &str) -> Result<(), String> {
    if detector.found {
        Err(message.to_owned())
    } else {
        Ok(())
    }
}
