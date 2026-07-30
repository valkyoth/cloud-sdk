use core::cell::Cell;
use std::format;

use crate::Method;
use crate::transport::{
    ContentType, HeaderSensitivity, RequestHeader, RequestHeaders, RequestTarget, TransportRequest,
};

use super::{
    CanonicalSigningInput, MAX_CANONICAL_SIGNING_INPUT_BYTES, MAX_SIGNING_BODY_DIGEST_BYTES,
    MAX_SIGNING_NONCE_BYTES, RequestBodyHasher, RequestSigner, SigningBodyDigest, SigningHeaders,
    SigningInputError, SigningNonce, SigningValueError, UnixTime,
};

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

#[test]
fn digest_and_nonce_are_nonempty_bounded_and_redacted() {
    assert_eq!(SigningBodyDigest::new(&[]), Err(SigningValueError::Empty));
    assert_eq!(SigningNonce::new(&[]), Err(SigningValueError::Empty));
    assert_eq!(
        SigningBodyDigest::new(&[0; MAX_SIGNING_BODY_DIGEST_BYTES + 1]),
        Err(SigningValueError::TooLong)
    );
    assert_eq!(
        SigningNonce::new(&[0; MAX_SIGNING_NONCE_BYTES + 1]),
        Err(SigningValueError::TooLong)
    );
    let digest = SigningBodyDigest::new(b"secret-digest");
    let nonce = SigningNonce::new(b"secret-nonce");
    assert!(digest.is_ok() && nonce.is_ok());
    if let (Ok(digest), Ok(nonce)) = (digest, nonce) {
        assert_eq!(format!("{digest:?}"), "SigningBodyDigest([redacted])");
        assert_eq!(format!("{nonce:?}"), "SigningNonce([redacted])");
    }
}

#[test]
fn selected_headers_must_be_sorted_unique_and_match_the_request() {
    let first = RequestHeader::new("accept", "application/json");
    let second = RequestHeader::content_type(ContentType::JSON);
    let (Ok(first), second) = (first, second) else {
        return;
    };
    let ordered = [first, second];
    assert!(SigningHeaders::new(&ordered).is_ok());
    let reversed = [second, first];
    assert_eq!(
        SigningHeaders::new(&reversed).map(|_| ()),
        Err(SigningInputError::HeaderOrder)
    );
    let duplicate = [first, first];
    assert_eq!(
        SigningHeaders::new(&duplicate).map(|_| ()),
        Err(SigningInputError::HeaderOrder)
    );

    let Some(request) = request("/objects", &ordered, b"{}") else {
        return;
    };
    let changed = [RequestHeader::new("accept", "text/plain").ok()];
    let Some(changed) = changed[0] else {
        return;
    };
    let changed = [changed];
    let Some(digest) = SigningBodyDigest::new(b"digest").ok() else {
        return;
    };
    let Some(nonce) = SigningNonce::new(b"nonce").ok() else {
        return;
    };
    let mut output = [0xa5_u8; 512];
    assert_eq!(
        CanonicalSigningInput::new(
            request,
            SigningHeaders::new(&changed).unwrap_or_else(|_| unreachable!()),
            digest,
            nonce,
            UnixTime::from_seconds(42),
            &mut output,
        )
        .map(|_| ()),
        Err(SigningInputError::HeaderMismatch)
    );
}

#[test]
fn canonical_vector_is_versioned_length_framed_and_case_normalized() {
    let accept = RequestHeader::new("Accept", "application/json");
    let content_type = RequestHeader::content_type(ContentType::JSON);
    let (Ok(accept), content_type) = (accept, content_type) else {
        return;
    };
    let entries = [accept, content_type];
    let selected_entries = [accept, content_type];
    let Some(request) = request("/objects?limit=2", &entries, b"{}") else {
        return;
    };
    let Ok(selected) = SigningHeaders::new(&selected_entries) else {
        return;
    };
    let Ok(digest) = SigningBodyDigest::new(&[0xde, 0xad, 0xbe, 0xef]) else {
        return;
    };
    let Ok(nonce) = SigningNonce::new(b"nonce-1") else {
        return;
    };
    let mut output = [0xa5_u8; 512];
    {
        let canonical = CanonicalSigningInput::new(
            request,
            selected,
            digest,
            nonce,
            UnixTime::from_seconds(1_700_000_000),
            &mut output,
        );
        assert!(canonical.is_ok());
        if let Ok(canonical) = canonical {
            let expected = [
                b"cloud-sdk-signing-v1\0".as_slice(),
                &[4],
                b"POST",
                &[0, 16],
                b"/objects?limit=2",
                &[2],
                &[6],
                b"accept",
                &[0, 16],
                b"application/json",
                &[12],
                b"content-type",
                &[0, 16],
                b"application/json",
                &[4],
                &[0xde, 0xad, 0xbe, 0xef],
                &[0, 7],
                b"nonce-1",
                &1_700_000_000_u64.to_be_bytes(),
            ]
            .concat();
            assert_eq!(canonical.as_bytes(), expected);
            assert!(!format!("{canonical:?}").contains("nonce-1"));
        }
    }
    assert_eq!(output, [0_u8; 512]);
}

#[test]
fn nonce_and_time_changes_produce_distinct_replay_inputs() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"{}") else {
        return;
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        return;
    };
    let Ok(digest) = SigningBodyDigest::new(b"digest") else {
        return;
    };
    let mut captured = [[0_u8; 256]; 3];
    let contexts = [
        (b"nonce-a".as_slice(), 10_u64),
        (b"nonce-b".as_slice(), 10_u64),
        (b"nonce-a".as_slice(), 11_u64),
    ];
    for ((nonce, time), destination) in contexts.into_iter().zip(captured.iter_mut()) {
        let Ok(nonce) = SigningNonce::new(nonce) else {
            return;
        };
        let mut output = [0_u8; 256];
        let result = CanonicalSigningInput::new(
            request,
            headers,
            digest,
            nonce,
            UnixTime::from_seconds(time),
            &mut output,
        );
        assert!(result.is_ok());
        if let Ok(canonical) = result {
            let source = canonical.as_bytes();
            let Some(target) = destination.get_mut(..source.len()) else {
                return;
            };
            target.copy_from_slice(source);
        }
    }
    let [first, second, third] = captured;
    assert_ne!(first, second);
    assert_ne!(first, third);
}

#[test]
fn every_undersized_output_is_unchanged_and_success_clears_on_drop() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"{}") else {
        return;
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        return;
    };
    let Ok(digest) = SigningBodyDigest::new(b"digest") else {
        return;
    };
    let Ok(nonce) = SigningNonce::new(b"nonce") else {
        return;
    };
    for capacity in 0..96 {
        let mut output = [0xa5_u8; 96];
        let succeeded = {
            let result = CanonicalSigningInput::new(
                request,
                headers,
                digest,
                nonce,
                UnixTime::from_seconds(42),
                output.get_mut(..capacity).unwrap_or_default(),
            );
            match result {
                Ok(value) => {
                    drop(value);
                    true
                }
                Err(error) => {
                    assert_eq!(error, SigningInputError::OutputTooSmall);
                    false
                }
            }
        };
        if succeeded {
            assert!(
                output
                    .get(..capacity)
                    .is_some_and(|bytes| bytes.iter().all(|b| *b == 0))
            );
            break;
        }
        assert_eq!(output, [0xa5; 96]);
    }
}

struct FixedHasher;

impl RequestBodyHasher for FixedHasher {
    type Error = ();

    fn hash_body(&self, body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        let target = output.get_mut(..body.len()).ok_or(())?;
        target.copy_from_slice(body);
        Ok(body.len())
    }
}

struct CountingSigner<'a>(&'a Cell<usize>);

impl RequestSigner for CountingSigner<'_> {
    type Error = ();

    fn sign(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        self.0.set(self.0.get().saturating_add(1));
        let target = output.get_mut(..input.len()).ok_or(())?;
        target.copy_from_slice(input);
        Ok(input.len())
    }
}

#[test]
fn hashing_and_signing_are_entirely_caller_supplied() {
    let mut digest_output = [0_u8; 8];
    assert_eq!(FixedHasher.hash_body(b"body", &mut digest_output), Ok(4));
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"body") else {
        return;
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        return;
    };
    let Ok(digest) = SigningBodyDigest::new(&digest_output[..4]) else {
        return;
    };
    let Ok(nonce) = SigningNonce::new(b"nonce") else {
        return;
    };
    let mut input = [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES];
    let result = CanonicalSigningInput::new(
        request,
        headers,
        digest,
        nonce,
        UnixTime::from_seconds(42),
        &mut input,
    );
    assert!(result.is_ok());
    if let Ok(canonical) = result {
        let calls = Cell::new(0);
        let signer = CountingSigner(&calls);
        let mut signature = [0_u8; MAX_CANONICAL_SIGNING_INPUT_BYTES];
        assert!(canonical.sign_with(&signer, &mut signature).is_ok());
        assert_eq!(calls.get(), 1);
    }
}

#[test]
fn sensitive_selected_headers_remain_redacted() {
    let secret = RequestHeader::sensitive("x-signing-secret", "secret-value");
    let Ok(secret) = secret else {
        return;
    };
    assert_eq!(secret.sensitivity(), HeaderSensitivity::Sensitive);
    let entries = [secret];
    let Some(request) = request("/objects", &entries, b"") else {
        return;
    };
    let headers = SigningHeaders::new(&entries);
    assert!(headers.is_ok());
    if let Ok(headers) = headers {
        let debug = format!("{headers:?}");
        assert!(!debug.contains("secret-value"));
        let Ok(digest) = SigningBodyDigest::new(b"digest") else {
            return;
        };
        let Ok(nonce) = SigningNonce::new(b"nonce") else {
            return;
        };
        let mut output = [0_u8; 256];
        assert!(
            CanonicalSigningInput::new(
                request,
                headers,
                digest,
                nonce,
                UnixTime::from_seconds(42),
                &mut output,
            )
            .is_ok()
        );
    }
}
