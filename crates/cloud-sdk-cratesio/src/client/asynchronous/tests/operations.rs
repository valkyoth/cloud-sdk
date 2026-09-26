use super::*;
use crate::{
    accounts::personal::*,
    credentials::{ApiToken, CredentialOrigin},
    identifiers::{CrateName, NumericId, Version},
    ownership::{OwnerChangeRequest, OwnerSelector},
    publishing::YankRequest,
    wire::TEST_GATE_LOCK,
};

pub(super) fn token(origin: CredentialOrigin) -> ApiToken {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let mut bytes = [b'a'.saturating_add(
        u8::try_from(NEXT.fetch_add(1, Ordering::Relaxed) % 26).fixture("token variation"),
    ); 32];
    ApiToken::from_mut_bytes(origin, &mut bytes).fixture("token")
}
pub(super) fn name() -> CrateName<'static> {
    CrateName::new("serde").fixture("name")
}
pub(super) fn yank() -> YankRequest<'static> {
    YankRequest::yank(name(), Version::new("1.0.0").fixture("version"))
}
#[test]
fn yank_unyank_and_owner_permits_have_three_mode_wire_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let expected = [b'a'.saturating_add(
        u8::try_from(NEXT.fetch_add(1, Ordering::Relaxed) % 26).fixture("variation"),
    ); 32];
    let mut source = expected;
    let token =
        ApiToken::from_mut_bytes(CredentialOrigin::Production, &mut source).fixture("token");
    assert!(source.iter().all(|byte| *byte == 0));
    fn authorized<'a>(mut fixture: Fixture<'a>, expected: &'a [u8]) -> Fixture<'a> {
        fixture.expected_auth = Some(expected);
        fixture
    }
    parity(
        || yank().confirm(&token),
        authorized(
            Fixture::new(
                Method::Delete,
                "/api/v1/crates/serde/1.0.0/yank",
                b"",
                br#"{"ok":true}"#,
                200,
            ),
            &expected,
        ),
    );
    parity(
        || YankRequest::unyank(name(), Version::new("1.0.0").fixture("version")).confirm(&token),
        authorized(
            Fixture::new(
                Method::Put,
                "/api/v1/crates/serde/1.0.0/unyank",
                b"",
                br#"{"ok":true}"#,
                200,
            ),
            &expected,
        ),
    );
    let owners = [OwnerSelector::new("alice").fixture("owner")];
    parity(
        || {
            OwnerChangeRequest::add(name(), &owners)
                .fixture("request")
                .confirm_add(&token)
                .fixture("permit")
        },
        authorized(
            Fixture::new(
                Method::Put,
                "/api/v1/crates/serde/owners",
                br#"{"users":["alice"]}"#,
                br#"{"ok":true,"msg":"invited"}"#,
                200,
            ),
            &expected,
        ),
    );
    parity(
        || {
            OwnerChangeRequest::remove(name(), &owners)
                .fixture("request")
                .confirm_removal(&token)
                .fixture("permit")
        },
        authorized(
            Fixture::new(
                Method::Delete,
                "/api/v1/crates/serde/owners",
                br#"{"users":["alice"]}"#,
                br#"{"ok":true,"msg":"removed"}"#,
                200,
            ),
            &expected,
        ),
    );
}
#[test]
fn personal_api_permits_have_three_mode_wire_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let token = token(CredentialOrigin::Production);
    let id = NumericId::new(42).fixture("id");
    let notifications = [NotificationUpdate {
        crate_id: id,
        enabled: true,
    }];
    for (index, (request, path, body, response)) in [
        (
            PersonalRequest::follow(name()),
            "/api/v1/crates/serde/follow",
            b"".as_slice(),
            br#"{"ok":true}"#.as_slice(),
        ),
        (
            PersonalRequest::unfollow(name()),
            "/api/v1/crates/serde/follow",
            b"",
            br#"{"ok":true}"#,
        ),
        (
            PersonalRequest::accept_invitation(id),
            "/api/v1/me/crate_owner_invitations/42",
            br#"{"crate_owner_invite":{"crate_id":42,"accepted":true}}"#,
            br#"{"crate_owner_invitation":{"crate_id":42,"accepted":true}}"#,
        ),
        (
            PersonalRequest::decline_invitation(id),
            "/api/v1/me/crate_owner_invitations/42",
            br#"{"crate_owner_invite":{"crate_id":42,"accepted":false}}"#,
            br#"{"crate_owner_invitation":{"crate_id":42,"accepted":false}}"#,
        ),
        (
            PersonalRequest::resend_email(id),
            "/api/v1/users/42/resend",
            b"",
            br#"{"ok":true}"#,
        ),
        (
            PersonalRequest::update_email(
                id,
                EmailAddress::new("test@example.org").fixture("email"),
            ),
            "/api/v1/users/42",
            br#"{"user":{"email":"test@example.org"}}"#,
            br#"{"ok":true}"#,
        ),
        (
            PersonalRequest::publish_notifications(id, true),
            "/api/v1/users/42",
            br#"{"user":{"publish_notifications":true}}"#,
            br#"{"ok":true}"#,
        ),
        (
            PersonalRequest::legacy_email_notifications(&notifications).fixture("updates"),
            "/api/v1/me/email_notifications",
            br#"[{"id":42,"email_notifications":true}]"#,
            br#"{"ok":true}"#,
        ),
    ]
    .into_iter()
    .enumerate()
    {
        parity(
            || {
                match index {
                    0 => PersonalRequest::follow(name()),
                    1 => PersonalRequest::unfollow(name()),
                    2 => PersonalRequest::accept_invitation(id),
                    3 => PersonalRequest::decline_invitation(id),
                    4 => PersonalRequest::resend_email(id),
                    5 => PersonalRequest::update_email(
                        id,
                        EmailAddress::new("test@example.org").fixture("email"),
                    ),
                    6 => PersonalRequest::publish_notifications(id, true),
                    7 => PersonalRequest::legacy_email_notifications(&notifications)
                        .fixture("updates"),
                    _ => unreachable!("fixture index"),
                }
                .confirm(&token)
            },
            Fixture::new(request.operation().method(), path, body, response, 200),
        );
    }
}
#[test]
fn anonymous_facade_keeps_three_mode_parity() {
    let _lock = TEST_GATE_LOCK.lock().fixture("gate");
    let mut fixture = Fixture::new(
        Method::Get,
        "/api/v1/site_metadata",
        b"",
        include_bytes!("../../../discovery/fixtures/get_site_metadata.json"),
        200,
    );
    fixture.auth = false;
    parity(crate::discovery::DiscoveryRequest::site_metadata, fixture);
}
