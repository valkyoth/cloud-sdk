use super::*;
use bytes::Bytes;
use cloud_sdk::transport::{CustomEndpointAcknowledgement, FormQuery, RequestPath, RequestQuery};
use std::{
    format,
    string::ToString,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

fn endpoint(value: &str) -> HttpsEndpoint {
    HttpsEndpoint::new_custom(
        value,
        CustomEndpointAcknowledgement::trusted_operator_configuration(),
    )
    .unwrap_or_else(|_| unreachable!("valid endpoint fixture"))
}

#[test]
fn raw_uri_matches_previous_composition_exactly() {
    for base in [
        "https://example.test",
        "https://example.test:8443/api",
        "https://[::1]/api",
    ] {
        let endpoint = endpoint(base);
        for value in [
            "/",
            "/confirm/abc-012",
            "/a?",
            "/a?x=%20&y=%2F",
            "/a%3Ab?q=%25",
            "/a'b",
        ] {
            let target =
                RequestTarget::new(value).unwrap_or_else(|_| unreachable!("valid target fixture"));
            let old = endpoint
                .compose(target)
                .unwrap_or_else(|_| unreachable!("previous composition"));
            let uri = compose(&endpoint, target)
                .unwrap_or_else(|_| unreachable!("protected composition"));
            assert_eq!(uri.to_string(), old.as_str());
        }
        let path = RequestPath::new("/search").unwrap_or_else(|_| unreachable!("path"));
        let query = FormQuery::new("q=a+b%2Bc").unwrap_or_else(|_| unreachable!("query"));
        let mut output = [0; 128];
        let target = RequestTarget::assemble(path, RequestQuery::Form(query), &mut output)
            .unwrap_or_else(|_| unreachable!("form target"));
        assert_eq!(
            compose(&endpoint, target)
                .unwrap_or_else(|_| unreachable!("URI"))
                .to_string(),
            endpoint
                .compose(target)
                .unwrap_or_else(|_| unreachable!("URL"))
                .as_str()
        );
    }
}

#[test]
fn raw_uri_preserves_every_admitted_ascii_path_and_canonical_query() {
    let endpoint = endpoint("https://example.test/api");
    for byte in 0u8..=127 {
        for value in [
            format!("/a{}b", char::from(byte)),
            format!("/a?q=x{}y", char::from(byte)),
        ] {
            // Invalid inputs are not part of the typed transport domain.
            match RequestTarget::new(&value) {
                Ok(target) => {
                    let old = endpoint.compose(target);
                    let new = compose(&endpoint, target);
                    assert_eq!(old.is_ok(), new.is_ok());
                    match (old, new) {
                        (Ok(old), Ok(new)) => assert_eq!(old.as_str(), new.to_string()),
                        (Err(_), Err(_)) => {}
                        _ => unreachable!("composition mismatch"),
                    }
                }
                Err(_) => assert!(byte != b'a'),
            }
        }
    }
}

fn observed(value: &[u8], drops: &Arc<AtomicUsize>) -> Bytes {
    SanitizedBody::copy_from(value)
        .unwrap_or_else(|_| unreachable!("allocation"))
        .with_drop_probe(Arc::clone(drops))
        .into_bytes()
}

#[test]
fn pinned_uri_clones_and_origin_form_retain_cleanup_owner_without_copying() {
    let drops = Arc::new(AtomicUsize::new(0));
    let source = observed(b"/confirm/opaque-token?x=%20", &drops);
    let base = "https://example.test"
        .parse()
        .unwrap_or_else(|_| unreachable!("base"));
    let uri = with_path(base, source.clone()).unwrap_or_else(|_| unreachable!("valid URI"));
    let authority = uri
        .authority()
        .unwrap_or_else(|| unreachable!("authority"))
        .clone();
    let clone = uri.clone();
    let path = clone
        .path_and_query()
        .unwrap_or_else(|| unreachable!("path"));
    assert_eq!(path.as_str().as_ptr(), source.as_ptr());
    let mut parts = http::uri::Parts::default();
    parts.path_and_query = Some(path.clone());
    let origin = http::Uri::from_parts(parts).unwrap_or_else(|_| unreachable!("origin URI"));
    drop(source);
    drop(uri);
    drop(clone);
    assert_eq!(drops.load(Ordering::SeqCst), 0);
    assert_eq!(origin.path(), "/confirm/opaque-token");
    drop(origin);
    assert_eq!(drops.load(Ordering::SeqCst), 1);
    assert_eq!(authority.as_str(), "example.test");
}

#[test]
fn pinned_uri_rejection_releases_cleanup_owner() {
    for value in [b"/bad\n".as_slice(), b"/bad#fragment"] {
        let drops = Arc::new(AtomicUsize::new(0));
        let base = "https://example.test"
            .parse()
            .unwrap_or_else(|_| unreachable!("base"));
        assert!(with_path(base, observed(value, &drops)).is_err());
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }
}
