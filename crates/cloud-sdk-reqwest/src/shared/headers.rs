use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
use reqwest::header::{HeaderMap, HeaderValue};

use super::TransportError;

const REVIEWED_PUBLIC_RESPONSE_HEADERS: &[&str] = &[
    "content-length",
    "content-type",
    "date",
    "ratelimit-limit",
    "ratelimit-remaining",
    "ratelimit-reset",
];

pub(crate) fn capture_response_headers(
    source: &HeaderMap,
    captured: &mut ResponseHeaders<'_>,
) -> Result<(), TransportError> {
    for (name, value) in source {
        let sensitivity = response_sensitivity(name.as_str(), value);
        captured
            .try_push(name.as_str(), value.as_bytes(), sensitivity)
            .map_err(|_| TransportError::InvalidResponseHeaders)?;
    }
    Ok(())
}

fn response_sensitivity(name: &str, value: &HeaderValue) -> HeaderSensitivity {
    if value.is_sensitive()
        || !REVIEWED_PUBLIC_RESPONSE_HEADERS
            .iter()
            .any(|candidate| name.eq_ignore_ascii_case(candidate))
    {
        HeaderSensitivity::Sensitive
    } else {
        HeaderSensitivity::Public
    }
}

#[cfg(test)]
mod tests {
    use reqwest::header::{HeaderMap, HeaderValue};

    use super::capture_response_headers;
    use crate::shared::TransportError;
    use cloud_sdk::transport::HeaderSensitivity;

    #[test]
    fn defaults_unknown_metadata_to_sensitive() {
        let mut source = HeaderMap::new();
        source.insert("x-request-id", HeaderValue::from_static("abc"));
        source.insert("x-api-key", HeaderValue::from_static("secret-key"));
        source.insert("authorization", HeaderValue::from_static("Bearer secret"));
        source.insert("set-cookie", HeaderValue::from_static("secret=1"));
        let mut storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
        let mut headers = cloud_sdk::transport::ResponseHeaders::new(&mut storage);
        let captured = capture_response_headers(&source, &mut headers);
        assert!(captured.is_ok());
        if captured.is_ok() {
            assert_eq!(headers.len(), 4);
            for name in ["x-request-id", "x-api-key", "authorization", "set-cookie"] {
                assert_eq!(
                    headers.get(name).map(|header| header.sensitivity()),
                    Some(HeaderSensitivity::Sensitive)
                );
            }
        }
    }

    #[test]
    fn classifies_only_reviewed_metadata_as_public() {
        for name in super::REVIEWED_PUBLIC_RESPONSE_HEADERS {
            let mut source = HeaderMap::new();
            source.insert(*name, HeaderValue::from_static("1"));
            let mut storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
            let mut headers = cloud_sdk::transport::ResponseHeaders::new(&mut storage);
            let captured = capture_response_headers(&source, &mut headers);
            assert!(captured.is_ok());
            if captured.is_ok() {
                assert_eq!(
                    headers.get(name).map(|header| header.sensitivity()),
                    Some(HeaderSensitivity::Public)
                );
            }
        }
    }

    #[test]
    fn preserves_explicit_sensitivity_on_reviewed_public_metadata() {
        let mut source = HeaderMap::new();
        let mut value = HeaderValue::from_static("application/json");
        value.set_sensitive(true);
        source.insert("content-type", value);

        let mut storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
        let mut headers = cloud_sdk::transport::ResponseHeaders::new(&mut storage);
        let captured = capture_response_headers(&source, &mut headers);
        assert!(captured.is_ok());
        if captured.is_ok() {
            assert_eq!(
                headers
                    .get("content-type")
                    .map(|header| header.sensitivity()),
                Some(HeaderSensitivity::Sensitive)
            );
        }
    }

    #[test]
    fn rejects_identical_and_conflicting_duplicates() {
        for duplicate in ["same", "different"] {
            let mut source = HeaderMap::new();
            source.append("x-test", HeaderValue::from_static("same"));
            source.append(
                "x-test",
                HeaderValue::from_str(duplicate).unwrap_or(HeaderValue::from_static("invalid")),
            );
            let mut storage = [0_u8; cloud_sdk::transport::MAX_RESPONSE_HEADER_BYTES];
            let mut headers = cloud_sdk::transport::ResponseHeaders::new(&mut storage);
            assert!(matches!(
                capture_response_headers(&source, &mut headers),
                Err(TransportError::InvalidResponseHeaders)
            ));
        }
    }
}
