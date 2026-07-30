use crate::authentication::{
    RequestBodyHasher, ScopeValue, SigningAlgorithm, SigningContext, SigningDigestAlgorithm,
    SigningHeaders, SigningKeyId,
};
use crate::transport::{ContentType, EndpointIdentity, EndpointScheme, RequestHeader};
use crate::{ProviderId, ServiceId};

use super::{MAX_SIGNING_BODY_DIGEST_BYTES, canonical_with_hasher, request};

struct NamedHasher(&'static str);

impl RequestBodyHasher for NamedHasher {
    type Error = ();

    fn digest_algorithm(&self) -> SigningDigestAlgorithm<'_> {
        SigningDigestAlgorithm::new(self.0).unwrap_or_else(|_| unreachable!())
    }

    fn hash_body(&self, _body: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        let byte = output.first_mut().ok_or(())?;
        *byte = 0x42;
        Ok(1)
    }
}

#[derive(Clone, Copy)]
struct ContextParts {
    provider: &'static str,
    service: &'static str,
    scheme: EndpointScheme,
    host: &'static str,
    port: u16,
    base_path: &'static str,
    audience: Option<&'static str>,
    account: Option<&'static str>,
    tenant: Option<&'static str>,
    key_id: &'static str,
    digest_algorithm: &'static str,
    signature_algorithm: &'static str,
}

const BASE: ContextParts = ContextParts {
    provider: "hetzner",
    service: "robot",
    scheme: EndpointScheme::Https,
    host: "one.example.test",
    port: 443,
    base_path: "/api",
    audience: None,
    account: None,
    tenant: None,
    key_id: "key-1",
    digest_algorithm: "body-sha256",
    signature_algorithm: "hmac-sha256",
};

fn signing_context(parts: ContextParts) -> Option<SigningContext<'static>> {
    let provider = ProviderId::new(parts.provider).ok()?;
    let service = ServiceId::new(parts.service).ok()?;
    let endpoint =
        EndpointIdentity::new(parts.scheme, parts.host, parts.port, parts.base_path).ok()?;
    let key_id = SigningKeyId::new(parts.key_id).ok()?;
    let digest_algorithm = SigningDigestAlgorithm::new(parts.digest_algorithm).ok()?;
    let signature_algorithm = SigningAlgorithm::new(parts.signature_algorithm).ok()?;
    let mut context = SigningContext::new(
        provider,
        service,
        endpoint,
        key_id,
        digest_algorithm,
        signature_algorithm,
    );
    if let Some(value) = parts.audience {
        context = context.with_audience(ScopeValue::new(value).ok()?);
    }
    if let Some(value) = parts.account {
        context = context.with_account(ScopeValue::new(value).ok()?);
    }
    if let Some(value) = parts.tenant {
        context = context.with_tenant(ScopeValue::new(value).ok()?);
    }
    Some(context)
}

#[test]
fn every_security_domain_field_changes_the_canonical_input() {
    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"{}") else {
        unreachable!("valid transport request must construct");
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        unreachable!("valid signing headers must construct");
    };
    let contexts = [
        BASE,
        ContextParts {
            provider: "scaleway",
            ..BASE
        },
        ContextParts {
            service: "cloud",
            ..BASE
        },
        ContextParts {
            scheme: EndpointScheme::Http,
            ..BASE
        },
        ContextParts {
            host: "two.example.test",
            ..BASE
        },
        ContextParts { port: 444, ..BASE },
        ContextParts {
            base_path: "/other",
            ..BASE
        },
        ContextParts {
            audience: Some("audience"),
            ..BASE
        },
        ContextParts {
            account: Some("account"),
            ..BASE
        },
        ContextParts {
            tenant: Some("tenant"),
            ..BASE
        },
        ContextParts {
            key_id: "key-2",
            ..BASE
        },
        ContextParts {
            digest_algorithm: "body-sha512",
            ..BASE
        },
        ContextParts {
            signature_algorithm: "ed25519",
            ..BASE
        },
    ];
    let mut captured = [[0_u8; 512]; 13];
    for (parts, destination) in contexts.into_iter().zip(captured.iter_mut()) {
        let Some(context) = signing_context(parts) else {
            unreachable!("valid signing context must construct");
        };
        let mut digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
        let mut output = [0_u8; 512];
        let hasher = NamedHasher(parts.digest_algorithm);
        let Ok(canonical) =
            canonical_with_hasher(request, context, headers, &hasher, &mut digest, &mut output)
        else {
            unreachable!("canonical signing input must construct");
        };
        let source = canonical.as_bytes();
        let Some(target) = destination.get_mut(..source.len()) else {
            unreachable!("capture buffer must fit canonical signing input");
        };
        target.copy_from_slice(source);
    }
    for left in 0..captured.len() {
        for right in left.saturating_add(1)..captured.len() {
            assert_ne!(captured.get(left), captured.get(right));
        }
    }
}

#[test]
fn equivalent_ipv6_endpoint_spellings_have_one_canonical_input() {
    let compact = ContextParts {
        host: "[2001:db8::1]",
        ..BASE
    };
    let expanded = ContextParts {
        host: "[2001:0db8:0:0:0:0:0:1]",
        ..BASE
    };
    let (Some(compact_context), Some(expanded_context)) =
        (signing_context(compact), signing_context(expanded))
    else {
        unreachable!("equivalent IPv6 contexts must construct");
    };
    assert_eq!(compact_context.endpoint(), expanded_context.endpoint());

    let entries = [RequestHeader::content_type(ContentType::JSON)];
    let Some(request) = request("/objects", &entries, b"{}") else {
        unreachable!("valid transport request must construct");
    };
    let Ok(headers) = SigningHeaders::new(&entries) else {
        unreachable!("valid signing headers must construct");
    };
    let mut compact_digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut expanded_digest = [0_u8; MAX_SIGNING_BODY_DIGEST_BYTES];
    let mut compact_output = [0_u8; 512];
    let mut expanded_output = [0_u8; 512];
    let Ok(compact_canonical) = canonical_with_hasher(
        request,
        compact_context,
        headers,
        &NamedHasher(BASE.digest_algorithm),
        &mut compact_digest,
        &mut compact_output,
    ) else {
        unreachable!("compact IPv6 canonical input must construct");
    };
    let Ok(expanded_canonical) = canonical_with_hasher(
        request,
        expanded_context,
        headers,
        &NamedHasher(BASE.digest_algorithm),
        &mut expanded_digest,
        &mut expanded_output,
    ) else {
        unreachable!("expanded IPv6 canonical input must construct");
    };
    assert_eq!(compact_canonical.as_bytes(), expanded_canonical.as_bytes());
    let canonical_host = [
        2, 0x20, 0x01, 0x0d, 0xb8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    ];
    assert!(
        compact_canonical
            .as_bytes()
            .windows(canonical_host.len())
            .any(|window| window == canonical_host)
    );
    assert!(
        !compact_canonical
            .as_bytes()
            .windows(compact.host.len())
            .any(|window| window == compact.host.as_bytes())
    );
}
