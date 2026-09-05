use super::*;
#[cfg(feature = "std")]
use crate::std as test_std;
use alloc::{format, vec};
use cloud_sdk::transport::{BoundTransport, EndpointIdentity, EndpointIdentityError};
use cloud_sdk_sanitization::{SecretBuffer, SecretString};

pub(super) struct Transport(pub EndpointIdentity<'static>);
impl BoundTransport for Transport {
    fn endpoint_identity(&self) -> Result<EndpointIdentity<'_>, EndpointIdentityError> {
        Ok(self.0)
    }
}
pub(super) fn transport() -> Transport {
    Transport(
        CredentialOrigin::Production
            .endpoint()
            .identity()
            .unwrap_or_else(|_| unreachable!("official fixture")),
    )
}
pub(super) fn credential<K: CredentialKind>(text: &[u8]) -> Credential<K> {
    let mut source = text.to_vec();
    let result = Credential::from_mut_bytes(CredentialOrigin::Production, &mut source);
    assert!(source.iter().all(|byte| *byte == 0));
    result.unwrap_or_else(|_| unreachable!("protected fixture"))
}

fn rejects<K: CredentialKind>(value: &[u8], expected: CredentialError) {
    let mut source = value.to_vec();
    let result = Credential::<K>::from_mut_bytes(CredentialOrigin::Production, &mut source);
    assert!(matches!(result, Err(error) if error == expected));
    assert!(source.iter().all(|byte| *byte == 0));
}

#[test]
fn all_kinds_reject_empty_oversized_and_unsafe_input_and_clear_sources() {
    fn run<K: CredentialKind>() {
        rejects::<K>(b"", CredentialError::Empty);
        rejects::<K>(&vec![b'x'; K::MAX_BYTES + 1], CredentialError::TooLong);
        for invalid in [
            b" a".as_slice(),
            b"a ",
            b"a\tb",
            b"a\rb",
            b"a\nb",
            b"a\0b",
            b"a\x7fb",
            b"a\xffb",
            b"Bearer abc",
            b"Basic abc",
            b"a\"b",
            b"a\\b",
        ] {
            rejects::<K>(invalid, CredentialError::InvalidSyntax);
        }
    }
    run::<Api>();
    run::<TrustedPublishing>();
    run::<Oidc>();
    run::<EmailConfirmation>();
    run::<OwnerInvitation>();
}

#[test]
fn profiles_reject_path_and_compact_jwt_confusion() {
    for value in [
        b".".as_slice(),
        b"..",
        b"a/b",
        b"a?b",
        b"a#b",
        b"a%2Fb",
        b"a+b",
        b"a=b",
    ] {
        rejects::<EmailConfirmation>(value, CredentialError::InvalidSyntax);
        rejects::<OwnerInvitation>(value, CredentialError::InvalidSyntax);
    }
    for value in [
        b"abc".as_slice(),
        b"a.b",
        b"a.b.c.d",
        b".b.c",
        b"a..c",
        b"a.b.",
        b"a.b.c=",
        b"a.b.c+",
    ] {
        rejects::<Oidc>(value, CredentialError::InvalidSyntax);
    }
    for value in [b"=".as_slice(), b"a=b", b"a:b", b"a,b"] {
        rejects::<Api>(value, CredentialError::InvalidSyntax);
        rejects::<TrustedPublishing>(value, CredentialError::InvalidSyntax);
    }
}

#[test]
fn every_byte_is_classified_and_sources_are_cleared() {
    for byte in u8::MIN..=u8::MAX {
        fn check<K: CredentialKind>(source: &mut [u8], allowed: bool) {
            let result = Credential::<K>::from_mut_bytes(CredentialOrigin::Production, source);
            assert_eq!(result.is_ok(), allowed);
            assert!(source.iter().all(|value| *value == 0));
        }
        let header = byte.is_ascii_alphanumeric() || b"-._~+/=".contains(&byte);
        let path = byte.is_ascii_alphanumeric() || b"-._~".contains(&byte);
        let jwt = byte.is_ascii_alphanumeric() || b"-_".contains(&byte);
        check::<Api>(&mut [b'x', byte], header);
        check::<TrustedPublishing>(&mut [b'x', byte], header);
        check::<EmailConfirmation>(&mut [b'x', byte], path);
        check::<OwnerInvitation>(&mut [b'x', byte], path);
        check::<Oidc>(&mut [b'a', b'.', byte, b'.', b'c'], jwt);
    }
}

#[test]
fn clear_rotation_and_protected_ownership_preserve_the_boundary() {
    let mut token = credential::<TrustedPublishing>(b"first");
    let context = CredentialContext::revoke(CredentialOrigin::Production);
    let mut next = *b"second";
    token
        .rotate_from_mut_bytes(&mut next)
        .unwrap_or_else(|_| unreachable!("rotation fixture"));
    assert_eq!(next, [0; 6]);
    let mut invalid = *b"bad value";
    assert_eq!(
        token.rotate_from_mut_bytes(&mut invalid),
        Err(CredentialError::InvalidSyntax)
    );
    assert_eq!(invalid, [0; 9]);
    let mut output = [0xa5; 64];
    token
        .with_material_for_adapter(&context, &transport(), &mut output, |_, material| {
            assert_eq!(
                material
                    .authorization()
                    .unwrap_or_else(|| unreachable!("header fixture"))
                    .as_str(),
                "Bearer second"
            );
        })
        .unwrap_or_else(|_| unreachable!("valid material"));
    assert_eq!(output, [0; 64]);
    assert_eq!(token.origin(), CredentialOrigin::Production);
    token.clear();
    assert!(token.secret.is_empty());
    assert_eq!(
        token.with_material_for_adapter(&context, &transport(), &mut output, |_, _| unreachable!(
            "cleared credential applied"
        )),
        Err(CredentialError::Empty)
    );

    let mut protected = SecretString::try_with_capacity(64)
        .unwrap_or_else(|_| unreachable!("protected allocation fixture"));
    protected.push_str("owned-fixture");
    let pointer = protected
        .try_with_secret(str::as_ptr)
        .unwrap_or_else(|_| unreachable!("protected access fixture"));
    let token = ApiToken::from_secret_string(CredentialOrigin::Production, protected)
        .unwrap_or_else(|_| unreachable!("owned fixture"));
    assert_eq!(
        token
            .secret
            .try_with_secret(str::as_ptr)
            .unwrap_or_else(|_| unreachable!("protected access fixture")),
        pointer
    );
    let mut guarded_source = *b"guarded";
    drop(
        ApiToken::from_secret_buffer(
            CredentialOrigin::Production,
            SecretBuffer::new(&mut guarded_source),
        )
        .unwrap_or_else(|_| unreachable!("guard fixture")),
    );
    assert_eq!(guarded_source, [0; 7]);
}

#[test]
fn every_material_kind_formats_exactly_and_clears_the_entire_output() {
    fn run<K: CredentialKind>(secret: &[u8], context: CredentialContext<'_, K>, expected: &str) {
        let token = credential::<K>(secret);
        let mut output = vec![0xa5; expected.len() + 3];
        token
            .with_material_for_adapter(&context, &transport(), &mut output, |_, material| {
                let wire = match K::KIND {
                    0 | 1 => {
                        assert!(material.json_body().is_none());
                        material
                            .authorization()
                            .unwrap_or_else(|| unreachable!("header fixture"))
                            .as_str()
                    }
                    2 => {
                        assert!(material.authorization().is_none());
                        let body = material
                            .json_body()
                            .unwrap_or_else(|| unreachable!("JSON body fixture"));
                        core::str::from_utf8(body).unwrap_or_else(|_| unreachable!("JSON fixture"))
                    }
                    3 | 4 => {
                        assert!(material.authorization().is_none());
                        assert!(material.json_body().is_none());
                        material.target().as_str()
                    }
                    _ => unreachable!("credential kind fixture"),
                };
                assert_eq!(wire, expected);
                assert_eq!(material.method(), context.method);
                assert_eq!(material.endpoint().host(), "crates.io");
                assert!(!format!("{material:?}").contains(expected));
            })
            .unwrap_or_else(|_| unreachable!("material fixture"));
        assert!(output.iter().all(|byte| *byte == 0));
        let mut small = vec![0xa5; expected.len() - 1];
        assert_eq!(
            token.with_material_for_adapter(
                &context,
                &transport(),
                &mut small,
                |_, _| unreachable!("small output applied")
            ),
            Err(CredentialError::OutputTooSmall)
        );
        assert!(small.iter().all(|byte| *byte == 0));
        let mut exact = vec![0xa5; expected.len()];
        token
            .with_material_for_adapter(&context, &transport(), &mut exact, |_, _| ())
            .unwrap_or_else(|_| unreachable!("exact output fixture"));
        assert!(exact.iter().all(|byte| *byte == 0));
    }
    let origin = CredentialOrigin::Production;
    let api = CredentialContext::api(
        origin,
        cloud_sdk::Method::Put,
        crate::endpoint::ApiRequestTarget::new("/api/v1/crates/new")
            .unwrap_or_else(|_| unreachable!("path fixture")),
    )
    .unwrap_or_else(|_| unreachable!("API fixture"));
    run::<Api>(b"raw-value", api, "raw-value");
    run::<TrustedPublishing>(
        b"temporary",
        CredentialContext::publish(origin),
        "Bearer temporary",
    );
    run::<Oidc>(
        b"e30.e30.c2ln",
        CredentialContext::exchange(origin),
        "{\"jwt\":\"e30.e30.c2ln\"}",
    );
    run::<EmailConfirmation>(
        b"confirmation",
        CredentialContext::confirm_email(origin),
        "/api/v1/confirm/confirmation",
    );
    run::<OwnerInvitation>(
        b"invitation",
        CredentialContext::accept_invitation(origin),
        "/api/v1/me/crate_owner_invitations/accept/invitation",
    );
}

#[test]
fn maximum_lengths_are_accepted_by_storage_and_material() {
    fn run<K: CredentialKind>(context: CredentialContext<'_, K>) {
        let mut source = vec![b'x'; K::MAX_BYTES];
        if K::KIND == 2 {
            *source
                .get_mut(1)
                .unwrap_or_else(|| unreachable!("JWT fixture")) = b'.';
            *source
                .get_mut(3)
                .unwrap_or_else(|| unreachable!("JWT fixture")) = b'.';
        }
        let token = credential::<K>(&source);
        let mut output = vec![0; K::MAX_BYTES + 128];
        token
            .with_material_for_adapter(&context, &transport(), &mut output, |_, _| ())
            .unwrap_or_else(|_| unreachable!("maximum fixture"));
        assert!(output.iter().all(|byte| *byte == 0));
    }
    let origin = CredentialOrigin::Production;
    run::<Api>(
        CredentialContext::api(
            origin,
            cloud_sdk::Method::Get,
            crate::endpoint::ApiRequestTarget::new("/api/v1/crates")
                .unwrap_or_else(|_| unreachable!("path fixture")),
        )
        .unwrap_or_else(|_| unreachable!("API fixture")),
    );
    run::<TrustedPublishing>(CredentialContext::revoke(origin));
    run::<Oidc>(CredentialContext::exchange(origin));
    run::<EmailConfirmation>(CredentialContext::confirm_email(origin));
    run::<OwnerInvitation>(CredentialContext::accept_invitation(origin));
}

#[test]
fn credentials_and_errors_are_payload_free_in_diagnostics() {
    let token = credential::<Api>(b"redaction-fixture");
    assert!(!format!("{token:?} {token}").contains("redaction-fixture"));
    for error in [
        CredentialError::Empty,
        CredentialError::TooLong,
        CredentialError::InvalidSyntax,
        CredentialError::Allocation,
        CredentialError::StorageUnavailable,
        CredentialError::OperationNotAllowed,
        CredentialError::DestinationMismatch,
        CredentialError::OutputTooSmall,
    ] {
        assert!(!format!("{error:?} {error}").is_empty());
        assert!(core::error::Error::source(&error).is_none());
    }
}

#[cfg(feature = "std")]
#[test]
fn callback_error_and_unwinding_clear_caller_storage() {
    let token = credential::<TrustedPublishing>(b"unwind-fixture");
    let context = CredentialContext::publish(CredentialOrigin::Production);
    let mut output = [0xa5; 128];
    assert_eq!(
        token.with_material_for_adapter(&context, &transport(), &mut output, |_, _| Err::<(), _>(
            "adapter failure"
        )),
        Ok(Err("adapter failure"))
    );
    assert_eq!(output, [0; 128]);
    let result = test_std::panic::catch_unwind(test_std::panic::AssertUnwindSafe(|| {
        let _ = token.with_material_for_adapter(&context, &transport(), &mut output, |_, _| {
            unreachable!("adapter panic")
        });
    }));
    assert!(result.is_err());
    assert_eq!(output, [0; 128]);
}
