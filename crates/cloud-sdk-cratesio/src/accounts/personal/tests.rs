use super::*;
use crate::{
    credentials::{ApiToken, CredentialOrigin},
    discovery::tests::Fixture as _,
    identifiers::{CrateName, NumericId},
};
use alloc::{format, vec};
fn id(n: u64) -> NumericId {
    NumericId::new(n).fixture("id")
}
fn token() -> ApiToken {
    // Disposable lexical fixture, not a network credential.
    let mut bytes = [runtime_token_byte(); 32];
    ApiToken::from_mut_bytes(CredentialOrigin::Production, &mut bytes).fixture("token")
}
fn runtime_token_byte() -> u8 {
    use core::sync::atomic::{AtomicU8, Ordering};
    static NEXT: AtomicU8 = AtomicU8::new(0);
    b'a'.saturating_add(NEXT.fetch_add(1, Ordering::Relaxed) % 26)
}
#[test]
fn every_api_action_has_exact_target_and_body() {
    let changes = [NotificationUpdate {
        crate_id: id(42),
        enabled: false,
    }];
    let rows = [
        (
            PersonalRequest::follow(CrateName::new("serde").fixture("name")),
            "/api/v1/crates/serde/follow",
            "",
        ),
        (
            PersonalRequest::unfollow(CrateName::new("serde").fixture("name")),
            "/api/v1/crates/serde/follow",
            "",
        ),
        (
            PersonalRequest::accept_invitation(id(42)),
            "/api/v1/me/crate_owner_invitations/42",
            "{\"crate_owner_invite\":{\"crate_id\":42,\"accepted\":true}}",
        ),
        (
            PersonalRequest::decline_invitation(id(42)),
            "/api/v1/me/crate_owner_invitations/42",
            "{\"crate_owner_invite\":{\"crate_id\":42,\"accepted\":false}}",
        ),
        (
            PersonalRequest::resend_email(id(7)),
            "/api/v1/users/7/resend",
            "",
        ),
        (
            PersonalRequest::update_email(
                id(7),
                EmailAddress::new("private@example.org").fixture("email"),
            ),
            "/api/v1/users/7",
            "{\"user\":{\"email\":\"private@example.org\"}}",
        ),
        (
            PersonalRequest::publish_notifications(id(7), false),
            "/api/v1/users/7",
            "{\"user\":{\"publish_notifications\":false}}",
        ),
        (
            PersonalRequest::legacy_email_notifications(&changes).fixture("batch"),
            "/api/v1/me/email_notifications",
            "[{\"id\":42,\"email_notifications\":false}]",
        ),
    ];
    for (request, path, json) in rows {
        let mut target = [0; 256];
        assert_eq!(
            request
                .write_target(&mut target)
                .fixture("path")
                .as_request_target()
                .path()
                .as_str(),
            path
        );
        let mut body = [0xa5; 4096];
        let len = request.body(&mut body).fixture("body");
        assert_eq!(body.get(..len).fixture("slice"), json.as_bytes());
        for length in 0..len {
            let mut short = vec![0xa5; length];
            assert!(request.body(&mut short).is_err());
            assert!(short.iter().all(|v| *v == 0xa5));
        }
    }
}
#[test]
fn notification_batches_reject_ambiguity_and_excess() {
    assert!(PersonalRequest::legacy_email_notifications(&[]).is_err());
    let change = NotificationUpdate {
        crate_id: id(1),
        enabled: true,
    };
    assert!(PersonalRequest::legacy_email_notifications(&[change, change]).is_err());
    let changes: alloc::vec::Vec<_> = (1..=65)
        .map(|n| NotificationUpdate {
            crate_id: id(n),
            enabled: false,
        })
        .collect();
    assert!(PersonalRequest::legacy_email_notifications(&changes).is_err());
    let request = PersonalRequest::legacy_email_notifications(changes.get(..64).fixture("bound"))
        .fixture("maximum");
    let mut body = [0; MAX_PERSONAL_BODY_BYTES];
    assert!(request.body(&mut body).is_ok());
}

#[test]
fn scoped_invitations_reject_unrelated_or_contradictory_responses() {
    use cloud_sdk::incremental_json::IncrementalJsonDecoder;
    use invitations::{Decision, Filter, InvitationId, InvitationPermit};
    let token = token();
    let list = InvitationPermit::list(
        &token,
        Filter::Invitee(id(3)),
        crate::query::PerPage::DEFAULT,
        None,
    );
    let update = InvitationPermit::confirm_update(
        &token,
        InvitationId::new(19).fixture("id"),
        Decision::Accept,
    );
    let item = r#"{"id":19,"crate_id":4,"inviter_id":2,"invitee_id":3,"state":"pending","crate_name":"serde","created_at":"2026-10-21T00:00:00Z","expires_at":"2026-11-21T00:00:00Z"}"#;
    let check = |permit: &InvitationPermit<'_>, body: &str| {
        let mut builder = crate::discovery::value::Builder::default();
        let mut parser = IncrementalJsonDecoder::new();
        parser.push(body.as_bytes(), &mut builder).fixture("parse");
        parser.finish(&mut builder).fixture("finish");
        permit.check_tree(builder.finish().fixture("tree"))
    };
    let page = |items: &str| {
        format!(r#"{{"crate_owner_invitations":[{items}],"users":[],"meta":{{"next_page":null}}}}"#)
    };
    assert!(check(&list, &page(item)).is_ok());
    let user = r#"{"id":3,"login":"user","name":null,"avatar":null,"url":"https://example.org","github_username_matches":true,"created_at":null}"#;
    let with_users =
        |users: &str| page(item).replace("\"users\":[]", &format!("\"users\":[{users}]"));
    assert!(check(&list, &with_users(user)).is_ok());
    assert!(check(&list, &with_users(&user.replace("\"id\":3", "\"id\":12"))).is_err());
    assert!(check(&list, &with_users(&format!("{user},{user}"))).is_err());
    for bad in [
        item.replace("pending", "accepted"),
        item.replace("\"invitee_id\":3", "\"invitee_id\":4"),
        format!("{item},{item}"),
    ] {
        assert!(check(&list, &page(&bad)).is_err());
    }
    let reply = format!(r#"{{"crate_owner_invitation":{item}}}"#).replace("pending", "accepted");
    assert!(check(&update, &reply).is_ok());
    for bad in [
        reply.replace("19,", "18,"),
        reply.replace("accepted", "declined"),
        reply.replace("\"crate_id\":4", "\"crate_id\":0"),
    ] {
        assert!(check(&update, &bad).is_err());
    }
}

#[test]
fn scoped_invitation_transitions_use_the_documented_body_and_id_bounds() {
    use cloud_sdk::incremental_json::IncrementalJsonDecoder;
    use invitations::{Decision, InvitationId, InvitationPermit};
    let token = token();
    assert!(InvitationId::new(0).is_err());
    assert!(InvitationId::new(u64::MAX).is_err());
    let id = InvitationId::new(i64::MAX as u64).fixture("maximum");
    for decision in [Decision::Accept, Decision::Decline, Decision::Revoke] {
        let permit = InvitationPermit::confirm_update(&token, id, decision);
        let mut builder = crate::discovery::value::Builder::default();
        let mut parser = IncrementalJsonDecoder::new();
        parser
            .push(permit.wire_body().1, &mut builder)
            .fixture("parse");
        parser.finish(&mut builder).fixture("finish");
        crate::catalog::schema::validate_table(
            &builder.finish().fixture("tree"),
            schema_table::UPDATE_CRATE_OWNER_INVITATION_BODY,
            0,
            schema_table::NODES,
        )
        .fixture("source schema");
        assert!(permit.write_target(&mut [0; 4]).is_err());
    }
}

#[test]
fn serialized_bodies_match_source_projections_and_scoped_storage_is_cleared() {
    use super::schema_table as s;
    use crate::discovery::value::Builder;
    use cloud_sdk::incremental_json::IncrementalJsonDecoder;
    for (request, schema) in [
        (
            PersonalRequest::accept_invitation(id(42)),
            s::HANDLE_CRATE_OWNER_INVITATION_BODY,
        ),
        (
            PersonalRequest::decline_invitation(id(42)),
            s::HANDLE_CRATE_OWNER_INVITATION_BODY,
        ),
        (
            PersonalRequest::update_email(
                id(7),
                EmailAddress::new("private@example.org").fixture("email"),
            ),
            s::UPDATE_USER_BODY,
        ),
        (
            PersonalRequest::publish_notifications(id(7), true),
            s::UPDATE_USER_BODY,
        ),
    ] {
        let mut output = [0xa5; 1024];
        request
            .with_json_body(&mut output, |bytes| {
                let mut builder = Builder::default();
                let mut parser = IncrementalJsonDecoder::new();
                parser.push(bytes, &mut builder).fixture("parse");
                parser.finish(&mut builder).fixture("finish");
                crate::catalog::schema::validate_table(
                    &builder.finish().fixture("tree"),
                    schema,
                    0,
                    s::NODES,
                )
                .fixture("source schema");
            })
            .fixture("scoped body");
        assert!(output.iter().all(|v| *v == 0));
        let mut short = [0xa5; 1];
        assert!(
            request
                .with_json_body(&mut short, |_| unreachable!("short buffer exposed"))
                .is_err()
        );
        assert_eq!(short, [0]);
    }
}
#[test]
fn email_validation_and_serialization_are_bounded_and_redacted() {
    for value in ["", "x", "@a", "a@", "a@b@c", " x@y", "x@y\n", "x\0@y"] {
        assert!(EmailAddress::new(value).is_err());
    }
    assert!(EmailAddress::new(&"x".repeat(255)).is_err());
    let value = "\"private\\quoted\"@example.org";
    let email = EmailAddress::new(value).fixture("quoted email");
    let request = PersonalRequest::update_email(id(7), email);
    let mut bytes = [0; 1024];
    let len = request.body(&mut bytes).fixture("body");
    let json: serde_json::Value =
        serde_json::from_slice(bytes.get(..len).fixture("slice")).fixture("JSON");
    assert_eq!(
        json.get("user")
            .fixture("user")
            .get("email")
            .fixture("email"),
        value
    );
    assert!(!format!("{email:?} {request:?}").contains("private"));
    assert!(!format!("{:?}", request.confirm(&token())).contains("private"));
}
#[test]
fn follow_metadata_does_not_make_other_mutations_idempotent() {
    assert!(PersonalOperation::Follow.is_state_idempotent());
    assert!(PersonalOperation::Unfollow.is_state_idempotent());
    for op in [
        PersonalOperation::HandleInvitation,
        PersonalOperation::AcceptInvitationToken,
        PersonalOperation::Notifications,
        PersonalOperation::ResendEmail,
        PersonalOperation::UpdateUser,
        PersonalOperation::ConfirmEmail,
    ] {
        assert!(!op.is_state_idempotent());
        assert_eq!(op.method(), cloud_sdk::Method::Put);
    }
    assert_eq!(
        PersonalOperation::Unfollow.method(),
        cloud_sdk::Method::Delete
    );
}
fn decode(
    permit: &PersonalPermit<'_>,
    wire: &[u8],
) -> Result<PersonalResponse, crate::discovery::DiscoveryError> {
    use cloud_sdk::{
        rate_limit::WallClockTimestamp,
        transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode},
    };
    let mut bytes = vec![0xa5; wire.len()];
    let mut headers = [0xa5; 512];
    let mut response = ResponseBuffer::new(&mut bytes, wire.len(), &mut headers);
    let mut attempt = response.writer().begin_attempt().fixture("attempt");
    attempt.body_mut().fixture("body").copy_from_slice(wire);
    attempt
        .headers_mut()
        .fixture("headers")
        .try_push(
            "content-type",
            b"application/json",
            HeaderSensitivity::Public,
        )
        .fixture("media");
    attempt
        .commit(StatusCode::OK, wire.len(), ResponseMetadata::EMPTY)
        .fixture("commit");
    drop(attempt);
    let result = match crate::wire::JsonResponsePolicy::new(StatusCode::OK, wire.len())
        .fixture("policy")
        .admit(response, WallClockTimestamp::new(0))
    {
        Ok(success) => permit.decode_response(success),
        Err(_) => Err(crate::discovery::DiscoveryError::Json),
    };
    assert!(bytes.iter().all(|v| *v == 0));
    assert!(headers.iter().all(|v| *v == 0));
    result
}
#[test]
fn acknowledgement_requires_true_and_rejects_error_envelopes() {
    let token = token();
    let permit = PersonalRequest::resend_email(id(7)).confirm(&token);
    assert_eq!(
        decode(&permit, br#"{"ok":true}"#),
        Ok(PersonalResponse::Acknowledged)
    );
    for wire in [
        br#"{"ok":false}"#.as_slice(),
        br#"{"ok":"true"}"#,
        br#"{}"#,
        br#"{"ok":true,"errors":[]}"#,
        br#"{"ok":true,"ok":true}"#,
    ] {
        assert!(decode(&permit, wire).is_err());
    }
}
#[test]
fn invitation_response_binds_crate_and_decision() {
    let token = token();
    let permit = PersonalRequest::decline_invitation(id(42)).confirm(&token);
    assert_eq!(
        decode(
            &permit,
            br#"{"crate_owner_invitation":{"crate_id":42,"accepted":false}}"#
        ),
        Ok(PersonalResponse::Invitation {
            crate_id: id(42),
            accepted: false
        })
    );
    for wire in [
        br#"{"crate_owner_invitation":{"crate_id":43,"accepted":false}}"#.as_slice(),
        br#"{"crate_owner_invitation":{"crate_id":42,"accepted":true}}"#,
        br#"{"crate_owner_invitation":{"crate_id":-42,"accepted":false}}"#,
        br#"{"ok":true}"#,
    ] {
        assert!(decode(&permit, wire).is_err());
    }
}
#[cfg(feature = "blocking")]
mod admission;
#[cfg(feature = "blocking")]
mod execution;
