//! Source-locked path-template validation.

use cloud_sdk::transport::{RequestPath, RequestTarget};

use super::HetznerPreparationError;

pub(super) fn validate_target(storage: &[u8], len: usize) -> Result<(), HetznerPreparationError> {
    let text = storage
        .get(..len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok())
        .ok_or(HetznerPreparationError::Path)?;
    RequestTarget::new(text)
        .map(|_| ())
        .map_err(HetznerPreparationError::InvalidTarget)
}

pub(super) fn validate_or_clear(
    target: &mut [u8],
    path_len: usize,
    template: &str,
    body: &mut [u8],
) -> Result<(), HetznerPreparationError> {
    let path = target
        .get(..path_len)
        .and_then(|bytes| core::str::from_utf8(bytes).ok());
    if path.is_some_and(|path| RequestPath::new(path).is_ok() && matches(path, template)) {
        return Ok(());
    }
    cloud_sdk_sanitization::sanitize_bytes(target);
    cloud_sdk_sanitization::sanitize_bytes(body);
    Err(HetznerPreparationError::Path)
}

pub(super) fn matches(path: &str, template: &str) -> bool {
    if !path.starts_with('/') || !template.starts_with('/') {
        return false;
    }
    let mut path_segments = path.split('/');
    let mut template_segments = template.split('/');
    loop {
        match (path_segments.next(), template_segments.next()) {
            (Some(actual), Some(expected)) => {
                let placeholder = expected.starts_with('{') && expected.ends_with('}');
                if placeholder {
                    if actual.is_empty() {
                        return false;
                    }
                } else if actual != expected {
                    return false;
                }
            }
            (None, None) => return true,
            _ => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn accepts_bound_segments_and_rejects_wrong_routes() {
        assert!(super::matches(
            "/zones/example/rrsets/www/A",
            "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}"
        ));
        assert!(!super::matches(
            "/zones/example/rrsets/www",
            "/zones/{id_or_name}/rrsets/{rr_name}/{rr_type}"
        ));
        assert!(!super::matches(
            "/servers/7/actions/poweroff",
            "/servers/{id}/actions/poweron"
        ));
        assert!(!super::matches(
            "/servers//metrics",
            "/servers/{id}/metrics"
        ));
    }

    #[test]
    fn mismatch_clears_complete_caller_storage() {
        let mut target = [0x41; 24];
        let mut body = [0x42; 12];
        assert!(super::validate_or_clear(&mut target, 10, "/servers/{id}", &mut body).is_err());
        assert_eq!(target, [0; 24]);
        assert_eq!(body, [0; 12]);
    }

    #[test]
    fn endpoint_path_rejects_raw_and_encoded_delimiters() {
        for path in [
            "/servers/7?x=1",
            "/servers/7#fragment",
            "/servers/7%3Fx",
            "/servers/7%23fragment",
        ] {
            let mut target = [0; 32];
            let Some(output) = target.get_mut(..path.len()) else {
                unreachable!("path fixture exceeds target storage");
            };
            output.copy_from_slice(path.as_bytes());
            let mut body = [0x42; 8];
            assert!(
                super::validate_or_clear(&mut target, path.len(), "/servers/{id}", &mut body,)
                    .is_err()
            );
            assert_eq!(target, [0; 32]);
            assert_eq!(body, [0; 8]);
        }
    }
}
