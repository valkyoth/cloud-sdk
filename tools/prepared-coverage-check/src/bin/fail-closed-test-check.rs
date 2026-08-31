//! Rust-syntax-aware fail-closed test assurance.

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};

const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;

fn main() {
    if let Err(error) = run() {
        eprintln!("fail-closed test assurance: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let roots: Vec<PathBuf> = arguments.map(PathBuf::from).collect();
    if roots.is_empty() {
        return Err("missing Rust source root".to_owned());
    }
    let mut sources = Vec::new();
    for root in roots {
        collect_sources(&root, &mut sources)?;
    }
    sources.sort();
    sources.dedup();

    let mut failures = Vec::new();
    for source in sources {
        let bytes = fs::read(&source)
            .map_err(|error| format!("cannot read {}: {error}", source.display()))?;
        if bytes.len() > MAX_SOURCE_BYTES {
            return Err(format!("source exceeds size limit: {}", source.display()));
        }
        let text = std::str::from_utf8(&bytes)
            .map_err(|error| format!("source is not UTF-8 ({}): {error}", source.display()))?;
        let file = syn::parse_file(text)
            .map_err(|error| format!("cannot parse {}: {error}", source.display()))?;
        for test in tests_with_fail_open_paths(&file) {
            failures.push(format!("{}::{test}", source.display()));
        }
    }
    if failures.is_empty() {
        println!("Reviewed Rust test functions fail closed on fixture errors.");
        return Ok(());
    }
    Err(format!(
        "fail-open control flow in test function(s):\n{}",
        failures.join("\n")
    ))
}

fn collect_sources(root: &Path, output: &mut Vec<PathBuf>) -> Result<(), String> {
    let metadata = fs::symlink_metadata(root)
        .map_err(|error| format!("cannot inspect {}: {error}", root.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("source root is a symlink: {}", root.display()));
    }
    if metadata.is_file() {
        if root.extension().is_some_and(|extension| extension == "rs") {
            output.push(root.to_owned());
        }
        return Ok(());
    }
    if !metadata.is_dir() {
        return Err(format!(
            "source root is not a directory: {}",
            root.display()
        ));
    }
    for entry in
        fs::read_dir(root).map_err(|error| format!("cannot read {}: {error}", root.display()))?
    {
        let entry = entry.map_err(|error| format!("cannot read directory entry: {error}"))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
        if file_type.is_symlink() {
            return Err(format!(
                "source tree contains a symlink: {}",
                path.display()
            ));
        }
        if file_type.is_dir() {
            collect_sources(&path, output)?;
        } else if file_type.is_file() && path.extension().is_some_and(|extension| extension == "rs")
        {
            output.push(path);
        }
    }
    Ok(())
}

fn tests_with_fail_open_paths(file: &syn::File) -> BTreeSet<String> {
    let mut visitor = TestVisitor::default();
    visitor.visit_file(file);
    visitor.failures
}

#[derive(Default)]
struct TestVisitor {
    failures: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for TestVisitor {
    fn visit_item_fn(&mut self, function: &'ast syn::ItemFn) {
        if is_test_function(function) {
            let mut assurance = FailOpenVisitor::default();
            assurance.visit_block(&function.block);
            if assurance.found {
                self.failures.insert(function.sig.ident.to_string());
            }
            return;
        }
        visit::visit_item_fn(self, function);
    }
}

fn is_test_function(function: &syn::ItemFn) -> bool {
    function.attrs.iter().any(|attribute| {
        attribute
            .path()
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "test")
    })
}

#[derive(Default)]
struct FailOpenVisitor {
    found: bool,
}

impl<'ast> Visit<'ast> for FailOpenVisitor {
    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if expression.expr.is_none() {
            self.found = true;
        }
        visit::visit_expr_return(self, expression);
    }

    fn visit_expr_if(&mut self, expression: &'ast syn::ExprIf) {
        if success_pattern(&expression.cond)
            && !expression
                .else_branch
                .as_ref()
                .is_some_and(|(_, branch)| explicit_failure(branch))
        {
            self.found = true;
        }
        visit::visit_expr_if(self, expression);
    }
}

fn success_pattern(expression: &syn::Expr) -> bool {
    let syn::Expr::Let(binding) = expression else {
        return false;
    };
    pattern_contains_success(&binding.pat)
}

fn pattern_contains_success(pattern: &syn::Pat) -> bool {
    match pattern {
        syn::Pat::TupleStruct(pattern) => {
            pattern
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Ok" || segment.ident == "Some")
                || pattern.elems.iter().any(pattern_contains_success)
        }
        syn::Pat::Tuple(pattern) => pattern.elems.iter().any(pattern_contains_success),
        syn::Pat::Paren(pattern) => pattern_contains_success(&pattern.pat),
        syn::Pat::Reference(pattern) => pattern_contains_success(&pattern.pat),
        syn::Pat::Type(pattern) => pattern_contains_success(&pattern.pat),
        syn::Pat::Or(pattern) => pattern.cases.iter().any(pattern_contains_success),
        syn::Pat::Slice(pattern) => pattern.elems.iter().any(pattern_contains_success),
        syn::Pat::Struct(pattern) => pattern
            .fields
            .iter()
            .any(|field| pattern_contains_success(&field.pat)),
        _ => false,
    }
}

fn explicit_failure(expression: &syn::Expr) -> bool {
    match expression {
        syn::Expr::Block(expression) => expression
            .block
            .stmts
            .last()
            .is_some_and(statement_is_failure),
        syn::Expr::Group(expression) => explicit_failure(&expression.expr),
        syn::Expr::Paren(expression) => explicit_failure(&expression.expr),
        syn::Expr::Macro(expression) => failure_macro(&expression.mac),
        syn::Expr::Return(expression) => expression.expr.as_deref().is_some_and(returned_error),
        _ => false,
    }
}

fn statement_is_failure(statement: &syn::Stmt) -> bool {
    match statement {
        syn::Stmt::Expr(expression, _) => explicit_failure(expression),
        syn::Stmt::Macro(statement) => failure_macro(&statement.mac),
        _ => false,
    }
}

fn failure_macro(mac: &syn::Macro) -> bool {
    mac.path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "unreachable" || segment.ident == "panic")
}

fn returned_error(expression: &syn::Expr) -> bool {
    let syn::Expr::Call(call) = expression else {
        return false;
    };
    let syn::Expr::Path(function) = call.func.as_ref() else {
        return false;
    };
    function
        .path
        .segments
        .last()
        .is_some_and(|segment| segment.ident == "Err")
}

#[cfg(test)]
mod tests {
    use super::tests_with_fail_open_paths;

    #[test]
    fn detects_only_expressionless_returns_inside_tests() -> Result<(), syn::Error> {
        let file = syn::parse_file(
            r#"
            fn helper() { return; }
            #[test]
            fn secure() { if fixture_failed() { return; } }
            #[test]
            fn value_return() -> bool { return true; }
            "#,
        )?;
        let failures = tests_with_fail_open_paths(&file);
        assert_eq!(failures.len(), 1);
        assert!(failures.contains("secure"));
        Ok(())
    }

    #[test]
    fn detects_bare_returns_in_nested_test_closures() -> Result<(), syn::Error> {
        let file = syn::parse_file(
            r#"
            #[tokio::test]
            async fn nested() { run(|| { return; }); }
            "#,
        )?;
        assert!(tests_with_fail_open_paths(&file).contains("nested"));
        Ok(())
    }

    #[test]
    fn detects_success_conditionals_without_failing_else() -> Result<(), syn::Error> {
        let file = syn::parse_file(
            r#"
            #[test]
            fn direct() { if let Ok(value) = fixture() { assert_safe(value); } }
            #[test]
            fn nested_tuple() {
                if let (Ok(first), Some(second)) = fixtures() {
                    assert_safe((first, second));
                }
            }
            #[test]
            fn empty_else() {
                if let Some(value) = fixture() { assert_safe(value); } else {}
            }
            "#,
        )?;
        let failures = tests_with_fail_open_paths(&file);
        assert_eq!(failures.len(), 3);
        assert!(failures.contains("direct"));
        assert!(failures.contains("nested_tuple"));
        assert!(failures.contains("empty_else"));
        Ok(())
    }

    #[test]
    fn accepts_explicitly_failing_success_conditionals() -> Result<(), syn::Error> {
        let file = syn::parse_file(
            r#"
            fn helper() { if let Ok(value) = fixture() { use_value(value); } }
            #[test]
            fn unreachable_else() {
                if let Ok(value) = fixture() { assert_safe(value); }
                else { unreachable!("fixture failed"); }
            }
            #[test]
            fn error_return() -> Result<(), &'static str> {
                if let Some(value) = fixture() { assert_safe(value); }
                else { return Err("fixture failed"); }
                Ok(())
            }
            "#,
        )?;
        assert!(tests_with_fail_open_paths(&file).is_empty());
        Ok(())
    }
}
