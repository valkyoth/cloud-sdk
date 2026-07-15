use cloud_sdk::transport::ResponseContentType;
use reqwest::header::{CONTENT_TYPE, HeaderMap};

use super::TransportError;

pub(crate) fn parse_response_content_type(
    headers: &HeaderMap,
) -> Result<Option<ResponseContentType>, TransportError> {
    let mut values = headers.get_all(CONTENT_TYPE).iter();
    let Some(value) = values.next() else {
        return Ok(None);
    };
    if values.next().is_some() {
        return Err(TransportError::InvalidResponseContentType);
    }
    let value = value
        .to_str()
        .map_err(|_| TransportError::InvalidResponseContentType)?;
    ResponseContentType::new(value)
        .map(Some)
        .map_err(|_| TransportError::InvalidResponseContentType)
}

#[cfg(test)]
mod tests {
    use reqwest::header::{CONTENT_TYPE, HeaderMap, HeaderValue};

    use super::parse_response_content_type;
    use crate::shared::TransportError;

    #[test]
    fn accepts_absent_or_one_valid_content_type() {
        let headers = HeaderMap::new();
        assert_eq!(parse_response_content_type(&headers), Ok(None));

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset=utf-8"),
        );
        let parsed = parse_response_content_type(&headers);
        assert!(parsed.is_ok());
        if let Ok(Some(parsed)) = parsed {
            assert_eq!(parsed.as_str(), "application/json; charset=utf-8");
        }
    }

    #[test]
    fn rejects_duplicate_or_malformed_content_types() {
        let mut headers = HeaderMap::new();
        headers.append(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.append(CONTENT_TYPE, HeaderValue::from_static("text/plain"));
        assert_eq!(
            parse_response_content_type(&headers),
            Err(TransportError::InvalidResponseContentType)
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_static("application/json; charset"),
        );
        assert_eq!(
            parse_response_content_type(&headers),
            Err(TransportError::InvalidResponseContentType)
        );
    }
}
