use std::format;

use cloud_sdk::transport::CustomEndpointAcknowledgement;

use super::{
    BasicCredential, BasicCredentialError, BasicPassword, BasicPasswordError, BasicUsername,
    BasicUsernameError, MAX_BASIC_PASSWORD_BYTES, MAX_BASIC_USERNAME_BYTES,
};
use crate::shared::{BasicCredentialScope, HttpsEndpoint};

fn test_scope() -> Option<BasicCredentialScope> {
    let endpoint = HttpsEndpoint::new_custom(
        "https://robot-ws.your-server.de",
        CustomEndpointAcknowledgement::trusted_operator_configuration(),
    )
    .ok()?;
    Some(BasicCredentialScope::new(
        cloud_sdk::provider_id!("hetzner"),
        cloud_sdk::service_id!("robot"),
        endpoint,
    ))
}

#[test]
fn rfc_vector_is_exact_bounded_sensitive_and_redacted() {
    let username = BasicUsername::new("Aladdin");
    let password = BasicPassword::new("open sesame");
    let (Ok(username), Ok(password), Some(scope)) = (username, password, test_scope()) else {
        unreachable!("security fixture construction failed");
    };
    let credential = BasicCredential::new(username, password, scope);
    assert!(credential.is_ok());
    let Ok(credential) = credential else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        credential.owned_bytes(),
        b"Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ=="
    );
    assert!(!format!("{credential:?}").contains("Aladdin"));
    let header = credential.header_value();
    assert!(header.as_ref().is_ok_and(|value| value.is_sensitive()));
}

#[test]
fn username_rejects_colon_space_controls_non_ascii_and_bounds() {
    for value in ["", "user:name", "user name", "user\nname", "anv\u{e4}ndare"] {
        let result = BasicUsername::new(value);
        let expected = if value.is_empty() {
            BasicUsernameError::Empty
        } else {
            BasicUsernameError::InvalidByte
        };
        assert_eq!(result.map(|_| ()), Err(expected));
    }
    let accepted = [b'u'; MAX_BASIC_USERNAME_BYTES];
    assert!(BasicUsername::from_bytes(&accepted).is_ok());
    let rejected = [b'u'; MAX_BASIC_USERNAME_BYTES + 1];
    assert_eq!(
        BasicUsername::from_bytes(&rejected).map(|_| ()),
        Err(BasicUsernameError::TooLong)
    );
}

#[test]
fn password_allows_spaces_and_colons_but_rejects_controls_non_ascii_and_bounds() {
    assert!(BasicPassword::new("open sesame:again").is_ok());
    assert_eq!(
        BasicPassword::new("").map(|_| ()),
        Err(BasicPasswordError::Empty)
    );
    for value in ["line\nbreak", "l\u{f6}senord"] {
        assert_eq!(
            BasicPassword::new(value).map(|_| ()),
            Err(BasicPasswordError::InvalidByte)
        );
    }
    let accepted = [b'p'; MAX_BASIC_PASSWORD_BYTES];
    assert!(BasicPassword::from_bytes(&accepted).is_ok());
    let rejected = [b'p'; MAX_BASIC_PASSWORD_BYTES + 1];
    assert_eq!(
        BasicPassword::from_bytes(&rejected).map(|_| ()),
        Err(BasicPasswordError::TooLong)
    );
}

#[test]
fn mutable_sources_clear_on_success_and_rejection() {
    let mut username = *b"robot-user";
    let mut password = *b"secret-pass";
    let Some(scope) = test_scope() else {
        unreachable!("security fixture construction failed");
    };
    assert!(BasicCredential::from_mut_bytes(&mut username, &mut password, scope).is_ok());
    assert_eq!(username, [0; 10]);
    assert_eq!(password, [0; 11]);

    let mut invalid_username = *b"bad:user";
    let mut valid_password = *b"password";
    let Some(scope) = test_scope() else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(
        BasicCredential::from_mut_bytes(&mut invalid_username, &mut valid_password, scope,)
            .map(|_| ()),
        Err(BasicCredentialError::UsernameRejected(
            BasicUsernameError::InvalidByte
        ))
    );
    assert_eq!(invalid_username, [0; 8]);
    assert_eq!(valid_password, [0; 8]);
}

#[test]
fn exact_individual_bounds_fit_the_aggregate_authorization_limit() {
    let username = [b'u'; MAX_BASIC_USERNAME_BYTES];
    let password = [b'p'; MAX_BASIC_PASSWORD_BYTES];
    let (Ok(username), Ok(password), Some(scope)) = (
        BasicUsername::from_bytes(&username),
        BasicPassword::from_bytes(&password),
        test_scope(),
    ) else {
        unreachable!("security fixture construction failed");
    };
    let credential = BasicCredential::new(username, password, scope);
    assert!(credential.is_ok());
    let Ok(credential) = credential else {
        unreachable!("security fixture construction failed");
    };
    assert_eq!(credential.owned_bytes().len(), 3_082);
}

#[test]
fn separately_constructed_basic_credentials_receive_distinct_bindings() {
    let make = || {
        let (Ok(username), Ok(password), Some(scope)) = (
            BasicUsername::new("robot-user"),
            BasicPassword::new("secret-pass"),
            test_scope(),
        ) else {
            unreachable!("security fixture construction failed");
        };
        BasicCredential::new(username, password, scope)
            .unwrap_or_else(|_| unreachable!("credential construction failed"))
    };
    let first = make();
    let second = make();
    assert!(!first.binding().matches(second.binding()));
}
