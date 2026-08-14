use alloc::format;

use super::support::*;
use super::*;

const RULES: &str = r#"{"input":[{"ip_version":"ipv4","name":"HTTPS","dst_ip":"192.0.2.0/24","src_ip":null,"dst_port":"443","src_port":null,"protocol":"tcp","tcp_flags":"syn|ack","action":"accept"}],"output":[]}"#;

fn pending<'a>(
    request: &RobotFirewallTemplateCreateRequest<'a>,
) -> PendingRobotFirewallTemplate<'a> {
    let body = format!(
        r#"{{"firewall_template":{{"id":17,"filter_ipv6":false,"whitelist_hos":true,"is_default":false,"rules":{RULES}}}}}"#
    );
    decode_template_create(request, &body)
        .unwrap_or_else(|_| unreachable!("pending template failed"))
        .into_confirmed()
        .err()
        .unwrap_or_else(|| unreachable!("pending state was erased"))
}

#[test]
fn pending_reconciliation_rejects_substituted_intent_and_torn_summaries() {
    let request_rules = [request_rule()];
    let request = RobotFirewallTemplateCreateRequest::new(template_config(&request_rules));
    let mismatches = [
        r#"{"id":18,"name":"baseline","filter_ipv6":false,"whitelist_hos":true,"is_default":false}"#,
        r#"{"id":17,"name":"changed","filter_ipv6":false,"whitelist_hos":true,"is_default":false}"#,
        r#"{"id":17,"name":"baseline","filter_ipv6":true,"whitelist_hos":true,"is_default":false}"#,
        r#"{"id":17,"name":"baseline","filter_ipv6":false,"whitelist_hos":false,"is_default":false}"#,
        r#"{"id":17,"name":"baseline","filter_ipv6":false,"whitelist_hos":true,"is_default":true}"#,
    ];
    for mismatch in mismatches {
        let body = format!(r#"[{{"firewall_template":{mismatch}}}]"#);
        let list = decode_template_list(body.as_bytes())
            .unwrap_or_else(|_| unreachable!("summary mismatch fixture failed"));
        let summary = list
            .as_slice()
            .first()
            .unwrap_or_else(|| unreachable!("summary mismatch disappeared"));
        assert!(pending(&request).reconcile_with_summary(summary).is_err());
    }
}
