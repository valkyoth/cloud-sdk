use cloud_sdk::Method;
use cloud_sdk::transport::{
    AsyncRawHttpExecutor, BlockingRawHttpExecutor, DeliveryPhase, LocalAsyncRawHttpExecutor,
    MediaType, RawResponsePolicy, RequestTarget, ResponseBuffer, ResponseMediaPolicy,
    TransportRequest,
};
use core::future::Future;
use core::task::{Context, Poll, Waker};

use crate::{RawFault, RawFaultExecutor};

fn policy() -> Option<RawResponsePolicy<'static>> {
    RawResponsePolicy::new(
        1,
        1,
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        ResponseMediaPolicy::Optional(&[MediaType::JSON]),
        &[],
        0,
    )
    .ok()
}

#[test]
fn preserves_every_conservative_delivery_phase() {
    let Ok(target) = RequestTarget::new("/fault") else {
        return;
    };
    let Some(policy) = policy() else { return };
    for (fault, expected) in [
        (RawFault::NotSent, DeliveryPhase::NotSent),
        (RawFault::PossiblySent, DeliveryPhase::PossiblySent),
        (RawFault::ResponseStarted, DeliveryPhase::ResponseStarted),
        (RawFault::Unknown, DeliveryPhase::PossiblySent),
    ] {
        let executor = RawFaultExecutor::new(fault);
        let mut body = [0xa5_u8; 1];
        let mut headers = [0xa5_u8; 32];
        let mut response = ResponseBuffer::new(&mut body, 1, &mut headers);
        let failure = BlockingRawHttpExecutor::execute(
            &executor,
            TransportRequest::new(Method::Get, target),
            policy,
            response.writer(),
        );
        assert!(matches!(failure, Err(error) if error.phase() == expected));
    }
}

#[test]
fn async_unknown_fault_is_immediately_possibly_sent() {
    let Ok(target) = RequestTarget::new("/fault") else {
        return;
    };
    let Some(policy) = policy() else { return };
    let executor = RawFaultExecutor::new(RawFault::Unknown);
    let mut body = [0xa5_u8; 1];
    let mut headers = [0xa5_u8; 32];
    let mut response = ResponseBuffer::new(&mut body, 1, &mut headers);
    let future = AsyncRawHttpExecutor::execute(
        &executor,
        TransportRequest::new(Method::Get, target),
        policy,
        response.writer(),
    );
    let mut future = core::pin::pin!(future);
    let waker = Waker::noop();
    let mut context = Context::from_waker(waker);
    assert!(matches!(
        Future::poll(future.as_mut(), &mut context),
        Poll::Ready(Err(error)) if error.phase() == DeliveryPhase::PossiblySent
    ));
}

#[test]
fn send_raw_executor_automatically_satisfies_local_async() {
    let Ok(target) = RequestTarget::new("/fault") else {
        return;
    };
    let Some(policy) = policy() else { return };
    let executor = RawFaultExecutor::new(RawFault::NotSent);
    let mut body = [0xa5_u8; 1];
    let mut headers = [0xa5_u8; 32];
    let mut response = ResponseBuffer::new(&mut body, 1, &mut headers);
    let future = LocalAsyncRawHttpExecutor::execute_local(
        &executor,
        TransportRequest::new(Method::Get, target),
        policy,
        response.writer(),
    );
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        Future::poll(future.as_mut(), &mut context),
        Poll::Ready(Err(error)) if error.phase() == DeliveryPhase::NotSent
    ));
}
