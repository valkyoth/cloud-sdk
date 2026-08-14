use alloc::{format, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    OperationImpact, PreparationStorage, PrepareOperation, RequestBodySensitivity,
    ResponsePolicyError,
};
use cloud_sdk::transport::StatusCode;

use super::*;

mod support;
use support::*;

const RULE_JSON: &str = r#"{"ip_version":"ipv4","name":"HTTPS","dst_ip":"192.0.2.0/24","src_ip":null,"dst_port":"443","src_port":null,"protocol":"tcp","tcp_flags":"syn|ack","action":"accept"}"#;

fn firewall_json(server: u64, status: &str, rules: &str) -> alloc::string::String {
    format!(
        r#"{{"firewall":{{"server_ip":"192.0.2.1","server_number":{server},"status":"{status}","filter_ipv6":false,"whitelist_hos":true,"port":"main","rules":{rules}}}}}"#
    )
}

fn template_json(id: u64, name: &str, rules: &str) -> alloc::string::String {
    format!(
        r#"{{"firewall_template":{{"id":{id},"name":"{name}","filter_ipv6":false,"whitelist_hos":true,"is_default":false,"rules":{rules}}}}}"#
    )
}

fn template_json_without_name(id: u64, rules: &str) -> alloc::string::String {
    format!(
        r#"{{"firewall_template":{{"id":{id},"filter_ipv6":false,"whitelist_hos":true,"is_default":false,"rules":{rules}}}}}"#
    )
}

#[test]
fn validates_source_locked_rule_values_and_conflicts() {
    assert!(RobotFirewallTemplateId::new(0).is_err());
    for misleading in [
        '\u{061c}', '\u{200b}', '\u{200e}', '\u{200f}', '\u{202a}', '\u{202e}', '\u{2060}',
        '\u{2066}', '\u{2069}', '\u{feff}',
    ] {
        let name = format!("visible{misleading}hidden");
        assert!(RobotFirewallTemplateName::new(&name).is_err());
        assert!(
            RobotFirewallRule::new(RobotFirewallAction::Accept)
                .with_name(&name)
                .is_err()
        );
    }
    assert!(RobotFirewallCidr::new("192.0.2.1/24").is_err());
    assert!(RobotFirewallCidr::new("192.0.2.0/24").is_ok());
    assert!(RobotFirewallCidr::new("01.2.3.4").is_err());
    assert!(RobotFirewallPortRange::new("0").is_err());
    assert!(RobotFirewallPortRange::new("443-80").is_err());
    assert_eq!(port("80-443").bounds(), (80, 443));
    assert!(RobotFirewallTcpFlags::new("syn||ack").is_err());

    let ip_without_version =
        RobotFirewallRule::new(RobotFirewallAction::Accept).with_source_ip(cidr("192.0.2.0/24"));
    assert_eq!(
        ip_without_version.validate().err(),
        Some(RobotFirewallRuleError::FieldConflict)
    );
    let ipv6_with_ip = ip_without_version.with_ip_version(RobotFirewallIpVersion::Ipv6);
    assert_eq!(
        ipv6_with_ip.validate().err(),
        Some(RobotFirewallRuleError::FieldConflict)
    );
    let flags_without_tcp =
        RobotFirewallRule::new(RobotFirewallAction::Accept).with_tcp_flags(flags("syn"));
    assert_eq!(
        flags_without_tcp.validate().err(),
        Some(RobotFirewallRuleError::FieldConflict)
    );
    let port_without_protocol = RobotFirewallRule::new(RobotFirewallAction::Accept)
        .with_ip_version(RobotFirewallIpVersion::Ipv4)
        .with_destination_port(port("80"));
    assert!(port_without_protocol.validate().is_ok());
    assert_eq!(
        port_without_protocol
            .with_protocol(RobotFirewallProtocol::Icmp)
            .validate()
            .err(),
        Some(RobotFirewallRuleError::FieldConflict)
    );
}

#[test]
fn ordered_rules_reject_exact_duplicates() {
    let rule = request_rule();
    assert_eq!(
        RobotFirewallRules::new(&[rule, rule], &[]).err(),
        Some(RobotFirewallRuleError::DuplicateRule)
    );
    let distinct = RobotFirewallRule::new(RobotFirewallAction::Discard);
    let ordered = [rule, distinct];
    let rules = RobotFirewallRules::new(&ordered, &[])
        .unwrap_or_else(|_| unreachable!("distinct rules failed"));
    assert_eq!(rules.input(), &[rule, distinct]);
}

#[test]
fn prepares_all_eight_operations_with_exact_policy() {
    let rules = [request_rule()];
    assert_prepared(
        RobotFirewallGetRequest::new(server(321)),
        Method::Get,
        "/firewall/321",
        "robot_get_firewall",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallReplaceRequest::new(server(321), inline_intent(&rules)),
        Method::Post,
        "/firewall/321",
        "robot_update_firewall",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallDeleteRequest::new(server(321)),
        Method::Delete,
        "/firewall/321",
        "robot_delete_firewall",
        OperationImpact::Destructive,
        RequestBodySensitivity::Public,
        MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallTemplateListRequest::new(),
        Method::Get,
        "/firewall/template",
        "robot_list_firewall_templates",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallTemplateCreateRequest::new(template_config(&rules)),
        Method::Post,
        "/firewall/template",
        "robot_create_firewall_template",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallTemplateGetRequest::new(template_id()),
        Method::Get,
        "/firewall/template/17",
        "robot_get_firewall_template",
        OperationImpact::ReadOnly,
        RequestBodySensitivity::Public,
        MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallTemplateUpdateRequest::new(template_id(), template_config(&rules)),
        Method::Post,
        "/firewall/template/17",
        "robot_update_firewall_template",
        OperationImpact::Mutation,
        RequestBodySensitivity::Sensitive,
        MAX_ROBOT_FIREWALL_ITEM_RESPONSE_BYTES,
    );
    assert_prepared(
        RobotFirewallTemplateDeleteRequest::new(template_id()),
        Method::Delete,
        "/firewall/template/17",
        "robot_delete_firewall_template",
        OperationImpact::Destructive,
        RequestBodySensitivity::Public,
        0,
    );
}

#[test]
fn inline_and_template_forms_are_exact_and_mutually_exclusive() {
    let rules = [request_rule()];
    let inline = RobotFirewallReplaceRequest::new(server(321), inline_intent(&rules));
    let body = prepare_body(&inline);
    assert_eq!(
        body,
        "status=active&filter_ipv6=false&whitelist_hos=true&rules%5Binput%5D%5B0%5D%5Bname%5D=HTTPS&rules%5Binput%5D%5B0%5D%5Bip_version%5D=ipv4&rules%5Binput%5D%5B0%5D%5Bdst_ip%5D=192.0.2.0%2F24&rules%5Binput%5D%5B0%5D%5Bdst_port%5D=443&rules%5Binput%5D%5B0%5D%5Bprotocol%5D=tcp&rules%5Binput%5D%5B0%5D%5Btcp_flags%5D=syn%7Cack&rules%5Binput%5D%5B0%5D%5Baction%5D=accept"
    );

    let template = RobotFirewallReplaceRequest::new(
        server(321),
        RobotFirewallReplaceIntent::Template {
            status: RobotFirewallStatus::Active,
            filter_ipv6: None,
            template_id: template_id(),
        },
    );
    assert_eq!(prepare_body(&template), "status=active&template_id=17");

    let source_rule = RobotFirewallRule::new(RobotFirewallAction::Accept)
        .with_name("rule 1")
        .unwrap_or_else(|_| unreachable!("official rule name failed"))
        .with_ip_version(RobotFirewallIpVersion::Ipv4)
        .with_source_ip(cidr("1.1.1.1"))
        .with_destination_port(port("80"));
    let source_rules = [source_rule];
    let source = RobotFirewallReplaceRequest::new(server(321), inline_intent(&source_rules));
    let body = prepare_body(&source);
    assert!(body.contains("rules%5Binput%5D%5B0%5D%5Bdst_port%5D=80"));
    assert!(!body.contains("protocol"));
}

#[test]
fn checked_get_is_strict_bound_and_redacted() {
    let body = firewall_json(
        321,
        "active",
        &format!(r#"{{"input":[{RULE_JSON}],"output":[]}}"#),
    );
    let result = decode_get(&body).unwrap_or_else(|_| unreachable!("firewall fixture failed"));
    assert_eq!(result.status(), RobotFirewallRuntimeStatus::Active);
    assert_eq!(result.rules().input().len(), 1);
    let Some(rule) = result.rules().input().first() else {
        unreachable!("validated firewall rule disappeared")
    };
    assert_eq!(rule.action(), RobotFirewallAction::Accept);
    assert_eq!(
        rule.try_with_name(|value| value.map(str::len))
            .unwrap_or_else(|_| unreachable!("protected name lost UTF-8")),
        Some(5)
    );
    let debug = format!("{result:?}");
    assert!(!debug.contains("192.0.2"));
    assert!(!debug.contains("HTTPS"));

    assert_eq!(
        decode_get(&firewall_json(999, "active", r#"{"input":[],"output":[]}"#)).err(),
        Some(RobotFirewallDecodeError::ResponseIdentityMismatch)
    );
    assert!(
        decode_get(&body.replace("\"port\":\"main\"", "\"port\":\"main\",\"extra\":1")).is_err()
    );
}

#[test]
fn official_source_examples_decode_and_expose_complete_policy() {
    let firewall = decode_get(include_str!(
        "../../../../../tests/fixtures/robot-firewall/official-firewall-response.json"
    ))
    .unwrap_or_else(|_| unreachable!("official firewall response failed"));
    let Some(rule) = firewall.rules().input().first() else {
        unreachable!("official input rule disappeared")
    };
    assert_eq!(rule.protocol(), None);
    assert_eq!(
        rule.try_with_destination_port(|value| value == Some("80")),
        Ok(true)
    );
    assert_eq!(rule.try_with_source_port(|value| value.is_none()), Ok(true));
    assert_eq!(rule.try_with_tcp_flags(|value| value.is_none()), Ok(true));

    let request = RobotFirewallTemplateGetRequest::new(
        RobotFirewallTemplateId::new(123)
            .unwrap_or_else(|_| unreachable!("official template ID failed")),
    );
    let template = decode_template_get(
        &request,
        include_str!(
            "../../../../../tests/fixtures/robot-firewall/official-template-response.json"
        ),
    )
    .unwrap_or_else(|_| unreachable!("official template response failed"));
    assert_eq!(
        template.summary().try_with_name(|value| value.is_none()),
        Ok(true)
    );
    assert!(!template.summary().filter_ipv6());
    assert!(template.summary().whitelist_hos());
}

#[test]
fn mutations_require_exact_outcomes() {
    let request_rules = [request_rule()];
    let rules = format!(r#"{{"input":[{RULE_JSON}],"output":[]}}"#);
    let replace = RobotFirewallReplaceRequest::new(server(321), inline_intent(&request_rules));
    assert!(decode_replace(&replace, &firewall_json(321, "in process", &rules)).is_ok());
    assert_eq!(
        decode_replace(&replace, &firewall_json(321, "active", &rules)).err(),
        Some(RobotFirewallDecodeError::MutationOutcomeMismatch)
    );
    assert_eq!(
        decode_replace(
            &replace,
            &firewall_json(321, "in process", r#"{"input":[],"output":[]}"#)
        )
        .err(),
        Some(RobotFirewallDecodeError::MutationOutcomeMismatch)
    );

    let delete = RobotFirewallDeleteRequest::new(server(321));
    assert!(decode_delete(&delete, &firewall_json(321, "in process", r#"{}"#)).is_ok());
    assert_eq!(
        decode_delete(&delete, &firewall_json(321, "in process", &rules)).err(),
        Some(RobotFirewallDecodeError::MutationOutcomeMismatch)
    );
}

#[test]
fn template_inventory_and_mutations_are_request_bound() {
    let summary = r#"{"firewall_template":{"id":17,"name":"baseline","filter_ipv6":false,"whitelist_hos":true,"is_default":false}}"#;
    let list = decode_template_list(format!("[{summary}]").as_bytes())
        .unwrap_or_else(|_| unreachable!("template list failed"));
    assert_eq!(list.len(), 1);
    assert!(decode_template_list(format!("[{summary},{summary}]").as_bytes()).is_err());

    let rules = format!(r#"{{"input":[{RULE_JSON}],"output":[]}}"#);
    let body = template_json(17, "baseline", &rules);
    let get = RobotFirewallTemplateGetRequest::new(template_id());
    assert!(decode_template_get(&get, &body).is_ok());
    let wrong = RobotFirewallTemplateGetRequest::new(
        RobotFirewallTemplateId::new(18).unwrap_or_else(|_| unreachable!("template ID failed")),
    );
    assert_eq!(
        decode_template_get(&wrong, &body).err(),
        Some(RobotFirewallDecodeError::ResponseIdentityMismatch)
    );
    let request_rules = [request_rule()];
    let create = RobotFirewallTemplateCreateRequest::new(template_config(&request_rules));
    let confirmed = decode_template_create(&create, &body)
        .unwrap_or_else(|_| unreachable!("confirmed template failed"));
    assert!(confirmed.is_confirmed());
    assert_eq!(
        confirmed
            .template()
            .reconcile(template_config(&request_rules)),
        RobotFirewallTemplateReconciliation::Confirmed
    );
    let unconfirmed = decode_template_create(&create, &template_json_without_name(17, &rules))
        .unwrap_or_else(|_| unreachable!("no-name template failed"));
    assert!(!unconfirmed.is_confirmed());
    assert_eq!(
        unconfirmed
            .template()
            .reconcile(template_config(&request_rules)),
        RobotFirewallTemplateReconciliation::NameUnconfirmed
    );
    assert_eq!(
        decode_template_create(&create, &template_json(17, "changed", &rules)).err(),
        Some(RobotFirewallDecodeError::MutationOutcomeMismatch)
    );
}

#[test]
fn list_response_enforces_complete_boundary() {
    let request = RobotFirewallTemplateListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    let body = vec![b' '; MAX_ROBOT_FIREWALL_TEMPLATE_LIST_RESPONSE_BYTES + 1];
    with_response(
        StatusCode::OK,
        &body,
        Some("application/json"),
        |response| {
            assert_eq!(
                prepared.validate_response(response).err(),
                Some(ResponsePolicyError::BodyTooLarge)
            );
        },
    );
}

#[test]
fn failed_preparation_clears_all_storage() {
    let rules = [request_rule()];
    let request = RobotFirewallReplaceRequest::new(server(321), inline_intent(&rules));
    let mut target = [0xa5_u8; 2];
    let mut body = [0x5a_u8; 4];
    assert!(
        request
            .prepare(PreparationStorage::new(&mut target, &mut body))
            .is_err()
    );
    assert_eq!(target, [0_u8; 2]);
    assert_eq!(body, [0_u8; 4]);
}

#[test]
fn mutation_and_destructive_authority_require_strong_request_bound_digests() {
    let rules = [request_rule()];
    let replace = RobotFirewallReplaceRequest::new(server(321), inline_intent(&rules));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 16_384];
    let prepared = replace
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("replace preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_firewall_plan_digest(
        plan(prepared),
        &mut scratch,
        &mut digest,
        &crate::association::Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("replace digest failed"));
    assert_eq!(scratch, [0_u8; 4_096]);
    let mut permit = RobotFirewallMutationPermit::new(
        fingerprint.subject(),
        cloud_sdk::operation::PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("mutation permit failed"));
    assert!(
        permit
            .begin(cloud_sdk::operation::PermitTimestamp::from_seconds(101))
            .is_ok()
    );

    let delete = RobotFirewallDeleteRequest::new(server(321));
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 1];
    let prepared = delete
        .prepare_bound(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    let mut scratch = [0xa5_u8; 4_096];
    let mut digest = [0x5a_u8; 32];
    let fingerprint = build_robot_firewall_plan_digest(
        plan(prepared),
        &mut scratch,
        &mut digest,
        &crate::association::Sha256PlanHasher,
    )
    .unwrap_or_else(|_| unreachable!("delete digest failed"));
    let mut permit = RobotFirewallDestructivePermit::new(
        fingerprint.subject(),
        cloud_sdk::operation::PermitTimestamp::from_seconds(100),
    )
    .unwrap_or_else(|_| unreachable!("destructive permit failed"));
    assert!(
        permit
            .begin(cloud_sdk::operation::PermitTimestamp::from_seconds(101))
            .is_ok()
    );
}
