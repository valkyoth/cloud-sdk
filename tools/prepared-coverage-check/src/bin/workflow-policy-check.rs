//! Semantic GitHub Actions workflow policy validation.

use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use saphyr::{LoadableYamlNode, Mapping, Yaml};
use saphyr_parser::{Event, Parser};

#[path = "workflow_policy/runner.rs"]
mod workflow_policy_runner;

const MAX_WORKFLOW_BYTES: usize = 1024 * 1024;
const MAX_YAML_EVENTS: usize = 10_000;
const MAX_YAML_DEPTH: usize = 64;
const ALLOWED_ACTIONS: [&str; 1] = ["actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1"];
const ALLOWED_EXPRESSIONS: [&str; 4] = ["matrix.os", "matrix.rust", "matrix.target", "runner.temp"];
const ALLOWED_RUN_COMMANDS: [&str; 18] = [
    "rustup target add \"$TARGET\"",
    "scripts/check_platform_matrix.sh --portable \"$TARGET\"",
    "scripts/check_platform_matrix.sh --native",
    "scripts/check_fips_deferred.py",
    "rustup toolchain install \"$RUST_VERSION\" --profile minimal",
    "scripts/check_rust_version_matrix.sh \"$RUST_VERSION\"",
    "scripts/check_packaged_feature_graphs.sh\ncargo doc --locked --workspace --all-features --no-deps",
    "rustup toolchain install nightly-2026-09-04 --profile minimal",
    "cargo install --locked cargo-fuzz --version 0.13.2",
    "scripts/check_fuzz_harness.sh --build\nscripts/check_fuzz_harness.sh --smoke",
    "rustup show",
    "cargo install --locked cargo-deny --version 0.20.2\ncargo install --locked cargo-audit --version 0.22.2",
    "cargo install --locked cargo-sbom --version 0.10.0",
    "scripts/checks.sh",
    r#"cargo deny check
cargo deny --manifest-path tests/reqwest-feature-unification/Cargo.toml \
  --config deny.toml --locked check advisories licenses sources
cargo deny --manifest-path fuzz/Cargo.toml \
  --config deny.toml --locked check advisories licenses sources
cargo deny --manifest-path tools/prepared-coverage-check/Cargo.toml \
  --config deny.toml --locked check advisories licenses sources"#,
    "scripts/check_rustsec_advisories.sh",
    "scripts/check_sbom_freshness.sh",
    "scripts/check_hetzner_api_surface.sh --fetch",
];
const ALLOWED_ENVIRONMENT: [(&str, &str); 5] = [
    ("CARGO_TERM_COLOR", "always"),
    ("RUSTDOCFLAGS", "-D warnings"),
    ("RUST_VERSION", "${{ matrix.rust }}"),
    ("TARGET", "${{ matrix.target }}"),
    ("TMPDIR", "${{ runner.temp }}"),
];

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
    reject_expansion_and_complexity(source, path)?;
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

fn reject_expansion_and_complexity(source: &str, path: &Path) -> Result<(), String> {
    let mut depth = 0_usize;
    for (count, result) in Parser::new_from_str(source).enumerate() {
        if count >= MAX_YAML_EVENTS {
            return Err(format!("{} contains too many YAML events", path.display()));
        }
        let (event, _) =
            result.map_err(|error| format!("cannot parse {}: {error}", path.display()))?;
        match event {
            Event::Alias(_)
            | Event::Scalar(_, _, 1.., _)
            | Event::SequenceStart(1.., _)
            | Event::MappingStart(1.., _) => {
                return Err(format!(
                    "{} contains forbidden YAML anchors or aliases",
                    path.display()
                ));
            }
            Event::SequenceStart(0, _) | Event::MappingStart(0, _) => {
                depth = depth
                    .checked_add(1)
                    .ok_or_else(|| "YAML nesting depth overflowed".to_owned())?;
                if depth > MAX_YAML_DEPTH {
                    return Err(format!("{} exceeds YAML nesting depth", path.display()));
                }
            }
            Event::SequenceEnd | Event::MappingEnd => {
                depth = depth
                    .checked_sub(1)
                    .ok_or_else(|| format!("{} has incoherent YAML nesting", path.display()))?;
            }
            _ => {}
        }
    }
    if depth != 0 {
        return Err(format!("{} has incomplete YAML nesting", path.display()));
    }
    Ok(())
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
    check_expressions(document, path)?;
    reject_key(root, "defaults", path, "workflow execution defaults")?;
    check_environment(root, path)?;
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
    if mapping_value(job, "environment").is_some() {
        return Err(format!(
            "{}: GitHub environments are forbidden in {name}",
            path.display()
        ));
    }
    for key in ["defaults", "container", "services", "uses"] {
        reject_key(job, key, path, &format!("job {key} in {name}"))?;
    }
    check_environment(job, path)?;
    if let Some(steps) = mapping_value(job, "steps") {
        let steps = steps
            .as_vec()
            .ok_or_else(|| format!("{}: steps in {name} must be a sequence", path.display()))?;
        for (index, step) in steps.iter().enumerate() {
            let step = mapping(step, path, &format!("step {index} in {name}"))?;
            check_step(step, path)?;
        }
    }
    workflow_policy_runner::check(job, path)?;
    Ok(())
}

fn check_step(step: &Mapping<'_>, path: &Path) -> Result<(), String> {
    let run = mapping_value(step, "run");
    let action = mapping_value(step, "uses");
    let allowed_keys: &[&str] = match (run, action) {
        (Some(command), None) => {
            check_run(command, path)?;
            check_environment(step, path)?;
            match mapping_value(step, "shell") {
                None => {}
                Some(shell) if shell.as_str() == Some("bash") => {}
                Some(_) => {
                    return Err(format!("{}: custom shell is forbidden", path.display()));
                }
            }
            &["name", "run", "env", "shell"]
        }
        (None, Some(reference)) => {
            check_action(reference, path)?;
            check_action_inputs(step, path)?;
            &["name", "uses", "with"]
        }
        _ => {
            return Err(format!(
                "{}: each step must contain exactly one approved run or uses entry",
                path.display()
            ));
        }
    };
    for key in step.keys().filter_map(Yaml::as_str) {
        if !allowed_keys.contains(&key) {
            return Err(format!(
                "{}: step key is not explicitly approved: {key}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn check_action(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let Some(reference) = node.as_str() else {
        return Err(format!("{}: action reference must be text", path.display()));
    };
    if !ALLOWED_ACTIONS.contains(&reference) {
        return Err(format!(
            "{}: action is not explicitly approved: {reference}",
            path.display()
        ));
    }
    Ok(())
}

fn check_action_inputs(step: &Mapping<'_>, path: &Path) -> Result<(), String> {
    let Some(inputs) = mapping_value(step, "with") else {
        return Ok(());
    };
    let inputs = mapping(inputs, path, "action inputs")?;
    let fetch_depth_is_zero = mapping_value(inputs, "fetch-depth")
        .and_then(Yaml::as_integer)
        .is_some_and(|value| value == 0);
    if inputs.len() != 1 || !fetch_depth_is_zero {
        return Err(format!(
            "{}: action inputs are not explicitly approved",
            path.display()
        ));
    }
    Ok(())
}

fn check_run(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    let Some(command) = node.as_str() else {
        return Err(format!("{}: run command must be text", path.display()));
    };
    let command = command.trim();
    if !ALLOWED_RUN_COMMANDS.contains(&command) {
        return Err(format!(
            "{}: run command is not explicitly approved: {command:?}",
            path.display()
        ));
    }
    Ok(())
}

fn check_environment(container: &Mapping<'_>, path: &Path) -> Result<(), String> {
    let Some(environment) = mapping_value(container, "env") else {
        return Ok(());
    };
    let environment = mapping(environment, path, "environment")?;
    for (name, value) in environment {
        let Some(name) = name.as_str() else {
            return Err(format!("{}: environment name must be text", path.display()));
        };
        let Some(value) = value.as_str() else {
            return Err(format!(
                "{}: environment value must be text",
                path.display()
            ));
        };
        if !ALLOWED_ENVIRONMENT.contains(&(name, value)) {
            return Err(format!(
                "{}: environment entry is not explicitly approved: {name}",
                path.display()
            ));
        }
    }
    Ok(())
}

fn check_expressions(node: &Yaml<'_>, path: &Path) -> Result<(), String> {
    match node {
        Yaml::Mapping(mapping) => {
            for (key, value) in mapping {
                let Some(key) = key.as_str() else {
                    return Err(format!(
                        "{} contains a non-string mapping key",
                        path.display()
                    ));
                };
                if key.eq_ignore_ascii_case("secrets") {
                    return Err(format!("{} contains a secrets mapping", path.display()));
                }
                check_expression_text(key, path)?;
                check_expressions(value, path)?;
            }
        }
        Yaml::Sequence(sequence) => {
            for value in sequence {
                check_expressions(value, path)?;
            }
        }
        _ => {
            if let Some(value) = node.as_str() {
                check_expression_text(value, path)?;
            }
        }
    }
    Ok(())
}

fn check_expression_text(value: &str, path: &Path) -> Result<(), String> {
    let mut remaining = value;
    loop {
        let opening = remaining.find("${{");
        let closing = remaining.find("}}");
        let Some(start) = opening else {
            if closing.is_some() {
                return Err(format!(
                    "{} contains an incomplete expression",
                    path.display()
                ));
            }
            return Ok(());
        };
        if closing.is_some_and(|end| end < start) {
            return Err(format!(
                "{} contains an incomplete expression",
                path.display()
            ));
        }
        let expression = &remaining[start + 3..];
        let Some(end) = expression.find("}}") else {
            return Err(format!(
                "{} contains an incomplete expression",
                path.display()
            ));
        };
        let expression_text = expression[..end].trim();
        if !ALLOWED_EXPRESSIONS.contains(&expression_text) {
            return Err(format!(
                "{}: expression is not explicitly approved: {expression_text:?}",
                path.display()
            ));
        }
        remaining = &expression[end + 2..];
    }
}

fn reject_key(mapping: &Mapping<'_>, key: &str, path: &Path, label: &str) -> Result<(), String> {
    if mapping_value(mapping, key).is_some() {
        return Err(format!("{}: {label} is forbidden", path.display()));
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
