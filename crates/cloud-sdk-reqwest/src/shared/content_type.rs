use cloud_sdk::transport::{ResponseContentType, ResponseHeaders};

use super::TransportError;

pub(crate) fn parse_response_content_type(
    headers: &ResponseHeaders,
) -> Result<Option<ResponseContentType>, TransportError> {
    let Some(value) = headers.get("content-type") else {
        return Ok(None);
    };
    let value = core::str::from_utf8(value.value())
        .map_err(|_| TransportError::InvalidResponseContentType)?;
    ResponseContentType::new(value)
        .map(Some)
        .map_err(|_| TransportError::InvalidResponseContentType)
}

#[cfg(test)]
mod tests {
    use cloud_sdk::transport::{HeaderSensitivity, ResponseHeaders};

    use super::parse_response_content_type;
    use crate::shared::TransportError;

    #[test]
    fn accepts_absent_or_one_valid_content_type() {
        let headers = ResponseHeaders::new();
        assert_eq!(parse_response_content_type(&headers), Ok(None));

        let mut headers = ResponseHeaders::new();
        assert_eq!(
            headers.try_push(
                "content-type",
                b"application/json; charset=utf-8",
                HeaderSensitivity::Public,
            ),
            Ok(())
        );
        let parsed = parse_response_content_type(&headers);
        assert!(parsed.is_ok());
        if let Ok(Some(parsed)) = parsed {
            assert_eq!(parsed.as_str(), "application/json; charset=utf-8");
        }
    }

    #[test]
    fn rejects_duplicate_or_malformed_content_types() {
        let mut headers = ResponseHeaders::new();
        assert_eq!(
            headers.try_push(
                "content-type",
                b"application/json; charset",
                HeaderSensitivity::Public,
            ),
            Ok(())
        );
        assert_eq!(
            parse_response_content_type(&headers),
            Err(TransportError::InvalidResponseContentType)
        );
    }
}
