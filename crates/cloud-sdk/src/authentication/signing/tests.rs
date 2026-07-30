use core::fmt::{self, Write};

#[cfg(feature = "std")]
use crate::std as test_std;
use crate::transport::{
    ContentType, EndpointIdentity, EndpointScheme, HeaderSensitivity, RequestHeader,
    RequestHeaders, RequestTarget, TransportRequest,
};
use crate::{Method, ProviderId, ServiceId};

use super::{
    CanonicalSigningInput, MAX_CANONICAL_SIGNING_INPUT_BYTES, MAX_SIGNING_ALGORITHM_BYTES,
    MAX_SIGNING_BODY_DIGEST_BYTES, MAX_SIGNING_DIGEST_ALGORITHM_BYTES, MAX_SIGNING_KEY_ID_BYTES,
    MAX_SIGNING_NONCE_BYTES, RequestBodyHasher, SigningAlgorithm, SigningBuildError,
    SigningContext, SigningContextValueError, SigningDigestAlgorithm, SigningFreshness,
    SigningHeaders, SigningInputError, SigningKeyId, SigningNonce, SigningValueError, UnixTime,
};
use crate::authentication::ScopeValue;

mod context_domain;
mod output;
mod request_domain;

struct DebugBuffer {
    bytes: [u8; 128],
    len: usize,
}

impl DebugBuffer {
    const fn new() -> Self {
        Self {
            bytes: [0; 128],
            len: 0,
        }
    }

    fn as_str(&self) -> &str {
        core::str::from_utf8(self.bytes.get(..self.len).unwrap_or_default()).unwrap_or_default()
    }
}

impl Write for DebugBuffer {
    fn write_str(&mut self, value: &str) -> fmt::Result {
        let end = self.len.checked_add(value.len()).ok_or(fmt::Error)?;
        let output = self.bytes.get_mut(self.len..end).ok_or(fmt::Error)?;
        output.copy_from_slice(value.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn debug_text(value: &impl fmt::Debug) -> DebugBuffer {
    let mut output = DebugBuffer::new();
    let _ = write!(&mut output, "{value:?}");
    output
}

fn request<'a>(
    target: &'a str,
    entries: &'a [RequestHeader<'a>],
    body: &'a [u8],
) -> Option<TransportRequest<'a>> {
    let target = RequestTarget::new(target).ok()?;
    let headers = RequestHeaders::new(entries).ok()?;
    Some(
        TransportRequest::new(Method::Post, target)
            .with_headers(headers)
            .with_body(body),
    )
}

fn context<'a>(
    service: &'static str,
    host: &'a str,
    key_id: &'a str,
    algorithm: &'a str,
) -> Option<SigningContext<'a>> {
    let provider = ProviderId::new("hetzner").ok()?;
    let service = ServiceId::new(service).ok()?;
    let endpoint = EndpointIdentity::new(EndpointScheme::Https, host, 443, "/api").ok()?;
    let key_id = SigningKeyId::new(key_id).ok()?;
    let digest_algorithm = SigningDigestAlgorithm::new("body-sha256").ok()?;
    let signature_algorithm = SigningAlgorithm::new(algorithm).ok()?;
    Some(SigningContext::new(
        provider,
        service,
        endpoint,
        key_id,
        digest_algorithm,
        signature_algorithm,
    ))
}

struct BodyHasher;

impl RequestBodyHasher for BodyHasher {
    type Error = ();

    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_> {
        SigningDigestAlgorithm::new("body-sha256").unwrap_or_else(|_| unreachable!())
    }

    fn hash_body(&self, body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        let len = body.len().checked_add(1).ok_or(())?;
        let target = output.get_mut(..len).ok_or(())?;
        let Some(prefix) = target.first_mut() else {
            return Err(());
        };
        *prefix = u8::try_from(body.len()).map_err(|_| ())?;
        let remainder = target.get_mut(1..).ok_or(())?;
        remainder.copy_from_slice(body);
        Ok(len)
    }
}

struct FailingHasher;

impl RequestBodyHasher for FailingHasher {
    type Error = ();

    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_> {
        SigningDigestAlgorithm::new("body-sha256").unwrap_or_else(|_| unreachable!())
    }

    fn hash_body(&self, _body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xa5);
        Err(())
    }
}

struct LengthHasher(usize);

impl RequestBodyHasher for LengthHasher {
    type Error = ();

    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_> {
        SigningDigestAlgorithm::new("body-sha256").unwrap_or_else(|_| unreachable!())
    }

    fn hash_body(&self, _body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xa5);
        Ok(self.0)
    }
}

struct MismatchedHasher;

impl RequestBodyHasher for MismatchedHasher {
    type Error = ();

    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_> {
        SigningDigestAlgorithm::new("body-sha512").unwrap_or_else(|_| unreachable!())
    }

    fn hash_body(&self, _body: &[u8], _output: &mut [u8]) -> Result<usize, Self::Error> {
        Ok(1)
    }
}

#[cfg(feature = "std")]
struct PanickingHasher;

#[cfg(feature = "std")]
impl RequestBodyHasher for PanickingHasher {
    type Error = ();

    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_> {
        SigningDigestAlgorithm::new("body-sha256").unwrap_or_else(|_| unreachable!())
    }

    fn hash_body(&self, _body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        output.fill(0xa5);
        test_std::panic::resume_unwind(test_std::boxed::Box::new(()))
    }
}

fn canonical<'output, 'request>(
    request: TransportRequest<'request>,
    context: SigningContext<'request>,
    headers: SigningHeaders<'_>,
    digest: &mut [u8],
    output: &'output mut [u8],
) -> Result<CanonicalSigningInput<'output, 'request>, SigningBuildError<()>> {
    canonical_with_hasher(request, context, headers, &BodyHasher, digest, output)
}

fn canonical_with_hasher<'output, 'request>(
    request: TransportRequest<'request>,
    context: SigningContext<'request>,
    headers: SigningHeaders<'_>,
    hasher: &dyn RequestBodyHasher<Error = ()>,
    digest: &mut [u8],
    output: &'output mut [u8],
) -> Result<CanonicalSigningInput<'output, 'request>, SigningBuildError<()>> {
    let nonce = SigningNonce::new(b"nonce").unwrap_or_else(|_| unreachable!());
    let freshness = SigningFreshness::new(nonce, UnixTime::from_seconds(42));
    CanonicalSigningInput::new_hashed(request, context, headers, freshness, hasher, digest, output)
}

#[test]
fn context_and_nonce_values_are_bounded_and_redacted() {
    assert_eq!(SigningNonce::new(&[]), Err(SigningValueError::Empty));
    assert_eq!(
        SigningNonce::new(&[0; MAX_SIGNING_NONCE_BYTES + 1]),
        Err(SigningValueError::TooLong)
    );
    assert!(matches!(
        SigningKeyId::new("has space"),
        Err(SigningContextValueError::InvalidByte)
    ));
    assert!(matches!(
        SigningKeyId::new(&"a".repeat(MAX_SIGNING_KEY_ID_BYTES + 1)),
        Err(SigningContextValueError::TooLong)
    ));
    assert!(matches!(
        SigningDigestAlgorithm::new(&"a".repeat(MAX_SIGNING_DIGEST_ALGORITHM_BYTES + 1)),
        Err(SigningContextValueError::TooLong)
    ));
    assert!(matches!(
        SigningAlgorithm::new(&"a".repeat(MAX_SIGNING_ALGORITHM_BYTES + 1)),
        Err(SigningContextValueError::TooLong)
    ));
    let Some(context) = context("robot", "robot.example.test", "secret-key", "hmac-sha256") else {
        unreachable!("valid signing context must construct");
    };
    let debug = debug_text(&context);
    assert!(debug.as_str().contains("[redacted]"));
    assert!(!debug.as_str().contains("secret-key"));
}

#[test]
fn selected_headers_must_be_sorted_unique_and_match_the_request() {
    let first = RequestHeader::new("accept", "application/json");
    let second = RequestHeader::content_type(ContentType::JSON);
    let (Ok(first), second) = (first, second) else {
        unreachable!("valid request headers must construct");
    };
    let ordered = [first, second];
    assert!(SigningHeaders::new(&ordered).is_ok());
    assert!(matches!(
        SigningHeaders::new(&[second, first]),
        Err(SigningInputError::HeaderOrder)
    ));
    assert!(matches!(
        SigningHeaders::new(&[first, first]),
        Err(SigningInputError::HeaderOrder)
    ));
    let Some(request) = request("/objects", &ordered, b"{}") else {
        unreachable!("valid transport request must construct");
    };
    let Ok(changed) = RequestHeader::new("accept", "text/plain") else {
        unreachable!("valid changed header must construct");
    };
    let changed = [changed];
    let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
        unreachable!("valid signing context must construct");
    };
    let mut digest = [0xa5_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut output = [0xa5_u8; 512];
    assert!(matches!(
        canonical(
            request,
            context,
            SigningHeaders::new(&changed).unwrap_or_else(|_| unreachable!()),
            &mut digest,
            &mut output,
        ),
        Err(SigningBuildError::Input(SigningInputError::HeaderMismatch))
    ));
    assert_eq!(digest, [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES]);
    assert_eq!(output, [0xa5_u8; 512]);
}

#[test]
fn canonical_v2_vector_binds_complete_security_domain() {
    let accept = RequestHeader::new("Accept", "application/json");
    let content_type = RequestHeader::content_type(ContentType::JSON);
    let (Ok(accept), content_type) = (accept, content_type) else {
        unreachable!("valid request headers must construct");
    };
    let entries = [accept, content_type];
    let Some(request) = request("/objects?limit=2", &entries, b"{}") else {
        unreachable!("valid transport request must construct");
    };
    let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
        unreachable!("valid signing context must construct");
    };
    let Some(audience) = ScopeValue::new("aud").ok() else {
        unreachable!("valid audience must construct");
    };
    let Some(tenant) = ScopeValue::new("tenant").ok() else {
        unreachable!("valid tenant must construct");
    };
    let context = context.with_audience(audience).with_tenant(tenant);
    let mut digest = [0xa5_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut output = [0xa5_u8; 512];
    {
        let result = canonical(
            request,
            context,
            SigningHeaders::new(&entries).unwrap_or_else(|_| unreachable!()),
            &mut digest,
            &mut output,
        );
        let Ok(canonical) = result else {
            unreachable!("canonical signing input must construct");
        };
        let expected = [
            b"cloud-sdk-signing-v2\0".as_slice(),
            &[7],
            b"hetzner",
            &[5],
            b"robot",
            &[5],
            b"https",
            &[0],
            &[0, 18],
            b"robot.example.test",
            &443_u16.to_be_bytes(),
            &[0, 4],
            b"/api",
            &[1, 0, 3],
            b"aud",
            &[0],
            &[1, 0, 6],
            b"tenant",
            &[0, 5],
            b"key-1",
            &[0, 11],
            b"body-sha256",
            &[0, 11],
            b"hmac-sha256",
            &[4],
            b"POST",
            &[0, 16],
            b"/objects?limit=2",
            &[2, 6],
            b"accept",
            &[0, 16],
            b"application/json",
            &[12],
            b"content-type",
            &[0, 16],
            b"application/json",
            &[3, 2],
            b"{}",
            &[0, 5],
            b"nonce",
            &42_u64.to_be_bytes(),
        ]
        .concat();
        assert_eq!(canonical.as_bytes(), expected);
        assert_eq!(canonical.request().body(), b"{}");
    }
    assert_eq!(digest, [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES]);
    assert_eq!(output, [0_u8; 512]);
}

#[test]
fn body_hashing_is_coupled_and_scratch_always_clears() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"exact-body") else {
        unreachable!("valid transport request must construct");
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        unreachable!("valid signing headers must construct");
    };
    let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
        unreachable!("valid signing context must construct");
    };
    let nonce = SigningNonce::new(b"nonce").unwrap_or_else(|_| unreachable!());
    let freshness = SigningFreshness::new(nonce, UnixTime::from_seconds(42));
    for hasher in [
        &MismatchedHasher as &dyn RequestBodyHasher<Error = ()>,
        &FailingHasher as &dyn RequestBodyHasher<Error = ()>,
        &LengthHasher(0),
        &LengthHasher(MAX_SIGNING_BODY_DIGEST_BYTES + 1),
    ] {
        let mut digest = [0xa5_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
        let mut output = [0xa5_u8; 512];
        let result = CanonicalSigningInput::new_hashed(
            request,
            context,
            headers,
            freshness,
            hasher,
            &mut digest,
            &mut output,
        );
        assert!(result.is_err());
        drop(result);
        assert_eq!(digest, [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES]);
        assert_eq!(output, [0xa5_u8; 512]);
    }

    #[cfg(feature = "std")]
    {
        let mut digest = [0xa5_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
        let mut output = [0xa5_u8; 512];
        let unwind = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
            let _ = CanonicalSigningInput::new_hashed(
                request,
                context,
                headers,
                freshness,
                &PanickingHasher,
                &mut digest,
                &mut output,
            );
        }));
        assert!(unwind.is_err());
        assert_eq!(digest, [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES]);
        assert_eq!(output, [0xa5_u8; 512]);
    }
}

#[test]
fn every_undersized_input_is_unchanged_and_digest_is_cleared() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"{}") else {
        unreachable!("valid transport request must construct");
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        unreachable!("valid signing headers must construct");
    };
    let Some(context) = context("robot", "robot.example.test", "key-1", "hmac-sha256") else {
        unreachable!("valid signing context must construct");
    };
    for capacity in 0..256 {
        let mut digest = [0xa5_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
        let mut output = [0xa5_u8; 256];
        let succeeded = {
            let result = canonical(
                request,
                context,
                headers,
                &mut digest,
                output.get_mut(..capacity).unwrap_or_default(),
            );
            match result {
                Ok(value) => {
                    drop(value);
                    true
                }
                Err(SigningBuildError::Input(SigningInputError::OutputTooSmall)) => false,
                Err(error) => unreachable!("unexpected canonicalization error: {error:?}"),
            }
        };
        assert_eq!(digest, [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES]);
        if succeeded {
            assert!(
                output
                    .get(..capacity)
                    .is_some_and(|bytes| bytes.iter().all(|byte| *byte == 0))
            );
            break;
        }
        assert_eq!(output, [0xa5_u8; 256]);
    }
}

#[test]
fn sensitive_selected_headers_remain_redacted() {
    let Ok(secret) = RequestHeader::sensitive("x-signing-secret", "secret-value") else {
        unreachable!("valid sensitive header must construct");
    };
    assert_eq!(secret.sensitivity(), HeaderSensitivity::Sensitive);
    let entries = [secret];
    let Ok(headers) = SigningHeaders::new(&entries) else {
        unreachable!("valid signing headers must construct");
    };
    assert!(!debug_text(&headers).as_str().contains("secret-value"));
}
