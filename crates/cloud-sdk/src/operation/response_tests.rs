use super::{
    ContentTypePolicy, RequestIdPolicy, ResponseBodyPolicy, ResponsePolicy, ResponsePolicyError,
};
use crate::Method;
use crate::transport::{
    BlockingTransport, HeaderSensitivity, MediaType, RequestTarget, ResponseBuffer,
    ResponseMetadata, ResponseWriter, StatusCode, TransportRequest,
};

static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

#[test]
fn response_policy_classifies_every_rejection_before_decoding() {
    let required = json_response_policy(4);
    assert!(required.is_ok());
    let Ok(required) = required else { return };
    let status = StatusCode::new(201).unwrap_or(StatusCode::OK);
    assert!(matches!(
        validate_fixture(required, status, b"{}", None),
        Err(ResponsePolicyError::UnexpectedStatus)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"12345", None),
        Err(ResponsePolicyError::BodyTooLarge)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"", None),
        Err(ResponsePolicyError::MissingBody)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"{}", None),
        Err(ResponsePolicyError::MissingContentType)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"{}", Some(b"text/plain")),
        Err(ResponsePolicyError::UnexpectedContentType)
    ));
    assert_eq!(
        validate_fixture(
            required,
            StatusCode::OK,
            b"{}",
            Some(b"application/json; charset=utf-8"),
        ),
        Ok(2)
    );

    let forbidden = ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Forbidden,
        ResponseBodyPolicy::Forbidden,
        0,
    );
    assert!(forbidden.is_ok());
    if let Ok(forbidden) = forbidden {
        assert!(matches!(
            validate_fixture(forbidden, StatusCode::OK, b"x", None),
            Err(ResponsePolicyError::ForbiddenBody)
        ));
        assert!(matches!(
            validate_fixture(forbidden, StatusCode::OK, b"", Some(b"application/json")),
            Err(ResponsePolicyError::ForbiddenContentType)
        ));
        for malformed in [
            b"application/json; charset".as_slice(),
            b"application/json\xff".as_slice(),
        ] {
            assert!(matches!(
                validate_fixture(forbidden, StatusCode::OK, b"", Some(malformed)),
                Err(ResponsePolicyError::InvalidContentType)
            ));
        }
    }

    let optional = ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Optional(&JSON_MEDIA),
        ResponseBodyPolicy::Optional,
        4,
    );
    assert!(optional.is_ok());
    if let Ok(optional) = optional {
        for malformed in [
            b"application/json; charset".as_slice(),
            b"application/json\xff".as_slice(),
        ] {
            assert!(matches!(
                validate_fixture(optional, StatusCode::OK, b"{}", Some(malformed)),
                Err(ResponsePolicyError::InvalidContentType)
            ));
        }
    }
}

fn json_response_policy(
    max_body_bytes: usize,
) -> Result<ResponsePolicy, super::ResponsePolicyValidationError> {
    ResponsePolicy::new(
        &OK_STATUS,
        ContentTypePolicy::Required(&JSON_MEDIA),
        ResponseBodyPolicy::Required,
        max_body_bytes,
    )
}

fn validate_fixture(
    policy: ResponsePolicy,
    status: StatusCode,
    body: &[u8],
    content_type: Option<&[u8]>,
) -> Result<usize, ResponsePolicyError> {
    let mut storage = [0_u8; 32];
    let mut header_storage = [0_u8; 8192];
    let mut response = ResponseBuffer::new(&mut storage, 32, &mut header_storage);
    let target = RequestTarget::new("/").map_err(|_| ResponsePolicyError::UncommittedResponse)?;
    let transport = FixtureTransport {
        status,
        body,
        content_type,
    };
    BlockingTransport::send(
        &transport,
        TransportRequest::new(Method::Get, target),
        response.writer(),
    )
    .map_err(|_| ResponsePolicyError::UncommittedResponse)?;
    policy
        .validate(response, RequestIdPolicy::Discard)
        .map(|checked| checked.with_borrowed(|view| view.body().len()))
}

struct FixtureTransport<'a> {
    status: StatusCode,
    body: &'a [u8],
    content_type: Option<&'a [u8]>,
}

impl BlockingTransport for FixtureTransport<'_> {
    type Error = ();

    fn send(
        &self,
        _request: TransportRequest<'_>,
        response: &mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error> {
        if let Some(content_type) = self.content_type {
            response
                .headers_mut()
                .map_err(|_| ())?
                .try_push("content-type", content_type, HeaderSensitivity::Public)
                .map_err(|_| ())?;
        }
        response
            .body_mut()
            .map_err(|_| ())?
            .get_mut(..self.body.len())
            .ok_or(())?
            .copy_from_slice(self.body);
        response
            .commit(self.status, self.body.len(), ResponseMetadata::EMPTY)
            .map_err(|_| ())
    }
}
