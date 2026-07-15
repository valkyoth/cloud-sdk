//! Attribute and expression constraints shared by operation evidence.

use syn::visit::{self, Visit};
use syn::{Attribute, Expr, Item, Path, Signature, Stmt, Type};

use crate::parse::MatchesArgs;

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

pub(crate) fn require_unattributed_type(ty: &Type) -> Result<(), String> {
    let mut detector = AttributeDetector { found: false };
    detector.visit_type(ty);
    require_not_found(detector, "attributes inside adapter types are forbidden")
}

pub(crate) fn require_unattributed_path(path: &Path) -> Result<(), String> {
    let mut detector = AttributeDetector { found: false };
    detector.visit_path(path);
    require_not_found(detector, "attributes inside adapter paths are forbidden")
}

pub(crate) fn reject_nested_items(item: &Item) -> Result<(), String> {
    let mut detector = NestedItemDetector { found: false };
    detector.visit_item(item);
    detector.result()
}

pub(crate) fn reject_nested_expression(expression: &Expr) -> Result<(), String> {
    let mut detector = NestedItemDetector { found: false };
    detector.visit_expr(expression);
    detector.result()
}

pub(crate) fn reject_nested_type(ty: &Type) -> Result<(), String> {
    let mut detector = NestedItemDetector { found: false };
    detector.visit_type(ty);
    detector.result()
}

pub(crate) fn reject_nested_path(path: &Path) -> Result<(), String> {
    let mut detector = NestedItemDetector { found: false };
    detector.visit_path(path);
    detector.result()
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

struct NestedItemDetector {
    found: bool,
}

impl<'ast> Visit<'ast> for NestedItemDetector {
    fn visit_stmt(&mut self, statement: &'ast Stmt) {
        if matches!(statement, Stmt::Item(_) | Stmt::Macro(_)) {
            self.found = true;
            return;
        }
        visit::visit_stmt(self, statement);
    }

    fn visit_expr_macro(&mut self, expression: &'ast syn::ExprMacro) {
        if !expression.mac.path.is_ident("matches") {
            self.found = true;
            return;
        }
        let Ok(arguments) = syn::parse2::<MatchesArgs>(expression.mac.tokens.clone()) else {
            self.found = true;
            return;
        };
        if matches_arguments_are_unsafe(&arguments) {
            self.found = true;
        }
    }

    fn visit_type_macro(&mut self, _ty: &'ast syn::TypeMacro) {
        self.found = true;
    }

    fn visit_pat(&mut self, pattern: &'ast syn::Pat) {
        if matches!(pattern, syn::Pat::Macro(_)) {
            self.found = true;
            return;
        }
        visit::visit_pat(self, pattern);
    }
}

impl NestedItemDetector {
    fn result(self) -> Result<(), String> {
        if self.found {
            Err("nested items or opaque macros are forbidden".to_owned())
        } else {
            Ok(())
        }
    }
}

fn matches_arguments_are_unsafe(arguments: &MatchesArgs) -> bool {
    let mut detector = NestedItemDetector { found: false };
    detector.visit_expr(&arguments.expression);
    detector.visit_pat(&arguments.pattern);
    if let Some(guard) = &arguments.guard {
        detector.visit_expr(guard);
    }
    detector.found
}
