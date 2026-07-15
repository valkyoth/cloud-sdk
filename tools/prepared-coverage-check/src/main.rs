//! Rust-syntax-aware prepared-operation coverage evidence.

mod definitions;
mod inspect;
mod parse;

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
    let endpoint_root = required_path(arguments.next(), "endpoint root")?;
    let body_root = required_path(arguments.next(), "body root")?;
    let endpoint_dir = required_path(arguments.next(), "endpoint directory")?;
    let body_dir = required_path(arguments.next(), "body directory")?;
    if arguments.next().is_some() {
        return Err("unexpected checker argument".to_owned());
    }

    let endpoint_files = source_files(&endpoint_root, &endpoint_dir)?;
    let body_files = source_files(&body_root, &body_dir)?;
    emit_registry(&endpoint_files, RegistryKind::Endpoint)?;
    emit_registry(&body_files, RegistryKind::Body)
}

fn required_path(value: Option<std::ffi::OsString>, label: &str) -> Result<PathBuf, String> {
    value
        .map(PathBuf::from)
        .ok_or_else(|| format!("missing {label} argument"))
}

fn source_files(root: &Path, directory: &Path) -> Result<Vec<PathBuf>, String> {
    if !root.is_file() {
        return Err(format!("missing Rust source {}", root.display()));
    }
    let entries = fs::read_dir(directory)
        .map_err(|error| format!("cannot read {}: {error}", directory.display()))?;
    let mut files = Vec::new();
    files.push(root.to_path_buf());
    for entry in entries {
        let entry = entry.map_err(|error| format!("cannot read source entry: {error}"))?;
        let path = entry.path();
        if path.is_file() && path.extension() == Some(OsStr::new("rs")) {
            files.push(path);
        }
    }
    files
        .get_mut(1..)
        .ok_or_else(|| "invalid source list".to_owned())?
        .sort();
    if files.len() == 1 {
        return Err(format!(
            "no Rust sources found under {}",
            directory.display()
        ));
    }
    Ok(files)
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
