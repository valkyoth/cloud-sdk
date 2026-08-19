//! GitHub-hosted runner policy for the workflow checker.

use std::path::Path;

use saphyr::{Mapping, Yaml};

const MATRIX_RUNNER: &str = "${{ matrix.os }}";
const ALLOWED_RUNNERS: [&str; 4] = [
    "ubuntu-latest",
    "windows-latest",
    "macos-15",
    "macos-15-intel",
];

pub(crate) fn check(job: &Mapping<'_>, path: &Path) -> Result<(), String> {
    let runner = required(job, "runs-on", path, "runner")?
        .as_str()
        .ok_or_else(|| format!("{}: runner selector (runs-on) must be text", path.display()))?;
    if runner == MATRIX_RUNNER {
        check_matrix(job, path)
    } else if ALLOWED_RUNNERS.contains(&runner) {
        Ok(())
    } else {
        Err(format!(
            "{}: runner is not explicitly approved: {runner}",
            path.display()
        ))
    }
}

fn check_matrix(job: &Mapping<'_>, path: &Path) -> Result<(), String> {
    let strategy = mapping(
        required(job, "strategy", path, "runner strategy")?,
        path,
        "runner strategy",
    )?;
    let matrix = mapping(
        required(strategy, "matrix", path, "runner matrix")?,
        path,
        "runner matrix",
    )?;
    if matrix.len() != 1 {
        return Err(format!(
            "{}: runner matrix may contain only the os axis",
            path.display()
        ));
    }
    let runners = required(matrix, "os", path, "matrix.os")?
        .as_vec()
        .ok_or_else(|| format!("{}: matrix.os must be a sequence", path.display()))?;
    if runners.is_empty()
        || runners.iter().any(|runner| {
            runner
                .as_str()
                .is_none_or(|runner| !ALLOWED_RUNNERS.contains(&runner))
        })
    {
        return Err(format!(
            "{}: runner matrix contains an unapproved runner",
            path.display()
        ));
    }
    Ok(())
}

fn mapping<'input, 'node>(
    node: &'node Yaml<'input>,
    path: &Path,
    label: &str,
) -> Result<&'node Mapping<'input>, String> {
    node.as_mapping()
        .ok_or_else(|| format!("{}: {label} must be a mapping", path.display()))
}

fn mapping_value<'input, 'node>(
    mapping: &'node Mapping<'input>,
    name: &str,
) -> Option<&'node Yaml<'input>> {
    mapping
        .iter()
        .find_map(|(key, value)| (key.as_str() == Some(name)).then_some(value))
}

fn required<'input, 'node>(
    mapping: &'node Mapping<'input>,
    name: &str,
    path: &Path,
    label: &str,
) -> Result<&'node Yaml<'input>, String> {
    mapping_value(mapping, name).ok_or_else(|| format!("{}: missing {label}", path.display()))
}
