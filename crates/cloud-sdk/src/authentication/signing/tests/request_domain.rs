use crate::Method;
use crate::authentication::{
    CanonicalSigningInput, SigningFreshness, SigningHeaders, SigningNonce, UnixTime,
};
use crate::transport::{RequestHeader, RequestHeaders, RequestTarget, TransportRequest};

use super::{BodyHasher, MAX_SIGNING_BODY_DIGEST_BYTES, context};

#[derive(Clone, Copy)]
struct RequestParts {
    method: Method,
    target: &'static str,
    header_value: &'static str,
    body: &'static [u8],
    nonce: &'static [u8],
    time: u64,
}

const BASE: RequestParts = RequestParts {
    method: Method::Post,
    target: "/objects",
    header_value: "one",
    body: b"body-one",
    nonce: b"nonce-one",
    time: 42,
};

fn capture(parts: RequestParts) -> Option<([u8; 512], usize)> {
    let header = RequestHeader::new("x-provider-field", parts.header_value).ok()?;
    let entries = [header];
    let headers = RequestHeaders::new(&entries).ok()?;
    let target = RequestTarget::new(parts.target).ok()?;
    let request = TransportRequest::new(parts.method, target)
        .with_headers(headers)
        .with_body(parts.body);
    let selected = SigningHeaders::new(&entries).ok()?;
    let context = context("robot", "robot.example.test", "key-1", "hmac-sha256")?;
    let nonce = SigningNonce::new(parts.nonce).ok()?;
    let freshness = SigningFreshness::new(nonce, UnixTime::from_seconds(parts.time));
    let mut digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut storage = [0_u8; 512];
    let canonical = CanonicalSigningInput::new_hashed(
        request,
        context,
        selected,
        freshness,
        &BodyHasher,
        &mut digest,
        &mut storage,
    )
    .ok()?;
    let source = canonical.as_bytes();
    let len = source.len();
    let mut captured = [0_u8; 512];
    captured.get_mut(..len)?.copy_from_slice(source);
    Some((captured, len))
}

#[test]
fn every_request_and_freshness_field_changes_the_canonical_input() {
    let Some(base) = capture(BASE) else {
        unreachable!("baseline request must construct");
    };
    let changed = [
        RequestParts {
            method: Method::Get,
            ..BASE
        },
        RequestParts {
            target: "/other",
            ..BASE
        },
        RequestParts {
            header_value: "two",
            ..BASE
        },
        RequestParts {
            body: b"body-two",
            ..BASE
        },
        RequestParts {
            nonce: b"nonce-two",
            ..BASE
        },
        RequestParts { time: 43, ..BASE },
    ];
    for candidate in changed {
        let Some(candidate) = capture(candidate) else {
            unreachable!("changed request must construct");
        };
        assert_ne!(base, candidate);
    }
}
