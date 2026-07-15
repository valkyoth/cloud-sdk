//! Rust-syntax-aware prepared-operation coverage evidence.

mod definitions;
mod inspect;
mod parse;

use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use inspect::{RegistryKind, inspect_source};

const MAX_SOURCE_BYTES: usize = 2 * 1024 * 1024;

fn main() {
    if let Err(error) = run() {
        eprintln!("prepared operation coverage: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args_os();
    let _program = arguments.next();
    let crate_root = required_path(arguments.next(), "crate root")?;
    let prepared_root = required_path(arguments.next(), "prepared root")?;
    let endpoint_root = required_path(arguments.next(), "endpoint root")?;
    let body_root = required_path(arguments.next(), "body root")?;
    let endpoint_dir = required_path(arguments.next(), "endpoint directory")?;
    let body_dir = required_path(arguments.next(), "body directory")?;
    if arguments.next().is_some() {
        return Err("unexpected checker argument".to_owned());
    }

    require_canonical_layout(
        &crate_root,
        &prepared_root,
        &endpoint_root,
        &body_root,
        &endpoint_dir,
        &body_dir,
    )?;
    require_module_edge(&crate_root, "prepared", true)?;
    require_module_edge(&prepared_root, "endpoints", false)?;
    require_module_edge(&prepared_root, "bodies", false)?;

    let endpoint_files = source_files(&endpoint_root, &endpoint_dir)?;
    let body_files = source_files(&body_root, &body_dir)?;
    emit_registry(&endpoint_files, RegistryKind::Endpoint)?;
    emit_registry(&body_files, RegistryKind::Body)
}

fn require_canonical_layout(
    crate_root: &Path,
    prepared_root: &Path,
    endpoint_root: &Path,
    body_root: &Path,
    endpoint_dir: &Path,
    body_dir: &Path,
) -> Result<(), String> {
    let source_dir = crate_root
        .parent()
        .ok_or_else(|| "crate root has no parent directory".to_owned())?;
    if crate_root != source_dir.join("lib.rs") {
        return Err("noncanonical crate root path".to_owned());
    }
    let prepared_dir = source_dir.join("prepared");
    let expected = [
        (
            prepared_root,
            source_dir.join("prepared.rs"),
            "prepared root",
        ),
        (
            endpoint_root,
            prepared_dir.join("endpoints.rs"),
            "endpoint root",
        ),
        (body_root, prepared_dir.join("bodies.rs"), "body root"),
        (
            endpoint_dir,
            prepared_dir.join("endpoints"),
            "endpoint directory",
        ),
        (body_dir, prepared_dir.join("bodies"), "body directory"),
    ];
    for (actual, canonical, label) in expected {
        if actual != canonical {
            return Err(format!("noncanonical {label} path"));
        }
    }
    Ok(())
}

fn require_module_edge(parent: &Path, expected: &str, public: bool) -> Result<(), String> {
    require_regular_source(parent)?;
    let bytes =
        fs::read(parent).map_err(|error| format!("cannot read {}: {error}", parent.display()))?;
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err("prepared module parent exceeds local size limit".to_owned());
    }
    let source = std::str::from_utf8(&bytes)
        .map_err(|error| format!("{} is not UTF-8: {error}", parent.display()))?;
    let file = syn::parse_file(source)
        .map_err(|error| format!("cannot parse {}: {error}", parent.display()))?;
    if file.attrs.iter().any(forbidden_compilation_attribute) {
        return Err(format!(
            "conditional crate or module root is forbidden: {}",
            parent.display()
        ));
    }
    let mut matches = file.items.iter().filter_map(|item| {
        let syn::Item::Mod(module) = item else {
            return None;
        };
        (module.ident == expected).then_some(module)
    });
    let module = matches
        .next()
        .ok_or_else(|| format!("missing canonical mod {expected};"))?;
    if matches.next().is_some()
        || !module.attrs.is_empty()
        || module.content.is_some()
        || module.semi.is_none()
        || module.unsafety.is_some()
    {
        return Err(format!("noncanonical module edge for {expected}"));
    }
    let visibility_ok = if public {
        matches!(module.vis, syn::Visibility::Public(_))
    } else {
        matches!(module.vis, syn::Visibility::Inherited)
    };
    if !visibility_ok {
        return Err(format!("invalid visibility for module {expected}"));
    }
    Ok(())
}

fn forbidden_compilation_attribute(attribute: &syn::Attribute) -> bool {
    attribute.path().is_ident("cfg")
        || attribute.path().is_ident("cfg_attr")
        || attribute.path().is_ident("macro_use")
}

fn required_path(value: Option<std::ffi::OsString>, label: &str) -> Result<PathBuf, String> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {label} argument"))
}

fn source_files(root: &Path, directory: &Path) -> Result<Vec<PathBuf>, String> {
    require_regular_source(root)?;
    let root_bytes = fs::read(root)
        .map_err(|error| format!("cannot read module root {}: {error}", root.display()))?;
    if root_bytes.len() > MAX_SOURCE_BYTES {
        return Err("prepared module root exceeds local size limit".to_owned());
    }
    let root_source = std::str::from_utf8(&root_bytes)
        .map_err(|error| format!("{} is not UTF-8: {error}", root.display()))?;
    let root_file = syn::parse_file(root_source)
        .map_err(|error| format!("cannot parse module root {}: {error}", root.display()))?;
    let declared = declared_modules(&root_file.items)?;

    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    let mut available = BTreeMap::new();
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read source entry: {error}"))?;
        let path = entry.path();
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        require_regular_source(&path)?;
        let name = path
            .file_stem()
            .and_then(OsStr::to_str)
            .ok_or_else(|| "prepared module filename is not UTF-8".to_owned())?;
        if available.insert(name.to_owned(), path).is_some() {
            return Err("duplicate prepared module filename".to_owned());
        }
    }
    let available_names = available.keys().cloned().collect::<BTreeSet<_>>();
    let missing = declared.difference(&available_names).next();
    if let Some(name) = missing {
        return Err(format!(
            "declared prepared module {name} has no source file"
        ));
    }
    let orphan = available_names.difference(&declared).next();
    if let Some(name) = orphan {
        return Err(format!("orphan prepared module source: {name}.rs"));
    }
    if declared.is_empty() {
        return Err(format!(
            "no declared Rust modules found under {}",
            directory.display()
        ));
    }
    let mut files = Vec::with_capacity(declared.len() + 1);
    files.push(root.to_path_buf());
    for name in declared {
        files.push(
            available
                .remove(&name)
                .ok_or_else(|| "prepared module set changed during validation".to_owned())?,
        );
    }
    Ok(files)
}

fn declared_modules(items: &[syn::Item]) -> Result<BTreeSet<String>, String> {
    let mut modules = BTreeSet::new();
    for item in items {
        let syn::Item::Mod(module) = item else {
            continue;
        };
        if !module.attrs.is_empty() {
            return Err(format!(
                "prepared module {} cannot have attributes",
                module.ident
            ));
        }
        if module.content.is_some() || module.semi.is_none() {
            return Err(format!(
                "prepared module {} must be an external module declaration",
                module.ident
            ));
        }
        if !matches!(module.vis, syn::Visibility::Inherited) {
            return Err(format!(
                "prepared module {} must have inherited visibility",
                module.ident
            ));
        }
        let name = module.ident.to_string();
        if !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
        {
            return Err("prepared module name is not canonical ASCII".to_owned());
        }
        if !modules.insert(name.clone()) {
            return Err(format!("duplicate prepared module declaration: {name}"));
        }
    }
    Ok(modules)
}

fn require_regular_source(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("missing Rust source {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!(
            "prepared Rust source is not a regular file: {}",
            path.display()
        ));
    }
    Ok(())
}

fn emit_registry(files: &[PathBuf], kind: RegistryKind) -> Result<(), String> {
    let mut total = 0_usize;
    for (index, path) in files.iter().enumerate() {
        let bytes =
            fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        total = total
            .checked_add(bytes.len())
            .ok_or_else(|| "prepared source size overflow".to_owned())?;
        if total > MAX_SOURCE_BYTES {
            return Err("prepared source evidence exceeds local size limit".to_owned());
        }
        let source = std::str::from_utf8(&bytes)
            .map_err(|error| format!("{} is not UTF-8: {error}", path.display()))?;
        let keys = inspect_source(source, kind, index == 0)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        for key in keys {
            println!("{}\t{key}", kind.label());
        }
    }
    Ok(())
}
