use crate::endpoint::ApiRequestTarget;
use crate::identifiers::{MAX_VERSION_BYTES, Version};
use cloud_sdk::Method;

// Source-locked API-token operations. Tests compare the complete table with
// CRATESIO_API_SCOPE.tsv so additions and authentication changes need review.
pub(super) const API_ROUTES: &[(Method, &str)] = &[
    (Method::Get, "/api/v1/crates"),
    (Method::Put, "/api/v1/crates/new"),
    (Method::Patch, "/api/v1/crates/{name}"),
    (Method::Delete, "/api/v1/crates/{name}/follow"),
    (Method::Put, "/api/v1/crates/{name}/follow"),
    (Method::Delete, "/api/v1/crates/{name}/owners"),
    (Method::Put, "/api/v1/crates/{name}/owners"),
    (Method::Patch, "/api/v1/crates/{name}/{version}"),
    (Method::Put, "/api/v1/crates/{name}/{version}/unyank"),
    (Method::Delete, "/api/v1/crates/{name}/{version}/yank"),
    (Method::Put, "/api/v1/me/crate_owner_invitations/{crate_id}"),
    (Method::Put, "/api/v1/me/email_notifications"),
    (Method::Delete, "/api/v1/me/tokens/{id}"),
    (Method::Get, "/api/v1/me/tokens/{id}"),
    (Method::Delete, "/api/v1/tokens/current"),
    (Method::Get, "/api/v1/trusted_publishing/github_configs"),
    (Method::Post, "/api/v1/trusted_publishing/github_configs"),
    (
        Method::Delete,
        "/api/v1/trusted_publishing/github_configs/{id}",
    ),
    (Method::Get, "/api/v1/trusted_publishing/gitlab_configs"),
    (Method::Post, "/api/v1/trusted_publishing/gitlab_configs"),
    (
        Method::Delete,
        "/api/v1/trusted_publishing/gitlab_configs/{id}",
    ),
    (Method::Put, "/api/v1/users/{id}/resend"),
    (Method::Put, "/api/v1/users/{user}"),
];

pub(super) fn api_allowed(method: Method, target: ApiRequestTarget<'_>) -> bool {
    let target = target.as_request_target();
    if target.query().is_present() && method != Method::Get {
        return false;
    }
    API_ROUTES.iter().any(|(expected, template)| {
        method == *expected && matches_path(template, target.path().as_str())
    })
}

fn matches_path(template: &str, candidate: &str) -> bool {
    let mut candidate = candidate.split('/');
    for part in template.split('/') {
        let Some(value) = candidate.next() else {
            return false;
        };
        if part == "{version}" {
            if !valid_encoded_version(value) {
                return false;
            }
        } else if part.starts_with('{') {
            if value.is_empty() || value == "." || value == ".." {
                return false;
            }
            let numeric = matches!(part, "{id}" | "{crate_id}" | "{user}");
            if !value.bytes().all(|byte| {
                if numeric {
                    byte.is_ascii_digit()
                } else {
                    byte.is_ascii_alphanumeric() || b"-_.+".contains(&byte)
                }
            }) {
                return false;
            }
        } else if part != value {
            return false;
        }
    }
    candidate.next().is_none()
}

fn valid_encoded_version(value: &str) -> bool {
    // Only the version encoder's canonical '+' escape is admitted. Other
    // escapes stay literal and fail SemVer validation; never decode twice.
    let mut decoded = [0; MAX_VERSION_BYTES];
    let mut cursor = 0usize;
    for (index, part) in value.split("%2B").enumerate() {
        if index != 0 {
            let Some(slot) = decoded.get_mut(cursor) else {
                return false;
            };
            *slot = b'+';
            let Some(next) = cursor.checked_add(1) else {
                return false;
            };
            cursor = next;
        }
        let Some(end) = cursor.checked_add(part.len()) else {
            return false;
        };
        let Some(output) = decoded.get_mut(cursor..end) else {
            return false;
        };
        output.copy_from_slice(part.as_bytes());
        cursor = end;
    }
    let Some(bytes) = decoded.get(..cursor) else {
        return false;
    };
    let Ok(version) = core::str::from_utf8(bytes) else {
        return false;
    };
    Version::new(version).is_ok()
}
