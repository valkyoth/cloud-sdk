use super::{HttpsEndpoint, RawHttpError, SanitizedBody};
use cloud_sdk::transport::RequestTarget;

// RequestTarget validates origin form, percent encoding and dot segments;
// HttpsEndpoint validates the fixed prefix separately. Never round-trip a
// secret target through Url/String. The pinned http crate retains this owner.
pub(super) fn compose(
    endpoint: &HttpsEndpoint,
    target: RequestTarget<'_>,
) -> Result<http::Uri, RawHttpError> {
    // Preserve the previous URL normalizer's rejection of raw apostrophes in
    // provider-link queries (canonical/form queries already exclude them).
    if target
        .as_str()
        .split_once('?')
        .is_some_and(|(_, q)| q.contains('\''))
    {
        return Err(RawHttpError::TargetRejected);
    }
    // Keep the non-secret authority separate: Hyper may retain origin keys
    // longer than a request. Those keys must not share the token allocation.
    let base: http::Uri = endpoint
        .raw_prefix()
        .parse()
        .map_err(|_| RawHttpError::TargetRejected)?;
    let prefix = if base.path() == "/" { "" } else { base.path() };
    let owner = SanitizedBody::copy_parts(prefix.as_bytes(), target.as_str().as_bytes())
        .map_err(|_| RawHttpError::RequestBuildFailed)?;
    let bytes = owner.into_bytes();
    with_path(base, bytes)
}

fn with_path(base: http::Uri, bytes: bytes::Bytes) -> Result<http::Uri, RawHttpError> {
    let path = http::uri::PathAndQuery::from_maybe_shared(bytes.clone())
        .map_err(|_| RawHttpError::TargetRejected)?;
    // Check exact composition without allocating another copy of the target.
    if path.as_str().as_bytes() != bytes.as_ref() {
        return Err(RawHttpError::TargetRejected);
    }
    let mut parts = base.into_parts();
    parts.path_and_query = Some(path);
    http::Uri::from_parts(parts).map_err(|_| RawHttpError::TargetRejected)
}

#[cfg(test)]
mod link_tests;
#[cfg(test)]
mod tests;
