use alloc::format;

use cloud_sdk::Method;
use cloud_sdk::operation::{
    CostIntent, OperationImpact, PreparationStorageGuard, RequestSemantics, RetryEligibility,
};
use cloud_sdk::transport::StatusCode;

use super::*;

pub(super) const STANDARD_CREATED: &[u8] = br#"{"transaction":{"id":"B-order","date":"2026-08-16T12:00:00Z","status":"in process","server_number":null,"server_ip":null,"authorized_key":[],"host_key":[],"comment":null,"product":{"id":"EX40","name":"EX40","description":[],"traffic":"30 TB","dist":"Rescue system","@deprecated arch":"64","lang":"en","location":"FSN1"},"addons":["primary_ipv4"]}}"#;
const MARKET_CREATED: &[u8] = br#"{"transaction":{"id":"B-market","date":"2026-08-16T12:00:00Z","status":"in process","server_number":null,"server_ip":null,"authorized_key":[],"host_key":[],"comment":null,"product":{"id":282323,"name":"SB109","description":[],"traffic":"20 TB","dist":"Rescue system","@deprecated arch":"64","lang":"en","cpu":"Intel Core i7 980x","cpu_benchmark":8944,"memory_size":24,"hdd_size":120,"hdd_text":"SSD","hdd_count":2,"datacenter":"FSN1-DC4","network_speed":"1 Gbit/s","fixed_price":false,"next_reduce":-10800,"next_reduce_date":"2028-05-01 12:22:00"}}}"#;
const ADDON_CREATED: &[u8] = br#"{"transaction":{"id":"B-addon","date":"2026-08-16T12:00:00Z","status":"in process","server_number":321,"product":{"id":"additional_ipv4","name":"Additional IP address","price":{"location":"NBG1","price":{"net":"0.8403","gross":"1.0000","hourly_net":"0.0014","hourly_gross":"0.0017"},"price_setup":{"net":"19.0000","gross":"22.6100"}}},"resources":[]}}"#;

#[test]
fn prepares_all_billable_forms_with_fail_closed_metadata() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("standard cost failed"));
        assert_prepared(
            &request,
            "/order/server/transaction",
            b"product_id=EX40&dist=Rescue+system&lang=en&location=FSN1&addon%5B%5D=primary_ipv4",
            "robot_create_server_transaction",
        );
    });
    with_market_plan(|plan| {
        let request = RobotMarketOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!("market cost failed"));
        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = request
            .prepare_bound(&mut guard)
            .unwrap_or_else(|_| unreachable!("market preparation failed"));
        assert_contract(
            &prepared.inner,
            "/order/server_market/transaction",
            b"product_id=282323&dist=Rescue+system&lang=en",
            "robot_create_server_market_transaction",
        );
    });
    with_addon_plan(|plan| {
        let request = RobotAddonOrderCreateRequest::new(plan, 300_000)
            .unwrap_or_else(|_| unreachable!("addon cost failed"));
        let mut target = [0_u8; 128];
        let mut body = [0_u8; 256];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut body);
        let prepared = request
            .prepare_bound(&mut guard)
            .unwrap_or_else(|_| unreachable!("addon preparation failed"));
        assert_contract(
            &prepared.inner,
            "/order/server_addon/transaction",
            b"server_number=321&product_id=additional_ipv4",
            "robot_create_server_addon_transaction",
        );
    });
}

#[test]
fn spending_ceiling_fails_before_preparation_and_diagnostics_are_redacted() {
    with_standard_plan(|plan| {
        assert_eq!(
            RobotStandardOrderCreateRequest::new(plan, 1_020_189).err(),
            Some(RobotOrderCostError::SpendingCeilingExceeded)
        );
        let request = RobotStandardOrderCreateRequest::new(plan, 1_020_190)
            .unwrap_or_else(|_| unreachable!("exact ceiling rejected"));
        assert_eq!(
            format!("{request:?}"),
            "RobotStandardOrderCreateRequest([redacted])"
        );
        assert_eq!(
            format!(
                "{:?}",
                RobotOrderAccount::new(b"account-a").unwrap_or_else(|_| unreachable!())
            ),
            "RobotOrderAccount([redacted])"
        );
    });
}

#[test]
fn checked_created_responses_remain_bound_to_exact_intent() {
    with_standard_plan(|plan| {
        let request = RobotStandardOrderCreateRequest::new(plan, 1_100_000)
            .unwrap_or_else(|_| unreachable!());
        let mut target = [0_u8; 128];
        let mut request_body = [0_u8; 256];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = prepared_standard(&request, &mut guard);
        let mut body = [0_u8; STANDARD_CREATED.len()];
        let mut headers = [0_u8; 128];
        let response = json_response(
            &mut body,
            &mut headers,
            StatusCode::CREATED,
            STANDARD_CREATED,
        );
        assert!(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!())
                .decode_response()
                .is_ok()
        );
    });
    with_market_plan(|plan| {
        let request =
            RobotMarketOrderCreateRequest::new(plan, 1_100_000).unwrap_or_else(|_| unreachable!());
        let mut target = [0_u8; 128];
        let mut request_body = [0_u8; 128];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = request
            .prepare_bound(&mut guard)
            .unwrap_or_else(|_| unreachable!());
        let mut body = [0_u8; MARKET_CREATED.len()];
        let mut headers = [0_u8; 128];
        let response = json_response(&mut body, &mut headers, StatusCode::CREATED, MARKET_CREATED);
        assert!(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!())
                .decode_response()
                .is_ok()
        );
    });
    with_addon_plan(|plan| {
        let request =
            RobotAddonOrderCreateRequest::new(plan, 300_000).unwrap_or_else(|_| unreachable!());
        let mut target = [0_u8; 128];
        let mut request_body = [0_u8; 128];
        let mut guard = PreparationStorageGuard::new(&mut target, &mut request_body);
        let prepared = request
            .prepare_bound(&mut guard)
            .unwrap_or_else(|_| unreachable!());
        let mut body = [0_u8; ADDON_CREATED.len()];
        let mut headers = [0_u8; 128];
        let response = json_response(&mut body, &mut headers, StatusCode::CREATED, ADDON_CREATED);
        assert!(
            prepared
                .validate_response(response)
                .unwrap_or_else(|_| unreachable!())
                .decode_response()
                .is_ok()
        );
    });
}

fn assert_prepared(
    request: &RobotStandardOrderCreateRequest<'_>,
    target_text: &str,
    body_text: &[u8],
    id: &str,
) {
    let mut target = [0_u8; 128];
    let mut body = [0_u8; 256];
    let mut guard = PreparationStorageGuard::new(&mut target, &mut body);
    let prepared = request
        .prepare_bound(&mut guard)
        .unwrap_or_else(|_| unreachable!("standard preparation failed"));
    assert_contract(&prepared.inner, target_text, body_text, id);
}

fn assert_contract(
    prepared: &cloud_sdk::operation::PreparedRequest<'_>,
    target: &str,
    body: &[u8],
    id: &str,
) {
    assert_eq!(prepared.transport_request().method(), Method::Post);
    assert_eq!(prepared.transport_request().target().as_str(), target);
    assert_eq!(prepared.transport_request().body(), body);
    assert_eq!(
        prepared.operation_id().map(|value| value.as_str()),
        Some(id)
    );
    assert_eq!(prepared.metadata().impact(), OperationImpact::Mutation);
    assert_eq!(
        prepared.metadata().semantics(),
        RequestSemantics::NonIdempotent
    );
    assert_eq!(
        prepared.metadata().retry_eligibility(),
        RetryEligibility::Never
    );
    assert_eq!(prepared.metadata().cost_intent(), CostIntent::MayIncurCost);
    assert_eq!(
        prepared.response_policy().success_statuses(),
        &[StatusCode::CREATED]
    );
}
