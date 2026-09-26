use cloud_sdk_cratesio::{
    catalog::CatalogRequest,
    credentials::{ApiToken, CredentialOrigin},
    endpoint::{ApiRequestTarget, StaticDownloadTarget},
    identifiers::CrateName,
    query::{Parameter, SearchQuery},
};

pub fn exercise(data: &[u8]) -> (bool, bool) {
    let mut secret = data.to_vec();
    let token = ApiToken::from_mut_bytes(CredentialOrigin::Production, &mut secret);
    assert!(secret.iter().all(|b| *b == 0));
    if let Ok(token) = token {
        assert_eq!(format!("{token:?}"), "Credential([redacted])");
        assert_eq!(token.origin(), CredentialOrigin::Production);
    }
    let Ok(text) = core::str::from_utf8(data) else {
        return (false, false);
    };
    if let Ok(target) = ApiRequestTarget::new(text) {
        assert!(target.as_str().starts_with("/api/v1/"));
        assert_eq!(target.as_str(), text);
    }
    if let Ok(target) = StaticDownloadTarget::new(text) {
        assert!(target.as_str().starts_with("/crates/"));
        assert!(target.as_str().ends_with(".crate"));
        assert!(!target.as_str().contains('?'));
    }
    let name_valid = if let Ok(name) = CrateName::new(text) {
        let request = CatalogRequest::crate_metadata(name, &[]).expect("valid name");
        let expected = format!("/api/v1/crates/{text}");
        check_target(request, &expected);
        true
    } else {
        false
    };
    let query_valid = if let Ok(search) = SearchQuery::new(text) {
        let parameters = [Parameter::Search(search)];
        let request = CatalogRequest::cargo_search(&parameters).expect("valid search");
        let mut encoded = String::from("/api/v1/crates?q=");
        for byte in text.bytes() {
            if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                encoded.push(char::from(byte));
            } else {
                use core::fmt::Write;
                write!(&mut encoded, "%{byte:02X}").expect("string writer");
            }
        }
        check_target(request, &encoded);
        true
    } else {
        false
    };
    (name_valid, query_valid)
}

fn check_target(request: CatalogRequest<'_>, expected: &str) {
    let mut output = vec![0xa5; expected.len() + 1];
    assert_eq!(
        request
            .write_target(&mut output)
            .expect("large buffer")
            .as_str(),
        expected
    );
    assert_eq!(output[expected.len()], 0xa5);
    for capacity in [0, expected.len().saturating_sub(1)] {
        let mut small = vec![0xa5; capacity];
        assert!(request.write_target(&mut small).is_err());
        assert!(small.iter().all(|b| *b == 0xa5));
    }
}
