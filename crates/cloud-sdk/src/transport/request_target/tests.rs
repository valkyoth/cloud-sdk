use super::{
    CanonicalQuery, FormQuery, MAX_REQUEST_TARGET_BYTES, RequestPath, RequestPathError,
    RequestQuery, RequestTarget, RequestTargetError, StructuredQueryError,
};

macro_rules! valid {
    ($value:expr) => {{
        let result = $value;
        assert!(result.is_ok());
        let Ok(value) = result else {
            return;
        };
        value
    }};
}

#[test]
fn validates_canonical_paths() {
    for accepted in [
        "/",
        "/servers/42",
        "/servers/",
        "/zones/example.com/rrsets/%40/AAAA",
        "/zones/example.com/rrsets/%2A.apps/TXT",
    ] {
        assert_eq!(
            RequestPath::new(accepted).map(RequestPath::as_str),
            Ok(accepted)
        );
    }

    for (value, error) in [
        ("", RequestPathError::Empty),
        ("servers", RequestPathError::NotOriginForm),
        ("//authority", RequestPathError::NotOriginForm),
        ("/a//b", RequestPathError::DoubledSlash),
        ("/a/./b", RequestPathError::DotSegment),
        ("/a/../b", RequestPathError::DotSegment),
        ("/a?b", RequestPathError::InvalidByte),
        ("/a#b", RequestPathError::InvalidByte),
        ("/a\\b", RequestPathError::InvalidByte),
        ("/a b", RequestPathError::InvalidByte),
        ("/a/%", RequestPathError::InvalidPercentTriplet),
        ("/a/%2", RequestPathError::InvalidPercentTriplet),
        ("/a/%GG", RequestPathError::InvalidPercentTriplet),
        ("/a/%2f", RequestPathError::LowercasePercentHex),
        ("/a/%2F", RequestPathError::EncodedSeparator),
        ("/a/%5C", RequestPathError::EncodedSeparator),
        ("/a/%3F", RequestPathError::EncodedSeparator),
        ("/a/%23", RequestPathError::EncodedControl),
        ("/a/%00", RequestPathError::EncodedControl),
        ("/a/%41", RequestPathError::EncodedUnreserved),
        ("/a/[b]", RequestPathError::InvalidByte),
        ("/münchen", RequestPathError::InvalidByte),
    ] {
        assert_eq!(RequestPath::new(value), Err(error), "{value}");
    }
}

#[test]
fn validates_canonical_and_form_queries_separately() {
    for accepted in [
        "",
        "page=2",
        "name=test%20server",
        "label=env%3Dprod%26tier%3Dapi",
        "flag&empty=&repeat=1&repeat=2",
        "utf8=%C3%A4",
    ] {
        assert_eq!(
            CanonicalQuery::new(accepted).map(CanonicalQuery::as_str),
            Ok(accepted)
        );
    }

    assert_eq!(
        CanonicalQuery::new("name=test+server"),
        Err(StructuredQueryError::PlusForbidden)
    );
    assert_eq!(
        FormQuery::new("name=test+server").map(FormQuery::as_str),
        Ok("name=test+server")
    );

    for (value, error) in [
        ("=value", StructuredQueryError::EmptyKey),
        ("a=1&&b=2", StructuredQueryError::EmptyPair),
        ("a=1&", StructuredQueryError::EmptyPair),
        ("a=b=c", StructuredQueryError::MultipleEquals),
        ("a=%", StructuredQueryError::InvalidPercentTriplet),
        ("a=%2f", StructuredQueryError::LowercasePercentHex),
        ("a=%41", StructuredQueryError::EncodedUnreserved),
        ("a=%23", StructuredQueryError::EncodedControl),
        ("a=%5C", StructuredQueryError::EncodedControl),
        ("a b=c", StructuredQueryError::InvalidByte),
        ("a?b=c", StructuredQueryError::InvalidByte),
        ("münchen=1", StructuredQueryError::InvalidByte),
    ] {
        assert_eq!(CanonicalQuery::new(value), Err(error), "{value}");
    }
}

#[test]
fn preserves_query_order_duplicates_and_value_presence() {
    let query = valid!(CanonicalQuery::new("flag&empty=&a=1&a=2"));
    let mut pairs = query.pairs();
    let values = (pairs.next(), pairs.next(), pairs.next(), pairs.next());
    assert!(values.0.is_some() && values.1.is_some() && values.2.is_some() && values.3.is_some());
    let (Some(flag), Some(empty), Some(first), Some(second)) = values else {
        return;
    };
    assert_eq!((flag.key(), flag.value()), ("flag", None));
    assert_eq!((empty.key(), empty.value()), ("empty", Some("")));
    assert_eq!((first.key(), first.value()), ("a", Some("1")));
    assert_eq!((second.key(), second.value()), ("a", Some("2")));
    assert_eq!(pairs.next(), None);
}

#[test]
fn distinguishes_absent_and_present_empty_queries() {
    let absent = valid!(RequestTarget::new("/servers"));
    let present = valid!(RequestTarget::new("/servers?"));
    let empty = valid!(CanonicalQuery::new(""));

    assert_eq!(absent.query(), RequestQuery::Absent);
    assert_eq!(absent.query_bytes(), None);
    assert_eq!(present.query(), RequestQuery::Canonical(empty));
    assert_eq!(present.query_bytes(), Some(b"".as_slice()));
}

#[test]
fn assembles_atomically_and_preserves_exact_query_bytes() {
    let path = valid!(RequestPath::new("/servers"));
    let query = valid!(CanonicalQuery::new("flag&name=test%20server&name=api"));
    let mut output = [0xA5; 64];
    let target = valid!(RequestTarget::assemble(
        path,
        RequestQuery::Canonical(query),
        &mut output
    ));

    assert_eq!(target.as_str(), "/servers?flag&name=test%20server&name=api");
    assert_eq!(target.path(), path);
    assert_eq!(target.query_bytes(), Some(query.as_str().as_bytes()));
    assert_eq!(target.query(), RequestQuery::Canonical(query));
}

#[test]
fn assembly_failure_does_not_modify_output() {
    let path = valid!(RequestPath::new("/servers"));
    let query = valid!(CanonicalQuery::new("page=2"));
    let mut output = [0xA5; 8];
    let before = output;

    assert_eq!(
        RequestTarget::assemble(path, RequestQuery::Canonical(query), &mut output),
        Err(RequestTargetError::OutputTooSmall)
    );
    assert_eq!(output, before);
}

#[test]
fn form_targets_retain_their_explicit_dialect() {
    let path = valid!(RequestPath::new("/search"));
    let query = valid!(FormQuery::new("name=test+server"));
    let mut output = [0_u8; 64];
    let target = valid!(RequestTarget::assemble(
        path,
        RequestQuery::Form(query),
        &mut output
    ));

    assert_eq!(target.as_str(), "/search?name=test+server");
    assert_eq!(target.query(), RequestQuery::Form(query));
}

#[test]
fn complete_target_bound_includes_query_delimiter() {
    let mut path = [b'a'; MAX_REQUEST_TARGET_BYTES];
    path[0] = b'/';
    let path = valid!(core::str::from_utf8(&path));
    assert!(RequestTarget::new(path).is_ok());

    let mut oversized = [b'a'; MAX_REQUEST_TARGET_BYTES + 1];
    oversized[0] = b'/';
    let oversized = valid!(core::str::from_utf8(&oversized));
    assert_eq!(
        RequestTarget::new(oversized),
        Err(RequestTargetError::TooLong)
    );

    let path = valid!(RequestPath::new(path));
    let empty = valid!(CanonicalQuery::new(""));
    let mut output = [0xA5; MAX_REQUEST_TARGET_BYTES + 1];
    assert_eq!(
        RequestTarget::assemble(path, RequestQuery::Canonical(empty), &mut output),
        Err(RequestTargetError::TooLong)
    );
    assert!(output.iter().all(|byte| *byte == 0xA5));
}
