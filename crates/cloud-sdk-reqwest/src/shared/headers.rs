use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};
use reqwest::header::HeaderMap;

use super::TransportError;

pub(crate) fn capture_response_headers(
    source: &HeaderMap,
) -> Result<ResponseHeaders, TransportError> {
    let mut captured = ResponseHeaders::new();
    for (name, value) in source {
        let sensitivity = response_sensitivity(name.as_str());
        captured
            .try_push(name.as_str(), value.as_bytes(), sensitivity)
            .map_err(|_| TransportError::InvalidResponseHeaders)?;
    }
    Ok(captured)
}

fn response_sensitivity(name: &str) -> HeaderSensitivity {
    if [
        "authentication-info",
        "proxy-authenticate",
        "set-cookie",
        "www-authenticate",
    ]
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
    fn captures_ordered_metadata_and_marks_known_secrets() {
        let mut source = HeaderMap::new();
        source.insert("x-request-id", HeaderValue::from_static("abc"));
        source.insert("set-cookie", HeaderValue::from_static("secret=1"));
        let captured = capture_response_headers(&source);
        assert!(captured.is_ok());
        if let Ok(captured) = captured {
            assert_eq!(captured.len(), 2);
            assert_eq!(
                captured
                    .get("set-cookie")
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
            assert_eq!(
                capture_response_headers(&source),
                Err(TransportError::InvalidResponseHeaders)
            );
        }
    }
}
