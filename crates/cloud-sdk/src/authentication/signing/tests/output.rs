use core::cell::Cell;

use crate::authentication::{RequestSigner, SigningOutputError};
#[cfg(feature = "std")]
use crate::std as test_std;
use crate::transport::{ContentType, RequestHeader};

use super::{
    MAX_CANONICAL_SIGNING_INPUT_BYTES, MAX_SIGNING_BODY_DIGEST_BYTES, SigningHeaders, canonical,
    context, debug_text, request,
};

struct FixedSigner<'a>(&'a Cell<usize>);

impl RequestSigner for FixedSigner<'_> {
    type Error = ();

    fn sign(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.set(self.0.get().saturating_add(1));
        let target = output.get_mut(..4).ok_or(())?;
        target.copy_from_slice(b"sig!");
        Ok(4)
    }
}

struct LengthSigner(Result<usize, ()>);

impl RequestSigner for LengthSigner {
    type Error = ();

    fn sign(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xa5);
        self.0
    }
}

#[cfg(feature = "std")]
struct PanickingSigner;

#[cfg(feature = "std")]
impl RequestSigner for PanickingSigner {
    type Error = ();

    fn sign(&self, _input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xa5);
        test_std::panic::resume_unwind(test_std::boxed::Box::new(()))
    }
}

#[test]
fn signer_output_is_validated_retains_request_and_clears() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"exact-body") else {
        return;
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        return;
    };
    let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
        return;
    };
    let mut digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut input = [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES];
    let Ok(canonical) = canonical(request, context, headers, &mut digest, &mut input) else {
        return;
    };
    let calls = Cell::new(0);
    let mut signature = [0xa5_u8; 16];
    {
        let Ok(signed) = canonical.sign_into(&FixedSigner(&calls), &mut signature) else {
            return;
        };
        assert_eq!(signed.request().body(), b"exact-body");
        assert_eq!(signed.context().service().as_str(), "robot");
        assert_eq!(signed.signature(), b"sig!");
        assert!(!debug_text(&signed).as_str().contains("sig!"));
    }
    assert_eq!(calls.get(), 1);
    assert_eq!(signature, [0_u8; 16]);
    assert_eq!(input, [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES]);
}

#[test]
fn signer_failures_and_invalid_lengths_clear_complete_output() {
    for result in [Err(()), Ok(0), Ok(17)] {
        let entries = [RequestHeader::content_type(ContentType::JSON)];
        let Some(request) = request("/objects", &entries, b"body") else {
            return;
        };
        let Ok(headers) = SigningHeaders::new(&entries) else {
            return;
        };
        let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
            return;
        };
        let mut digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
        let mut input = [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES];
        let Ok(canonical) = canonical(request, context, headers, &mut digest, &mut input) else {
            return;
        };
        let mut signature = [0xff_u8; 16];
        let error = canonical.sign_into(&LengthSigner(result), &mut signature);
        assert!(matches!(
            error,
            Err(SigningOutputError::Signer(_))
                | Err(SigningOutputError::Empty)
                | Err(SigningOutputError::TooLong)
        ));
        drop(error);
        assert_eq!(signature, [0_u8; 16]);
    }
}

#[test]
#[cfg(feature = "std")]
fn signer_panic_clears_signature_and_canonical_input() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"body") else {
        return;
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        return;
    };
    let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
        return;
    };
    let mut digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut input = [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES];
    let Ok(canonical) = canonical(request, context, headers, &mut digest, &mut input) else {
        return;
    };
    let mut signature = [0xff_u8; 16];
    let unwind = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let _ = canonical.sign_into(&PanickingSigner, &mut signature);
    }));
    assert!(unwind.is_err());
    assert_eq!(signature, [0_u8; 16]);
    assert_eq!(input, [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES]);
}
