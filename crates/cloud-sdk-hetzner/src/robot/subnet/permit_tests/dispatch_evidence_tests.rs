use super::*;
use cloud_sdk::operation::{PermitDisposition, PermitExecutionError, PreparedExecutionError};

struct ExpiredEvidenceClock;

impl PermitClock for ExpiredEvidenceClock {
    fn now(&self) -> PermitTimestamp {
        PermitTimestamp::from_seconds(129)
    }
}

macro_rules! expired_dispatch_case {
    ($name:ident, blocking) => {
        #[test]
        fn $name() {
            let request = delete_request();
            let mut target = [0_u8; 128];
            let mut request_body = [0_u8; 1];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
                .unwrap_or_else(|_| unreachable!("MAC delete preparation failed"));
            let expected = expected_request(prepared.as_untyped());
            let endpoint = endpoint();
            let mut scratch = [0_u8; 4_096];
            let mut digest = [0_u8; 32];
            let fingerprint = build_robot_subnet_plan_digest(
                plan(prepared, endpoint),
                &mut scratch,
                &mut digest,
                &Sha256PlanHasher,
            )
            .unwrap_or_else(|_| unreachable!("MAC delete fingerprint failed"));
            let mut permit = RobotSubnetDestructivePermit::new(
                fingerprint.subject(),
                PermitTimestamp::from_seconds(100),
            )
            .unwrap_or_else(|_| unreachable!("destructive permit failed"));
            let attempt = permit
                .begin(PermitTimestamp::from_seconds(128))
                .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
            let exchanges = [MockExchange::new(expected, json_fixture(MAC_DELETED))];
            let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
            let mut response_body = [0xa5_u8; 256];
            let mut response_headers = [0x5a_u8; 128];

            let result = attempt.execute_blocking(
                &ExpiredEvidenceClock,
                &transport,
                &mut response_body,
                &mut response_headers,
            );

            assert_expired(result);
            assert_eq!(response_body, [0_u8; 256]);
            assert_eq!(response_headers, [0_u8; 128]);
            assert!(!transport.is_complete());
            assert_eq!(permit.state(), PermitState::Spent);
        }
    };
    ($name:ident, send_async) => {
        #[test]
        fn $name() {
            let request = delete_request();
            let mut target = [0_u8; 128];
            let mut request_body = [0_u8; 1];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
                .unwrap_or_else(|_| unreachable!("MAC delete preparation failed"));
            let expected = expected_request(prepared.as_untyped());
            let endpoint = endpoint();
            let mut scratch = [0_u8; 4_096];
            let mut digest = [0_u8; 32];
            let fingerprint = build_robot_subnet_plan_digest(
                plan(prepared, endpoint),
                &mut scratch,
                &mut digest,
                &Sha256PlanHasher,
            )
            .unwrap_or_else(|_| unreachable!("MAC delete fingerprint failed"));
            let mut state = SharedPermitState::new();
            let permit = RobotSubnetSharedDestructivePermit::new(
                &mut state,
                fingerprint.subject(),
                PermitTimestamp::from_seconds(100),
            )
            .unwrap_or_else(|_| unreachable!("shared destructive permit failed"));
            let attempt = permit
                .begin(PermitTimestamp::from_seconds(128))
                .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
            let exchanges = [MockExchange::new(expected, json_fixture(MAC_DELETED))];
            let transport = MockTransport::new(&exchanges).with_endpoint(endpoint);
            let mut response_body = [0xa5_u8; 256];
            let mut response_headers = [0x5a_u8; 128];

            let result = ready(attempt.execute_async(
                &ExpiredEvidenceClock,
                &transport,
                &mut response_body,
                &mut response_headers,
            ));

            assert_expired(result);
            assert_eq!(response_body, [0_u8; 256]);
            assert_eq!(response_headers, [0_u8; 128]);
            assert!(!transport.is_complete());
            assert_eq!(permit.state(), PermitState::Spent);
        }
    };
    ($name:ident, local_async) => {
        #[test]
        fn $name() {
            let request = delete_request();
            let mut target = [0_u8; 128];
            let mut request_body = [0_u8; 1];
            let prepared = request
                .prepare_bound(PreparationStorage::new(&mut target, &mut request_body))
                .unwrap_or_else(|_| unreachable!("MAC delete preparation failed"));
            let expected = expected_request(prepared.as_untyped());
            let endpoint = endpoint();
            let mut scratch = [0_u8; 4_096];
            let mut digest = [0_u8; 32];
            let fingerprint = build_robot_subnet_plan_digest(
                plan(prepared, endpoint),
                &mut scratch,
                &mut digest,
                &Sha256PlanHasher,
            )
            .unwrap_or_else(|_| unreachable!("MAC delete fingerprint failed"));
            let mut state = SharedPermitState::new();
            let permit = RobotSubnetSharedDestructivePermit::new(
                &mut state,
                fingerprint.subject(),
                PermitTimestamp::from_seconds(100),
            )
            .unwrap_or_else(|_| unreachable!("shared destructive permit failed"));
            let attempt = permit
                .begin(PermitTimestamp::from_seconds(128))
                .unwrap_or_else(|_| unreachable!("destructive attempt failed"));
            let exchanges = [MockExchange::new(expected, json_fixture(MAC_DELETED))];
            let transport = LocalMockTransport::new(&exchanges).with_endpoint(endpoint);
            let mut response_body = [0xa5_u8; 256];
            let mut response_headers = [0x5a_u8; 128];

            let result = ready(attempt.execute_local_async(
                &ExpiredEvidenceClock,
                &transport,
                &mut response_body,
                &mut response_headers,
            ));

            assert_expired(result);
            assert_eq!(response_body, [0_u8; 256]);
            assert_eq!(response_headers, [0_u8; 128]);
            assert!(!transport.is_complete());
            assert_eq!(permit.state(), PermitState::Spent);
        }
    };
}

expired_dispatch_case!(expired_evidence_blocks_blocking_dispatch, blocking);
expired_dispatch_case!(expired_evidence_blocks_send_async_dispatch, send_async);
expired_dispatch_case!(expired_evidence_blocks_local_async_dispatch, local_async);

fn assert_expired<E>(
    result: Result<
        CheckedRobotSubnet<'_, '_, RobotSubnetMacDeleteRequest>,
        PermitExecutionError<E>,
    >,
) {
    let error = result.err().unwrap_or_else(|| {
        unreachable!("expired authorization evidence reached transport dispatch")
    });
    assert!(matches!(
        error.execution(),
        PreparedExecutionError::AuthorizationInvalid(ExecutionPermitError::Expired)
    ));
    assert_eq!(error.disposition(), PermitDisposition::Spent);
}
