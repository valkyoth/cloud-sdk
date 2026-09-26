use super::{config, guard};
use cloud_sdk::{Method, transport::*};
use cloud_sdk_cratesio::{
    catalog::CatalogRequest,
    query::{Parameter, PerPage},
};

#[test]
fn opt_in_is_exact_and_ci_always_fails_closed() {
    assert_eq!(
        config::mode("anonymous", false, None),
        Ok(config::Mode::Anonymous)
    );
    assert_eq!(
        config::mode("token", false, Some("isolated-account-read-requests")),
        Ok(config::Mode::Token)
    );
    for mode in ["", "1", "true", "anonymous", "token", "publish"] {
        for acknowledgement in [None, Some("isolated-account-read-requests"), Some("write")] {
            assert!(config::mode(mode, true, acknowledgement).is_err());
        }
    }
    assert!(config::mode("token", false, None).is_err());
    assert!(config::mode("anonymous", false, Some("isolated-account-read-requests")).is_err());
    assert!(config::mode("token", false, Some("read-only-token")).is_err());
    assert!(config::identity("").is_err());
    assert!(config::identity("test/1 (tests@example.org)").is_ok());
}

#[test]
fn only_literal_read_routes_and_empty_bodies_are_admitted() -> Result<(), Box<dyn std::error::Error>>
{
    for path in [
        guard::METADATA,
        guard::KEYWORDS,
        guard::FOLLOWING,
        "/api/v1/crates/new",
        "/api/v1/tokens/1",
        "/api/v1/crates/serde/owners",
        "/api/v1/crates/serde/1.0.0/yank",
    ] {
        for method in [
            Method::Get,
            Method::Post,
            Method::Put,
            Method::Patch,
            Method::Delete,
            Method::Head,
        ] {
            for authorized in [false, true] {
                let request = TransportRequest::new(method, RequestTarget::new(path)?);
                let expected = method == Method::Get
                    && if authorized {
                        path == guard::FOLLOWING
                    } else {
                        matches!(path, guard::METADATA | guard::KEYWORDS)
                    };
                assert_eq!(guard::check(request, authorized).is_ok(), expected);
                assert!(guard::check(request.with_body(b"{}"), authorized).is_err());
            }
        }
    }
    let parameters = [Parameter::Following, Parameter::PerPage(PerPage::new(1)?)];
    let mut path = [0; 128];
    assert_eq!(
        CatalogRequest::list(&parameters)?
            .write_target(&mut path)?
            .as_str(),
        guard::FOLLOWING
    );
    Ok(())
}

#[test]
fn credential_input_is_bounded_redacted_and_rejects_extra_lines() {
    for input in [b"abc".as_slice(), b"abc\n", b"abc\r\n"] {
        let result = config::read_token(&mut &*input);
        assert!(result.is_ok());
        assert!(!format!("{result:?}").contains("abc"));
    }
    for input in [b"".as_slice(), b"abc\n\n", b"abc\r", b"abc\nxyz", b" abc"] {
        assert!(config::read_token(&mut &*input).is_err());
    }
    let maximum = vec![b'a'; 1024];
    assert!(config::read_token(&mut maximum.as_slice()).is_ok());
    let oversized = vec![b'a'; 1027];
    let mut remaining = oversized.as_slice();
    assert!(config::read_token(&mut remaining).is_err());
    assert!(remaining.is_empty());
}

#[test]
fn credential_reader_errors_and_maximum_line_endings_are_handled() {
    struct Failed;
    impl std::io::Read for Failed {
        fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
            output.fill(b's');
            Err(std::io::Error::other("sensitive upstream detail"))
        }
    }
    let error = config::read_token(&mut Failed);
    assert!(error.is_err());
    assert!(!format!("{error:?}").contains("sensitive upstream detail"));
    for length in [1024, 1025] {
        for suffix in [b"\n".as_slice(), b"\r\n"] {
            let mut input = vec![b'a'; length];
            input.extend_from_slice(suffix);
            assert_eq!(
                config::read_token(&mut input.as_slice()).is_ok(),
                length == 1024
            );
        }
    }
}

#[test]
fn inline_credentials_and_cookie_headers_are_rejected() -> Result<(), Box<dyn std::error::Error>> {
    assert!(RequestHeader::sensitive("authorization", "fixture").is_err());
    let headers = [RequestHeader::sensitive("cookie", "fixture")?];
    let request = TransportRequest::new(Method::Get, RequestTarget::new(guard::METADATA)?)
        .with_headers(RequestHeaders::new(&headers)?);
    assert!(guard::check(request, false).is_err());
    Ok(())
}
