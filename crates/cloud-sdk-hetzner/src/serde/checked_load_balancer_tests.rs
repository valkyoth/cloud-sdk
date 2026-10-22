use alloc::{format, string::String};
use cloud_sdk::operation::PreparationStorage;
use cloud_sdk::transport::StatusCode;

use super::checked_test_support::{
    assert_decode_error, decode_typed_checked_response, decode_typed_response, response,
};
use super::{HetznerDecodeError, HetznerSuccess, ResponseModelError};
use crate::association::{AssociatedOperation, operations::DeleteLoadBalancer};
use crate::cloud::load_balancers::{LoadBalancerEndpoint, LoadBalancerId};

pub(super) fn deletion_body(resources: &str) -> String {
    format!(
        r#"{{"action":{{"id":42,"command":"delete_load_balancer","status":"running","progress":10,"started":"2026-07-16T00:00:00Z","finished":null,"resources":{resources},"error":null}}}}"#
    )
}

#[test]
fn typed_deletion_binds_action_resources_in_both_decode_paths() {
    let id = LoadBalancerId::new(7).unwrap_or_else(|| unreachable!("valid load balancer fixture"));
    for checked in [false, true] {
        for (resources, matches) in [
            (r#"[{"id":7,"type":"load_balancer"}]"#, true),
            (
                r#"[{"id":8,"type":"server"},{"id":7,"type":"load_balancer"}]"#,
                true,
            ),
            (r#"[{"id":7,"type":"server"}]"#, false),
            (r#"[{"id":8,"type":"load_balancer"}]"#, false),
            (
                r#"[{"id":7,"type":"server"},{"id":8,"type":"load_balancer"}]"#,
                false,
            ),
            ("[]", false),
        ] {
            let mut target = [0; 256];
            let mut body = [0; 256];
            let prepared = AssociatedOperation::<DeleteLoadBalancer, _>::endpoint(
                LoadBalancerEndpoint::Delete(id),
            )
            .unwrap_or_else(|_| unreachable!("valid deletion association"))
            .prepare_typed(PreparationStorage::new(&mut target, &mut body))
            .unwrap_or_else(|_| unreachable!("valid deletion preparation"));
            let payload = deletion_body(resources);
            let fixture = response(StatusCode::CREATED, payload.as_bytes());
            let decoded = if checked {
                decode_typed_checked_response::<DeleteLoadBalancer>(prepared, fixture)
            } else {
                decode_typed_response::<DeleteLoadBalancer>(prepared, fixture)
            };
            if matches {
                assert!(matches!(
                    decoded.map(|value| value.into_success()),
                    Ok(HetznerSuccess::Action(_))
                ));
            } else {
                assert_decode_error(
                    decoded,
                    HetznerDecodeError::Model(ResponseModelError::ResponseIdentityMismatch),
                );
            }
        }
    }
}
