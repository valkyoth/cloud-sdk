use cloud_sdk::operation::{PermitTimestamp, PreparationStorage, SharedPermitState};
use cloud_sdk::retry::{DigestAlgorithm, FingerprintHasher};
use cloud_sdk::transport::{EndpointIdentity, StatusCode};
use cloud_sdk_hetzner::association::operations::UpdateStorageBox;
use cloud_sdk_hetzner::association::{
    AssociatedMutationPermit, AssociatedOperation, AssociatedSharedMutationPermit,
    HetznerOperation, MutationPermit as MutationPermitMarker, Prepared,
    build_associated_canonical_plan, build_associated_plan_digest,
};
use cloud_sdk_hetzner::serde::{
    HetznerDecodeError, ResponseModelError, decode_associated_checked_response,
};
use cloud_sdk_hetzner::storage::storage_boxes::{StorageBoxId, StorageBoxUpdateRequest};
use cloud_sdk_testkit::MockTransport;

use super::{ACTION, FixedClock, STORAGE_BOXES, associated_plan, endpoint, exchange};

struct Sha256Hasher;

impl FingerprintHasher for Sha256Hasher {
    type Error = core::convert::Infallible;

    fn algorithm(&self) -> DigestAlgorithm {
        DigestAlgorithm::Sha256
    }

    fn digest(&self, input: &[u8], output: &mut [u8]) -> Result<usize, Self::Error> {
        use sha2::Digest;

        let digest = sha2::Sha256::digest(input);
        let Some(target) = output.get_mut(..digest.len()) else {
            return Ok(0);
        };
        target.copy_from_slice(&digest);
        Ok(digest.len())
    }
}

#[test]
fn typed_mutation_permit_rejects_a_mismatched_storage_box_response() {
    let storage = endpoint("api.hetzner.com");
    let Some(storage_box) = StorageBoxId::new(42) else {
        unreachable!("Storage Box fixture ID failed")
    };
    let request = StorageBoxUpdateRequest::new(storage_box);
    let operation =
        AssociatedOperation::<UpdateStorageBox, _, _, _>::json(request.endpoint(), request);
    let Ok(operation) = operation else {
        unreachable!("Storage Box update association failed")
    };
    let mut target = [0_u8; 256];
    let mut request_body = [0_u8; 256];
    let prepared = operation.prepare_typed(PreparationStorage::new(&mut target, &mut request_body));
    let Ok(prepared) = prepared else {
        unreachable!("Storage Box update preparation failed")
    };
    let untyped = prepared.as_untyped();

    let value = serde_json::from_slice::<serde_json::Value>(STORAGE_BOXES);
    let Ok(mut value) = value else {
        unreachable!("Storage Box list fixture failed")
    };
    let Some(resources) = value
        .get_mut("storage_boxes")
        .and_then(serde_json::Value::as_array_mut)
    else {
        unreachable!("Storage Box list fixture lost its resources")
    };
    let Some(resource) = resources.first_mut() else {
        unreachable!("Storage Box list fixture became empty")
    };
    let mut resource = resource.take();
    let Some(fields) = resource.as_object_mut() else {
        unreachable!("Storage Box resource fixture is not an object")
    };
    fields.insert("id".into(), serde_json::json!(43));
    let body = serde_json::to_vec(&serde_json::json!({"storage_box": resource}));
    let Ok(body) = body else {
        unreachable!("Storage Box singleton fixture failed")
    };

    let mut scratch = [0_u8; 4_096];
    let fingerprint =
        build_associated_canonical_plan(associated_plan(prepared, storage), &mut scratch);
    let Ok(fingerprint) = fingerprint else {
        unreachable!("Storage Box update fingerprint failed")
    };
    let permit =
        AssociatedMutationPermit::new(fingerprint.subject(), PermitTimestamp::from_seconds(100));
    let Ok(mut permit) = permit else {
        unreachable!("Storage Box update permit failed")
    };
    let attempt = permit.begin(PermitTimestamp::from_seconds(101));
    let Ok(attempt) = attempt else {
        unreachable!("Storage Box update attempt failed")
    };
    let exchanges = [exchange(untyped, StatusCode::OK, &body, true)];
    let mock = MockTransport::new(&exchanges).with_endpoint(storage);
    let mut response_body = [0_u8; 8_192];
    let mut response_headers = [0_u8; 8_192];
    let response = attempt.execute_blocking(
        &FixedClock,
        &mock,
        &mut response_body,
        &mut response_headers,
    );
    let Ok(response) = response else {
        unreachable!("Storage Box update transport failed")
    };
    assert!(matches!(
        decode_associated_checked_response(response),
        Err(HetznerDecodeError::Model(
            ResponseModelError::ResponseIdentityMismatch,
        )),
    ));
}

pub(super) fn execute_shared_digest_mutation<O>(
    prepared: Prepared<'_, O>,
    endpoint: EndpointIdentity<'static>,
) where
    O: Copy + HetznerOperation<Permit = MutationPermitMarker>,
{
    let untyped = prepared.as_untyped();
    let mut scratch = [0_u8; 4_096];
    let mut digest = [0_u8; 32];
    let fingerprint = build_associated_plan_digest(
        associated_plan(prepared, endpoint),
        &mut scratch,
        &mut digest,
        &Sha256Hasher,
    );
    let Ok(fingerprint) = fingerprint else {
        unreachable!("shared mutation digest failed")
    };
    let mut state = SharedPermitState::new();
    let permit = AssociatedSharedMutationPermit::new(
        &mut state,
        fingerprint.subject(),
        PermitTimestamp::from_seconds(100),
    );
    let Ok(permit) = permit else {
        unreachable!("shared typed mutation permit failed")
    };
    let clone = permit.clone();
    let attempt = clone.begin(PermitTimestamp::from_seconds(101));
    let Ok(attempt) = attempt else {
        unreachable!("shared typed mutation attempt failed")
    };
    let exchanges = [exchange(untyped, StatusCode::CREATED, ACTION, true)];
    let mock = MockTransport::new(&exchanges).with_endpoint(endpoint);
    let mut body = [0_u8; 512];
    let mut headers = [0_u8; 8_192];
    let response = attempt.execute_blocking(&FixedClock, &mock, &mut body, &mut headers);
    let Ok(response) = response else {
        unreachable!("shared typed mutation execution failed")
    };
    assert!(decode_associated_checked_response(response).is_ok());
}
