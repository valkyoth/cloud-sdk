use super::{
    ContentTypePolicy, RequestIdPolicy, ResponseBodyPolicy, ResponsePolicy, ResponsePolicyError,
};
use crate::transport::{
    MediaType, ResponseBuffer, ResponseContentType, ResponseMetadata, StatusCode,
};

static OK_STATUS: [StatusCode; 1] = [StatusCode::OK];
static JSON_MEDIA: [MediaType<'static>; 1] = [MediaType::JSON];

#[test]
fn response_policy_classifies_every_rejection_before_decoding() {
    let required = json_response_policy(4);
    assert!(required.is_ok());
    let Ok(required) = required else { return };
    let json = ResponseContentType::new("application/json; charset=utf-8");
    assert!(json.is_ok());
    let Ok(json) = json else { return };

    let status = StatusCode::new(201).unwrap_or(StatusCode::OK);
    assert!(matches!(
        validate_fixture(required, status, b"{}", ResponseMetadata::EMPTY),
        Err(ResponsePolicyError::UnexpectedStatus)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"12345", ResponseMetadata::EMPTY,),
        Err(ResponsePolicyError::BodyTooLarge)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"", ResponseMetadata::EMPTY),
        Err(ResponsePolicyError::MissingBody)
    ));
    assert!(matches!(
        validate_fixture(required, StatusCode::OK, b"{}", ResponseMetadata::EMPTY,),
        Err(ResponsePolicyError::MissingContentType)
    ));
    let text = ResponseContentType::new("text/plain");
    assert!(text.is_ok());
    if let Ok(text) = text {
        assert!(matches!(
            validate_fixture(
                required,
                StatusCode::OK,
                b"{}",
                ResponseMetadata::EMPTY.with_content_type(text),
            ),
            Err(ResponsePolicyError::UnexpectedContentType)
        ));
    }
    assert_eq!(
        validate_fixture(
            required,
            StatusCode::OK,
            b"{}",
            ResponseMetadata::EMPTY.with_content_type(json.retain_copy()),
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
            validate_fixture(forbidden, StatusCode::OK, b"x", ResponseMetadata::EMPTY,),
            Err(ResponsePolicyError::ForbiddenBody)
        ));
        assert!(matches!(
            validate_fixture(
                forbidden,
                StatusCode::OK,
                b"",
                ResponseMetadata::EMPTY.with_content_type(json),
            ),
            Err(ResponsePolicyError::ForbiddenContentType)
        ));
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
    metadata: ResponseMetadata,
) -> Result<usize, ResponsePolicyError> {
    let mut storage = [0_u8; 32];
    let mut response = ResponseBuffer::new(&mut storage, 32);
    let output = response
        .writer()
        .body_mut()
        .map_err(|_| ResponsePolicyError::UncommittedResponse)?;
    let initialized = output
        .get_mut(..body.len())
        .ok_or(ResponsePolicyError::BodyTooLarge)?;
    initialized.copy_from_slice(body);
    response
        .writer()
        .commit(status, body.len(), metadata)
        .map_err(|_| ResponsePolicyError::UncommittedResponse)?;
    policy
        .validate(response, RequestIdPolicy::Discard)
        .map(|checked| checked.with_borrowed(|view| view.body().len()))
}
