//! Adversarial v0.38 response cleanup lifecycle tests.

use core::cell::Cell;
use core::future::{Future, pending};
use core::task::{Context, Poll, Waker};

use cloud_sdk::operation::{
    CheckedResponseGuard, ContentTypePolicy, RequestIdPolicy, ResponseBodyPolicy, ResponsePolicy,
    ResponsePolicyError,
};
use cloud_sdk::transport::{
    HeaderSensitivity, MediaType, ResponseBuffer, ResponseMetadata, ResponseStorageSanitizer,
    RetainedMetadataError, StatusCode,
};

static OK: [StatusCode; 1] = [StatusCode::OK];
static JSON: [MediaType<'static>; 1] = [MediaType::JSON];

#[test]
fn every_request_id_policy_has_an_explicit_lifecycle() -> Result<(), &'static str> {
    let mut protected_storage = [0xa5_u8; 32];
    let mut protected_headers = [0xa5_u8; 8192];
    let mut protected = checked_with_request_id(
        &mut protected_storage,
        &mut protected_headers,
        RequestIdPolicy::Protected,
        b"protected-id",
    )?;
    assert!(protected.with_borrowed(|view| {
        view.with_request_id(|request_id| request_id == Some(b"protected-id".as_slice()))
    }));
    let mut forbidden_retention = [0xa5_u8; 64];
    assert!(matches!(
        protected.retain_metadata_into(&mut forbidden_retention, 64),
        Err(RetainedMetadataError::RetentionForbidden)
    ));
    drop(protected);
    assert_eq!(protected_storage, [0_u8; 32]);
    assert_eq!(protected_headers, [0_u8; 8192]);

    let mut discarded_storage = [0xa5_u8; 32];
    let mut discarded_headers = [0xa5_u8; 8192];
    let discarded = checked_with_request_id(
        &mut discarded_storage,
        &mut discarded_headers,
        RequestIdPolicy::Discard,
        b"discarded-id",
    )?;
    assert!(
        discarded.with_borrowed(|view| { view.with_request_id(|request_id| request_id.is_none()) })
    );
    drop(discarded);
    assert_eq!(discarded_storage, [0_u8; 32]);
    assert_eq!(discarded_headers, [0_u8; 8192]);

    let mut retained_body = [0xa5_u8; 32];
    let mut retained_headers = [0xa5_u8; 8192];
    let mut retained_guard = checked_with_request_id(
        &mut retained_body,
        &mut retained_headers,
        RequestIdPolicy::Retain,
        b"retained-id",
    )?;
    let mut retained_storage = [0xa5_u8; 64];
    let retained_pointer = retained_storage.as_ptr();
    let retained = retained_guard
        .retain_metadata_into(&mut retained_storage, 64)
        .map_err(|_| "retainable request ID did not transfer")?;
    assert!(
        retained_guard
            .with_borrowed(|view| { view.with_request_id(|request_id| request_id.is_none()) })
    );
    assert!(
        retained.with_request_id(|request_id| { request_id == Some(b"retained-id".as_slice()) })
    );
    assert!(retained.with_request_id(|request_id| {
        request_id.is_some_and(|request_id| request_id.as_ptr() == retained_pointer)
    }));
    drop(retained_guard);
    assert_eq!(retained_body, [0_u8; 32]);
    assert_eq!(retained_headers, [0_u8; 8192]);
    drop(retained);
    assert_eq!(retained_storage, [0_u8; 64]);
    Ok(())
}

#[test]
fn rejected_retention_clears_the_source_and_invalid_ids_fail_closed() -> Result<(), &'static str> {
    let mut storage = [0xa5_u8; 32];
    let mut headers = [0xa5_u8; 8192];
    let mut checked = checked_with_request_id(
        &mut storage,
        &mut headers,
        RequestIdPolicy::Retain,
        b"too-long",
    )?;
    let mut retention = [0xa5_u8; 64];
    assert!(matches!(
        checked.retain_metadata_into(&mut retention, 3),
        Err(RetainedMetadataError::RetentionLimitExceeded)
    ));
    assert!(
        checked.with_borrowed(|view| { view.with_request_id(|request_id| request_id.is_none()) })
    );
    drop(checked);
    assert_eq!(storage, [0_u8; 32]);
    assert_eq!(headers, [0_u8; 8192]);

    let mut invalid_storage = [0xa5_u8; 32];
    let mut invalid_headers = [0xa5_u8; 8192];
    let invalid = checked_with_request_id(
        &mut invalid_storage,
        &mut invalid_headers,
        RequestIdPolicy::Protected,
        b"",
    );
    assert!(matches!(
        invalid,
        Err("response policy rejected request identifier")
    ));
    drop(invalid);
    assert_eq!(invalid_storage, [0_u8; 32]);
    assert_eq!(invalid_headers, [0_u8; 8192]);
    Ok(())
}

#[test]
fn mandatory_clear_survives_noop_and_recontaminating_hooks() {
    for hook in [
        &NoopHook as &dyn ResponseStorageSanitizer,
        &RecontaminatingHook as &dyn ResponseStorageSanitizer,
    ] {
        let mut storage = [0xa5_u8; 32];
        let mut headers = [0xa5_u8; 8192];
        {
            let mut response =
                ResponseBuffer::with_additive_sanitizer(&mut storage, 16, &mut headers, hook);
            if let Ok(mut attempt) = response.writer().begin_attempt()
                && let Ok(body) = attempt.body_mut()
            {
                body.fill(0x42);
            }
        }
        assert_eq!(storage, [0_u8; 32]);
        assert_eq!(headers, [0_u8; 8192]);
    }
}

#[test]
fn mandatory_final_clear_runs_when_an_additive_hook_panics() {
    let mut admission_storage = [0xa5_u8; 32];
    let mut admission_headers = [0xa5_u8; 8192];
    let admission_unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _response = ResponseBuffer::with_additive_sanitizer(
            &mut admission_storage,
            16,
            &mut admission_headers,
            &PanickingHook,
        );
    }));
    assert!(admission_unwind.is_err());
    assert_eq!(admission_storage, [0_u8; 32]);
    assert_eq!(admission_headers, [0_u8; 8192]);

    let hook = PanicOnDropHook {
        admitted: Cell::new(false),
    };
    let mut drop_storage = [0xa5_u8; 32];
    let mut drop_headers = [0xa5_u8; 8192];
    let drop_unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let response = ResponseBuffer::with_additive_sanitizer(
            &mut drop_storage,
            16,
            &mut drop_headers,
            &hook,
        );
        drop(response);
    }));
    assert!(drop_unwind.is_err());
    assert_eq!(drop_storage, [0_u8; 32]);
    assert_eq!(drop_headers, [0_u8; 8192]);
}

#[test]
fn panic_unwind_clears_response_attempt_state() {
    let mut storage = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 8192];
    let mut response = ResponseBuffer::new(&mut storage, 8, &mut headers);
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let mut attempt = response
            .writer()
            .begin_attempt()
            .unwrap_or_else(|_| unreachable!());
        attempt
            .body_mut()
            .unwrap_or_else(|_| unreachable!())
            .fill(0x5a);
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!())
            .try_push("x-request-id", b"partial", HeaderSensitivity::Sensitive)
            .unwrap_or_else(|_| unreachable!());
        std::panic::resume_unwind(std::boxed::Box::new("intentional response-attempt unwind"));
    }));
    assert!(unwind.is_err());
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!());
    assert!(
        attempt
            .body_mut()
            .is_ok_and(|body| body.iter().all(|byte| *byte == 0))
    );
    assert!(attempt.headers().is_empty());
}

#[test]
fn dirty_decode_workspace_clears_on_error_and_panic_unwind() -> Result<(), &'static str> {
    let mut success_storage = [0xa5_u8; 32];
    let mut success_headers = [0xa5_u8; 8192];
    let checked = checked_with_request_id(
        &mut success_storage,
        &mut success_headers,
        RequestIdPolicy::Discard,
        b"ignored",
    )?;
    let decoded = checked.decode_owned_with_workspace(|_response, workspace| {
        workspace.decoder_scratch_mut().fill(0x01);
        workspace.cursor_scratch_mut().fill(0x02);
        workspace.provider_link_scratch_mut().fill(0x03);
        Ok::<_, &'static str>(())
    });
    assert_eq!(decoded, Ok(()));
    assert_eq!(success_storage, [0_u8; 32]);

    let mut error_storage = [0xa5_u8; 32];
    let mut error_headers = [0xa5_u8; 8192];
    let checked = checked_with_request_id(
        &mut error_storage,
        &mut error_headers,
        RequestIdPolicy::Discard,
        b"ignored",
    )?;
    let decoded = checked.decode_owned_with_workspace(|_response, workspace| {
        workspace.decoder_scratch_mut().fill(0x11);
        workspace.cursor_scratch_mut().fill(0x22);
        workspace.provider_link_scratch_mut().fill(0x33);
        Err::<(), _>("decode failed")
    });
    assert_eq!(decoded, Err("decode failed"));
    assert_eq!(error_storage, [0_u8; 32]);

    let mut panic_storage = [0xa5_u8; 32];
    let mut panic_headers = [0xa5_u8; 8192];
    let checked = checked_with_request_id(
        &mut panic_storage,
        &mut panic_headers,
        RequestIdPolicy::Discard,
        b"ignored",
    )?;
    let unwind = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _: Result<(), ()> = checked.decode_owned_with_workspace(|_response, workspace| {
            workspace.decoder_scratch_mut().fill(0x44);
            std::panic::resume_unwind(std::boxed::Box::new("intentional cleanup unwind"));
        });
    }));
    assert!(unwind.is_err());
    assert_eq!(panic_storage, [0_u8; 32]);
    Ok(())
}

#[test]
fn dropping_a_pending_future_clears_its_owned_response_buffer() {
    let mut storage = [0xa5_u8; 32];
    let mut headers = [0xa5_u8; 8192];
    {
        let mut future = core::pin::pin!(pending_response(&mut storage, &mut headers));
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    assert_eq!(storage, [0_u8; 32]);
    assert_eq!(headers, [0_u8; 8192]);
}

async fn pending_response(storage: &mut [u8], header_storage: &mut [u8]) {
    let mut response = ResponseBuffer::new(storage, 16, header_storage);
    if let Ok(mut attempt) = response.writer().begin_attempt() {
        if let Ok(body) = attempt.body_mut() {
            body.fill(0x5a);
        }
        pending::<()>().await;
    }
}

fn checked_with_request_id<'a>(
    storage: &'a mut [u8],
    header_storage: &'a mut [u8],
    request_id_policy: RequestIdPolicy,
    request_id: &[u8],
) -> Result<CheckedResponseGuard<'a>, &'static str> {
    let mut response = ResponseBuffer::new(storage, 16, header_storage);
    {
        let mut attempt = response
            .writer()
            .begin_attempt()
            .map_err(|_| "response attempt was unavailable")?;
        let headers = attempt
            .headers_mut()
            .map_err(|_| "response headers were unavailable")?;
        headers
            .try_push("x-request-id", request_id, HeaderSensitivity::Sensitive)
            .map_err(|_| "request identifier header was invalid")?;
        headers
            .try_push(
                "content-type",
                b"application/json",
                HeaderSensitivity::Public,
            )
            .map_err(|_| "content type header was invalid")?;
        attempt
            .body_mut()
            .map_err(|_| "response body was unavailable")?
            .get_mut(..2)
            .ok_or("response body was too small")?
            .copy_from_slice(b"{}");
        attempt
            .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
            .map_err(|_| "response commitment failed")?;
    }
    json_policy()
        .validate(response, request_id_policy)
        .map_err(|error| match error {
            ResponsePolicyError::InvalidRequestId => "response policy rejected request identifier",
            _ => "response policy rejected fixture",
        })
}

fn json_policy() -> ResponsePolicy {
    ResponsePolicy::new(
        &OK,
        ContentTypePolicy::Required(&JSON),
        ResponseBodyPolicy::Required,
        16,
    )
    .unwrap_or_else(|_| unreachable!())
}

struct NoopHook;

impl ResponseStorageSanitizer for NoopHook {
    fn sanitize_response_storage(&self, _response_storage: &mut [u8]) {}
}

struct RecontaminatingHook;

impl ResponseStorageSanitizer for RecontaminatingHook {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        response_storage.fill(0xa5);
    }
}

struct PanickingHook;

impl ResponseStorageSanitizer for PanickingHook {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        response_storage.fill(0x5a);
        std::panic::resume_unwind(std::boxed::Box::new("intentional additive cleanup panic"));
    }
}

struct PanicOnDropHook {
    admitted: Cell<bool>,
}

impl ResponseStorageSanitizer for PanicOnDropHook {
    fn sanitize_response_storage(&self, response_storage: &mut [u8]) {
        if self.admitted.replace(true) {
            response_storage.fill(0x7a);
            std::panic::resume_unwind(std::boxed::Box::new("intentional drop cleanup panic"));
        }
    }
}
