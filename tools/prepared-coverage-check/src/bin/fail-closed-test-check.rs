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
        for test in tests_with_bare_returns(&file) {
            failures.push(format!("{}::{test}", source.display()));
        }
    }
    if failures.is_empty() {
        println!("Reviewed Rust test functions fail closed on fixture errors.");
        return Ok(());
    }
    Err(format!(
        "expressionless return in test function(s):\n{}",
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

fn tests_with_bare_returns(file: &syn::File) -> BTreeSet<String> {
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
            let mut returns = BareReturnVisitor::default();
            returns.visit_block(&function.block);
            if returns.found {
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
struct BareReturnVisitor {
    found: bool,
}

impl<'ast> Visit<'ast> for BareReturnVisitor {
    fn visit_expr_return(&mut self, expression: &'ast syn::ExprReturn) {
        if expression.expr.is_none() {
            self.found = true;
        }
        visit::visit_expr_return(self, expression);
    }
}

#[cfg(test)]
mod tests {
    use super::tests_with_bare_returns;

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
        let failures = tests_with_bare_returns(&file);
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
        assert!(tests_with_bare_returns(&file).contains("nested"));
        Ok(())
    }
}
