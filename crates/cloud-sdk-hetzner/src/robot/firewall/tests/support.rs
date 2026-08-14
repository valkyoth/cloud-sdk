use alloc::{string::String, vec};

use cloud_sdk::Method;
use cloud_sdk::operation::{
    AttemptBudget, OperationImpact, PermitContext, PermitTimestamp, PermitValidity, PlanChange,
    PlanFingerprintScope, PreparationStorage, PrepareOperation, ReplayPolicy,
    RequestBodySensitivity, RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::{HeaderSensitivity, ResponseBuffer, ResponseMetadata, StatusCode};

use super::super::*;
use crate::endpoint::official_robot_endpoint_identity;

#[allow(clippy::too_many_arguments)]
pub(super) fn assert_prepared<O>(
    operation: O,
    method: Method,
    target: &str,
    operation_id: &str,
    impact: OperationImpact,
    sensitivity: RequestBodySensitivity,
    maximum: usize,
) where
    O: PrepareOperation<Error = RobotFirewallRequestError>,
{
    let mut target_storage = [0_u8; 128];
    let mut body_storage = [0_u8; 16_384];
    let prepared = operation
        .prepare(PreparationStorage::new(
            &mut target_storage,
            &mut body_storage,
        ))
        .unwrap_or_else(|_| unreachable!("firewall preparation failed"));
    assert_eq!(prepared.transport_request().method(), method);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(operation_id)
    );
    assert_eq!(prepared.metadata().impact(), impact);
    assert_eq!(
        prepared.metadata().semantics(),
        if impact == OperationImpact::ReadOnly {
            RequestSemantics::Safe
        } else {
            RequestSemantics::NonIdempotent
        }
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        if impact == OperationImpact::ReadOnly {
            RetryEligibility::ExplicitPolicy
        } else {
            RetryEligibility::Never
        }
    );
    assert_eq!(prepared.body_sensitivity(), sensitivity);
    assert_eq!(prepared.response_policy().max_body_bytes(), maximum);
}

pub(super) fn prepare_body<O: PrepareOperation<Error = RobotFirewallRequestError>>(
    operation: &O,
) -> String {
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 16_384];
    let prepared = operation
        .prepare(PreparationStorage::new(&mut target, &mut body))
        .unwrap_or_else(|_| unreachable!("form preparation failed"));
    String::from(
        core::str::from_utf8(prepared.transport_request().body())
            .unwrap_or_else(|_| unreachable!("form lost UTF-8")),
    )
}

pub(super) fn plan<'storage, 'request, R: RobotFirewallPermitRequest>(
    prepared: PreparedRobotFirewall<'storage, 'request, R>,
) -> RobotFirewallPlanConfirmation<'static, 'storage, 'request, R> {
    RobotFirewallPlanConfirmation::new(
        prepared,
        official_robot_endpoint_identity().unwrap_or_else(|_| unreachable!("endpoint failed")),
        PlanFingerprintScope::Value(b"robot-account"),
        PlanFingerprintScope::Absent,
        PermitContext::new(b"v0.89 Robot firewall fixture")
            .unwrap_or_else(|_| unreachable!("context failed")),
        PermitValidity::new(
            PermitTimestamp::from_seconds(100),
            PermitTimestamp::from_seconds(125),
        )
        .unwrap_or_else(|_| unreachable!("validity failed")),
        ReplayPolicy::SingleAttempt,
        AttemptBudget::new(1).unwrap_or_else(|_| unreachable!("attempt budget failed")),
        PlanChange::ChangesState,
        None,
        None,
    )
}

pub(super) fn decode_get(body: &str) -> Result<RobotFirewall, RobotFirewallDecodeError> {
    let request = RobotFirewallGetRequest::new(server(321));
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("get preparation failed"));
    with_json(prepared, StatusCode::OK, body.as_bytes(), |checked| {
        checked.decode_response()
    })
}

pub(super) fn decode_replace(
    request: &RobotFirewallReplaceRequest<'_>,
    body: &str,
) -> Result<RobotFirewall, RobotFirewallDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16_384];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("replace preparation failed"));
    with_json(prepared, StatusCode::OK, body.as_bytes(), |checked| {
        checked.decode_response()
    })
}

pub(super) fn decode_delete(
    request: &RobotFirewallDeleteRequest,
    body: &str,
) -> Result<RobotFirewall, RobotFirewallDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("delete preparation failed"));
    with_json(prepared, StatusCode::OK, body.as_bytes(), |checked| {
        checked.decode_response()
    })
}

pub(super) fn decode_template_list(
    body: &[u8],
) -> Result<RobotFirewallTemplateList, RobotFirewallDecodeError> {
    let request = RobotFirewallTemplateListRequest::new();
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("list preparation failed"));
    with_json(prepared, StatusCode::OK, body, |checked| {
        checked.decode_response()
    })
}

pub(super) fn decode_template_get(
    request: &RobotFirewallTemplateGetRequest,
    body: &str,
) -> Result<RobotFirewallTemplate, RobotFirewallDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 1];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("template get preparation failed"));
    with_json(prepared, StatusCode::OK, body.as_bytes(), |checked| {
        checked.decode_response()
    })
}

pub(super) fn decode_template_create(
    request: &RobotFirewallTemplateCreateRequest<'_>,
    body: &str,
) -> Result<RobotFirewallTemplateMutationOutcome, RobotFirewallDecodeError> {
    let mut target = [0_u8; 128];
    let mut request_body = [0_u8; 16_384];
    let prepared = request
        .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
        .unwrap_or_else(|_| unreachable!("template create preparation failed"));
    with_json(prepared, StatusCode::CREATED, body.as_bytes(), |checked| {
        checked.decode_response()
    })
}

fn with_json<'request, R, O>(
    prepared: PreparedRobotFirewall<'_, 'request, R>,
    status: StatusCode,
    body: &[u8],
    decode: impl FnOnce(CheckedRobotFirewall<'_, 'request, R>) -> O,
) -> O {
    with_response(status, body, Some("application/json"), |response| {
        let checked = prepared
            .validate_response(response)
            .unwrap_or_else(|_| unreachable!("response policy failed"));
        decode(checked)
    })
}

pub(super) fn with_response<R>(
    status: StatusCode,
    body: &[u8],
    content_type: Option<&str>,
    inspect: impl FnOnce(ResponseBuffer<'_>) -> R,
) -> R {
    let mut storage = vec![0_u8; body.len()];
    let mut headers = [0_u8; 128];
    let mut response = ResponseBuffer::new(&mut storage, body.len(), &mut headers);
    let mut attempt = response
        .writer()
        .begin_attempt()
        .unwrap_or_else(|_| unreachable!("response attempt failed"));
    if let Some(content_type) = content_type {
        attempt
            .headers_mut()
            .unwrap_or_else(|_| unreachable!("headers failed"))
            .try_push(
                "content-type",
                content_type.as_bytes(),
                HeaderSensitivity::Public,
            )
            .unwrap_or_else(|_| unreachable!("content type failed"));
    }
    attempt
        .body_mut()
        .unwrap_or_else(|_| unreachable!("body failed"))
        .copy_from_slice(body);
    attempt
        .commit(status, body.len(), ResponseMetadata::EMPTY)
        .unwrap_or_else(|_| unreachable!("response commit failed"));
    drop(attempt);
    inspect(response)
}

pub(super) fn request_rule() -> RobotFirewallRule<'static> {
    RobotFirewallRule::new(RobotFirewallAction::Accept)
        .with_name("HTTPS")
        .unwrap_or_else(|_| unreachable!("rule name failed"))
        .with_ip_version(RobotFirewallIpVersion::Ipv4)
        .with_destination_ip(cidr("192.0.2.0/24"))
        .with_destination_port(port("443"))
        .with_protocol(RobotFirewallProtocol::Tcp)
        .with_tcp_flags(flags("syn|ack"))
}

fn request_rules<'a>(input: &'a [RobotFirewallRule<'static>]) -> RobotFirewallRules<'a> {
    RobotFirewallRules::new(input, &[]).unwrap_or_else(|_| unreachable!("rules failed"))
}

pub(super) fn inline_intent<'a>(
    input: &'a [RobotFirewallRule<'static>],
) -> RobotFirewallReplaceIntent<'a> {
    RobotFirewallReplaceIntent::Inline {
        status: RobotFirewallStatus::Active,
        filter_ipv6: Some(false),
        whitelist_hos: true,
        rules: request_rules(input),
    }
}

pub(super) fn template_config<'a>(
    input: &'a [RobotFirewallRule<'static>],
) -> RobotFirewallTemplateConfig<'a> {
    RobotFirewallTemplateConfig::new(
        RobotFirewallTemplateName::new("baseline").unwrap_or_else(|_| unreachable!("name failed")),
        true,
        false,
        request_rules(input),
    )
    .with_filter_ipv6(false)
}

pub(super) fn server(value: u64) -> crate::robot::RobotServerNumber {
    crate::robot::RobotServerNumber::new(value).unwrap_or_else(|_| unreachable!("server failed"))
}

pub(super) fn template_id() -> RobotFirewallTemplateId {
    RobotFirewallTemplateId::new(17).unwrap_or_else(|_| unreachable!("template ID failed"))
}

pub(super) fn cidr(value: &'static str) -> RobotFirewallCidr<'static> {
    RobotFirewallCidr::new(value).unwrap_or_else(|_| unreachable!("CIDR failed"))
}

pub(super) fn port(value: &'static str) -> RobotFirewallPortRange<'static> {
    RobotFirewallPortRange::new(value).unwrap_or_else(|_| unreachable!("port failed"))
}

pub(super) fn flags(value: &'static str) -> RobotFirewallTcpFlags<'static> {
    RobotFirewallTcpFlags::new(value).unwrap_or_else(|_| unreachable!("flags failed"))
}
