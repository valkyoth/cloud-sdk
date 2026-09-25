use super::PublishError as Error;
use crate::{
    discovery::DiscoveryValue as Value,
    identifiers::{CategorySlug, CrateName, Keyword, Version},
};

pub(super) fn text(v: &Value, max: usize, check: impl FnOnce(&str) -> bool) -> Result<(), Error> {
    v.with_text(|s| {
        if s.len() > max {
            Err(Error::Limit)
        } else if check(s) {
            Ok(())
        } else {
            Err(Error::Value)
        }
    })?
}
fn plain(s: &str) -> bool {
    !s.is_empty() && !s.chars().any(char::is_control)
}
fn optional(
    v: &Value,
    name: &str,
    max: usize,
    check: impl FnOnce(&str) -> bool,
) -> Result<(), Error> {
    if let Some(v) = v.get(name)?
        && !v.is_null()
    {
        text(v, max, check)?;
    }
    Ok(())
}
fn path(s: &str) -> bool {
    plain(s)
        && !s.contains(['\\', ':'])
        && s.split('/').all(|p| !p.is_empty() && p != "." && p != "..")
}
fn url(s: &str) -> bool {
    let Some(rest) = s
        .strip_prefix("https://")
        .or_else(|| s.strip_prefix("http://"))
    else {
        return false;
    };
    let host = rest.split(['/', '?', '#']).next().unwrap_or("");
    plain(s)
        && !s.chars().any(char::is_whitespace)
        && !s.contains('\\')
        && !host.is_empty()
        && !host.contains('@')
}
pub(super) fn feature(s: &str) -> bool {
    s.len() <= 256
        && s.as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphanumeric() || *b == b'_')
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_+.".contains(&b))
}
fn dependency_name(s: &str) -> bool {
    s.len() <= 64
        && s.as_bytes()
            .first()
            .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_".contains(&b))
}
fn feature_value(s: &str) -> bool {
    if let Some(name) = s.strip_prefix("dep:") {
        return dependency_name(name);
    }
    if let Some((dep, name)) = s.split_once('/') {
        return dependency_name(dep.strip_suffix('?').unwrap_or(dep)) && feature(name);
    }
    feature(s)
}
fn strings(v: &Value, max: usize, size: usize, check: impl Fn(&str) -> bool) -> Result<(), Error> {
    let values = v.array()?;
    if values.len() > max {
        return Err(Error::Limit);
    }
    for v in values {
        text(v, size, &check)?;
    }
    Ok(())
}
pub(super) fn validate(root: &Value) -> Result<(), Error> {
    root.visit_fields(|k, _| {
        if [
            "name",
            "vers",
            "deps",
            "features",
            "authors",
            "description",
            "documentation",
            "homepage",
            "readme",
            "readme_file",
            "keywords",
            "categories",
            "license",
            "license_file",
            "repository",
            "badges",
            "links",
            "rust_version",
        ]
        .contains(&k)
        {
            Ok(())
        } else {
            Err(Error::Schema)
        }
    })?;
    text(root.required("name")?, 64, |s| CrateName::new(s).is_ok())?;
    text(root.required("vers")?, 150, |s| Version::new(s).is_ok())?;
    strings(root.required("authors")?, 64, 1024, plain)?;
    strings(root.required("keywords")?, 5, 20, |s| {
        Keyword::new(s).is_ok()
    })?;
    strings(root.required("categories")?, 5, 256, |s| {
        CategorySlug::new(s).is_ok()
    })?;
    optional(root, "description", 1000, plain)?;
    optional(root, "readme", 65_536, |_| true)?;
    for key in ["homepage", "documentation", "repository"] {
        optional(root, key, 4096, url)?;
    }
    for key in ["readme_file", "license_file"] {
        optional(root, key, 1024, path)?;
    }
    optional(root, "links", 256, plain)?;
    optional(root, "license", 1024, |s| {
        plain(s) && spdx::Expression::parse(s).is_ok()
    })?;
    optional(root, "rust_version", 150, |s| {
        // Cargo accepts major.minor or major.minor.patch, without operators or pre/build suffixes.
        let mut parts = s.split('.');
        let valid = |p: &str| {
            !p.is_empty()
                && (p == "0" || !p.starts_with('0'))
                && p.bytes().all(|b| b.is_ascii_digit())
                && p.parse::<u64>().is_ok()
        };
        let Some(a) = parts.next() else {
            return false;
        };
        let Some(b) = parts.next() else {
            return false;
        };
        valid(a) && valid(b) && parts.next().is_none_or(valid) && parts.next().is_none()
    })?;
    if let Some(badges) = root.get("badges")?
        && !badges.is_null()
    {
        badges.visit_fields(|name, values| {
            if !plain(name) {
                return Err(Error::Value);
            }
            values.visit_fields(|_, value| text(value, 4096, |_| true))
        })?;
    }
    root.required("features")?.visit_fields(|name, values| {
        if !feature(name) {
            return Err(Error::Value);
        }
        strings(values, 300, 512, feature_value)
    })?;
    let dependencies = root.required("deps")?.array()?;
    if dependencies.len() > 256 {
        return Err(Error::Limit);
    }
    for (i, dep) in dependencies.iter().enumerate() {
        dependency(dep)?;
        for prior in dependencies.get(..i).ok_or(Error::Limit)? {
            if same(alias(dep)?, alias(prior)?)?
                && kind(dep)? == kind(prior)?
                && same_optional(dep, prior, "target")?
            {
                return Err(Error::Binding);
            }
        }
    }
    Ok(())
}
fn alias(dep: &Value) -> Result<&Value, Error> {
    match dep.get("explicit_name_in_toml")? {
        Some(v) if !v.is_null() => Ok(v),
        _ => dep.required("name"),
    }
}
fn same(a: &Value, b: &Value) -> Result<bool, Error> {
    a.with_text(|a| b.with_text(|b| a == b))?
}
fn kind(d: &Value) -> Result<u8, Error> {
    match d.get("kind")?.filter(|v| !v.is_null()) {
        None => Ok(0),
        Some(v) => v.with_text(|s| match s {
            "normal" => Ok(0),
            "dev" => Ok(1),
            "build" => Ok(2),
            _ => Err(Error::Value),
        })?,
    }
}
fn same_optional(a: &Value, b: &Value, key: &str) -> Result<bool, Error> {
    let a = a.get(key)?.filter(|v| !v.is_null());
    let b = b.get(key)?.filter(|v| !v.is_null());
    match (a, b) {
        (None, None) => Ok(true),
        (Some(a), Some(b)) => same(a, b),
        _ => Ok(false),
    }
}
fn dependency(d: &Value) -> Result<(), Error> {
    d.visit_fields(|k, _| {
        if [
            "name",
            "version_req",
            "features",
            "optional",
            "default_features",
            "target",
            "kind",
            "registry",
            "explicit_name_in_toml",
        ]
        .contains(&k)
        {
            Ok(())
        } else {
            Err(Error::Schema)
        }
    })?;
    text(d.required("name")?, 64, |s| CrateName::new(s).is_ok())?;
    text(d.required("version_req")?, 1024, |s| {
        semver::VersionReq::parse(s).is_ok()
    })?;
    strings(d.required("features")?, 300, 256, feature)?;
    d.required("optional")?.boolean()?;
    d.required("default_features")?.boolean()?;
    optional(d, "kind", 6, |s| matches!(s, "normal" | "build" | "dev"))?;
    optional(d, "target", 1024, super::target::valid)?;
    optional(d, "registry", 4096, url)?;
    optional(d, "explicit_name_in_toml", 64, dependency_name)
}
