//! Semantic GitHub Actions workflow policy validation.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use saphyr::{LoadableYamlNode, Mapping, Yaml};

const MAX_WORKFLOW_BYTES: usize = 1024 * 1024;
const FORBIDDEN_RUN_TEXT: [&str; 4] = ["cargo publish", "cargo owner", "gh release", "git push"];

fn main() {
    if let Err(error) = run(env::args_os().skip(1)) {
        eprintln!("workflow policy: {error}");
        std::process::exit(1);
    }
}

fn run(arguments: impl Iterator<Item = OsString>) -> Result<(), String> {
    let paths = arguments.map(PathBuf::from).collect::<Vec<_>>();
    if paths.is_empty() {
        return Err("missing workflow path".to_owned());
    }
    for path in paths {
        check_workflow(&path)?;
    }
    Ok(())
}

fn check_workflow(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect {}: {error}", path.display()))?;
    if !metadata.file_type().is_file() {
        return Err(format!("{} is not a regular file", path.display()));
    }
    let bytes =
        fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;
    if bytes.len() > MAX_WORKFLOW_BYTES {
        return Err(format!(
            "{} exceeds the workflow size limit",
            path.display()
        ));
    }
    let source = std::str::from_utf8(&bytes)
        .map_err(|error| format!("{} is not UTF-8: {error}", path.display()))?;
    let documents = Yaml::load_from_str(source)
        .map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
    let [document] = documents.as_slice() else {
        return Err(format!(
            "{} must contain exactly one YAML document",
            path.display()
        ));
    };
    reject_ambiguous_nodes(document, path)?;
    check_document(document, path)
}

fn reject_ambiguous_nodes(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    match node {
        Yaml::Mapping(mapping) => {
            for (key, value) in mapping {
                let Some(key) = key.as_str() else {
                    return Err(format!(
                        "{} contains a non-string mapping key",
                        path.display()
                    ));
                };
                if key == "<<" {
                    return Err(format!(
                        "{} contains a forbidden YAML merge key",
                        path.display()
                    ));
                }
                reject_ambiguous_nodes(value, path)?;
            }
        }
        Yaml::Sequence(sequence) => {
            for value in sequence {
                reject_ambiguous_nodes(value, path)?;
            }
        }
        Yaml::Alias(_) => {
            return Err(format!("{} contains a YAML alias", path.display()));
        }
        Yaml::Tagged(_, _) => {
            return Err(format!("{} contains a YAML tag", path.display()));
        }
        Yaml::Representation(_, _, _) | Yaml::BadValue => {
            return Err(format!(
                "{} contains an unresolved YAML value",
                path.display()
            ));
        }
        Yaml::Value(_) => {}
    }
    Ok(())
}

fn check_document(document: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let root = mapping(document, path, "workflow")?;
    if let Some(events) = mapping_value(root, "on") {
        check_triggers(events, path)?;
    }

    let permissions = mapping(
        required(root, "permissions", path, "top-level permissions")?,
        path,
        "top-level permissions",
    )?;
    if permissions.len() != 1 || scalar(mapping_value(permissions, "contents")) != Some("read") {
        return Err(format!(
            "{}: workflow permissions must be contents: read",
            path.display()
        ));
    }

    let jobs = mapping(required(root, "jobs", path, "jobs")?, path, "jobs")?;
    if jobs.is_empty() {
        return Err(format!("{}: jobs must not be empty", path.display()));
    }
    for (job_name, job) in jobs {
        let name = job_name
            .as_str()
            .ok_or_else(|| format!("{}: job name must be text", path.display()))?;
        check_job(name, job, path)?;
    }
    Ok(())
}

fn check_triggers(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let forbidden = |name: &str| matches!(name, "pull_request_target" | "release");
    let found = if let Some(name) = node.as_str() {
        forbidden(name)
    } else if let Some(events) = node.as_vec() {
        events.iter().filter_map(Yaml::as_str).any(forbidden)
    } else if let Some(events) = node.as_mapping() {
        events.keys().filter_map(Yaml::as_str).any(forbidden)
    } else {
        return Err(format!(
            "{}: workflow triggers must be text, a sequence, or a mapping",
            path.display()
        ));
    };
    if found {
        return Err(format!(
            "{} contains a forbidden workflow trigger",
            path.display()
        ));
    }
    Ok(())
}

fn check_job(name: &str, node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let job = mapping(node, path, &format!("job {name}"))?;
    if mapping_value(job, "permissions").is_some() {
        return Err(format!(
            "{}: job-level permissions are forbidden in {name}",
            path.display()
        ));
    }
    if let Some(reference) = mapping_value(job, "uses") {
        check_action(reference, path)?;
    }
    if let Some(steps) = mapping_value(job, "steps") {
        let steps = steps
            .as_vec()
            .ok_or_else(|| format!("{}: steps in {name} must be a sequence", path.display()))?;
        for (index, step) in steps.iter().enumerate() {
            let step = mapping(step, path, &format!("step {index} in {name}"))?;
            if let Some(reference) = mapping_value(step, "uses") {
                check_action(reference, path)?;
            }
            if let Some(command) = mapping_value(step, "run") {
                check_run(command, path)?;
            }
        }
    }
    Ok(())
}

fn check_action(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let Some(reference) = node.as_str() else {
        return Err(format!("{}: action reference must be text", path.display()));
    };
    let Some((repository, revision)) = reference.rsplit_once('@') else {
        return Err(format!(
            "{}: action is not SHA-pinned: {reference}",
            path.display()
        ));
    };
    let valid_repository = repository.contains('/')
        && repository
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-' | b'.' | b'/'));
    let valid_revision =
        revision.len() == 40 && revision.bytes().all(|byte| byte.is_ascii_hexdigit());
    if !valid_repository || !valid_revision {
        return Err(format!(
            "{}: action is not SHA-pinned: {reference}",
            path.display()
        ));
    }
    Ok(())
}

fn check_run(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let Some(command) = node.as_str() else {
        return Err(format!("{}: run command must be text", path.display()));
    };
    let lowered = command.to_ascii_lowercase();
    for forbidden in FORBIDDEN_RUN_TEXT {
        if lowered.contains(forbidden) {
            return Err(format!(
                "{}: forbidden workflow capability {forbidden}",
                path.display()
            ));
        }
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

fn scalar<'input, 'node>(node: Option<&'node Yaml<'input>>) -> Option<&'node str> {
    node.and_then(Yaml::as_str)
}
