use super::*;
#[test]
fn exact_routes_bodies_methods_and_explicit_confirmation() {
    let token = api(CredentialOrigin::Production);
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let params = [Parameter::Crate(CrateName::new("regex").fixture("crate"))];
        let query = Query::new(p.query(), &params).fixture("query");
        for (permit, suffix) in [
            (
                TrustedPublishingPermit::list(p, query, &token).fixture("list"),
                "?crate=regex",
            ),
            (
                TrustedPublishingPermit::confirm_create(config(p), &token),
                "",
            ),
            (
                TrustedPublishingPermit::confirm_delete(p, id(42), &token),
                "/42",
            ),
        ] {
            let mut target = [0; 512];
            let provider = if p == Publisher::GitHub {
                "github"
            } else {
                "gitlab"
            };
            assert_eq!(
                permit.write_target(&mut target).fixture("target").as_str(),
                format!("/api/v1/trusted_publishing/{provider}_configs{suffix}")
            );
            let mut tiny = [0xa5; 2];
            assert!(permit.write_target(&mut tiny).is_err());
            assert_eq!(tiny, [0xa5; 2]);
            assert!(!permit.operation().permits_automatic_retry());
            let mut body = [0xa5; 4096];
            permit
                .with_configuration_body(&mut body, |bytes| {
                    if matches!(permit.operation(), TrustedPublishingOperation::Create(_)) {
                        let value: Value = serde_json::from_slice(bytes).fixture("JSON body");
                        assert_eq!(
                            value
                                .get(p.field())
                                .fixture("config")
                                .get("crate")
                                .fixture("crate"),
                            "regex"
                        );
                        assert_eq!(
                            value
                                .get(p.field())
                                .fixture("config")
                                .get("environment")
                                .fixture("environment"),
                            &Value::Null
                        );
                    } else {
                        assert!(bytes.is_empty());
                    }
                })
                .fixture("body");
            assert!(body.iter().all(|b| *b == 0));
        }
        for params in [
            vec![],
            vec![
                Parameter::Crate(CrateName::new("regex").fixture("crate")),
                Parameter::UserId(id(42)),
            ],
            vec![
                Parameter::Crate(CrateName::new("regex").fixture("crate")),
                Parameter::Page(crate::query::Page::new(1).fixture("page")),
            ],
        ] {
            let query = Query::new(p.query(), &params).fixture("query");
            assert!(TrustedPublishingPermit::list(p, query, &token).is_err());
        }
        assert!(
            TrustedPublishingPermit::list(
                p,
                Query::new(crate::query::QueryOperation::Crates, &[]).fixture("query"),
                &token
            )
            .is_err()
        );
    }
}
#[test]
fn conservative_configuration_profile_and_redaction() {
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let c = config(p);
        for (owner, project, workflow, env) in [
            ("", c.project, c.workflow, None),
            ("../bad", c.project, c.workflow, None),
            (c.owner, "..", c.workflow, None),
            (c.owner, "repo*", c.workflow, None),
            (c.owner, c.project, "../ci.yml", None),
            (c.owner, c.project, "/ci.yml", None),
            (c.owner, c.project, "ci.txt", None),
            (c.owner, c.project, c.workflow, Some("*")),
            (c.owner, c.project, c.workflow, Some(" production")),
            (c.owner, c.project, c.workflow, Some("${ENV}")),
            (c.owner, c.project, c.workflow, Some("bad\nname")),
        ] {
            assert!(PublisherConfig::new(p, c.krate, owner, project, workflow, env).is_err());
        }
        assert_eq!(format!("{c:?}"), "PublisherConfig([redacted])");
        assert!(
            PublisherConfig::new(p, c.krate, &"a".repeat(256), c.project, c.workflow, None)
                .is_err()
        );
        assert!(
            PublisherConfig::new(
                p,
                c.krate,
                &"a".repeat(255),
                c.project,
                c.workflow,
                Some("production")
            )
            .is_ok()
        );
    }
    assert!(
        PublisherConfig::new(
            Publisher::GitLab,
            config(Publisher::GitLab).krate,
            "group/sub",
            "project",
            "nested/ci.yml",
            None
        )
        .is_ok()
    );
}
#[test]
fn created_configs_are_bound_and_private() {
    let token = api(CredentialOrigin::Production);
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let name = if p == Publisher::GitHub {
            "create_trustpub_github_config"
        } else {
            "create_trustpub_gitlab_config"
        };
        let wire = fixture(name);
        let TrustedPublishingResponse::Created(c) = decode(
            TrustedPublishingPermit::confirm_create(config(p), &token),
            &wire,
            100,
        )
        .fixture("create") else {
            unreachable!("variant")
        };
        assert_eq!(c.id(), id(42));
        assert_eq!(c.publisher(), p);
        assert!(!format!("{c:?}").contains("rust-lang"));
        for (key, value) in [
            ("crate", json!("other")),
            (p.fields()[0], json!("other")),
            (p.fields()[1], json!("other")),
            (p.fields()[2], json!("other.yml")),
            ("environment", json!("production")),
            ("id", json!(0)),
            ("created_at", json!("2026-02-30T00:00:00Z")),
        ] {
            let mut changed = wire.clone();
            *changed
                .get_mut(p.field())
                .fixture("config")
                .get_mut(key)
                .fixture("field") = value;
            assert!(
                decode(
                    TrustedPublishingPermit::confirm_create(config(p), &token),
                    &changed,
                    100
                )
                .is_err()
            );
        }
    }
}
#[test]
fn seek_pages_reject_duplicates_filter_loss_and_cross_origin_links() {
    let token = api(CredentialOrigin::Production);
    for p in [Publisher::GitHub, Publisher::GitLab] {
        let params = [Parameter::Crate(CrateName::new("regex").fixture("crate"))];
        let q = Query::new(p.query(), &params).fixture("query");
        let name = if p == Publisher::GitHub {
            "list_trustpub_github_configs"
        } else {
            "list_trustpub_gitlab_configs"
        };
        let field = if p == Publisher::GitHub {
            "github_configs"
        } else {
            "gitlab_configs"
        };
        let mut wire = fixture(name);
        *wire.pointer_mut("/meta/next_page").fixture("next") = json!("?crate=regex&seek=abc123");
        let TrustedPublishingResponse::Configurations(page) = decode(
            TrustedPublishingPermit::list(p, q, &token).fixture("list"),
            &wire,
            100,
        )
        .fixture("page") else {
            unreachable!("variant")
        };
        assert_eq!(page.entries().len(), 1);
        assert_eq!(page.total(), 42);
        page.with_next_link(|next| assert_eq!(next, Some("?crate=regex&seek=abc123")))
            .fixture("link");
        for link in [
            "?seek=abc123",
            "?crate=other&seek=abc123",
            "?crate=regex&page=2",
            "https://evil.invalid/?crate=regex&seek=abc123",
        ] {
            let mut changed = wire.clone();
            *changed.pointer_mut("/meta/next_page").fixture("next") = json!(link);
            assert!(
                decode(
                    TrustedPublishingPermit::list(p, q, &token).fixture("list"),
                    &changed,
                    100
                )
                .is_err()
            );
        }
        for different_id in [false, true] {
            let mut changed = wire.clone();
            let mut duplicate = changed
                .get(field)
                .fixture("field")
                .get(0)
                .fixture("record")
                .clone();
            if different_id {
                *duplicate.get_mut("id").fixture("id") = json!(43);
            }
            changed
                .get_mut(field)
                .fixture("field")
                .as_array_mut()
                .fixture("array")
                .push(duplicate);
            assert!(
                decode(
                    TrustedPublishingPermit::list(p, q, &token).fixture("list"),
                    &changed,
                    100
                )
                .is_err()
            );
        }
        for count in [0, 11] {
            let mut changed = wire.clone();
            let record = changed
                .get(field)
                .fixture("field")
                .get(0)
                .fixture("record")
                .clone();
            *changed.get_mut(field).fixture("field") = json!(vec![record; count]);
            assert!(
                decode(
                    TrustedPublishingPermit::list(p, q, &token).fixture("list"),
                    &changed,
                    100
                )
                .is_err()
            );
        }
    }
}
