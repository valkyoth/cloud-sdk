use core::cell::Cell;
use core::future::{Future, pending};
use core::task::{Context, Poll, Waker};

use crate::Method;

use super::{
    ASYNC_CANCELLATION_DELIVERY_PHASE, AsyncResponseStaging, AsyncTransport, DeliveryPhase,
    HeaderSensitivity, LocalAsyncTransport, RequestTarget, ResponseBuffer, ResponseCompletion,
    ResponseMetadata, StatusCode, TransportRequest, drive_async, drive_local,
};

struct PendingLocalTransport {
    polls: Cell<u8>,
}

impl LocalAsyncTransport for PendingLocalTransport {
    type Error = ();

    async fn send_local<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        self.polls.set(self.polls.get().saturating_add(1));
        response.body_mut().map_err(|_| ())?.fill(0x5a);
        response
            .headers_mut()
            .map_err(|_| ())?
            .try_push("x-secret", b"partial", HeaderSensitivity::Sensitive)
            .map_err(|_| ())?;
        pending::<()>().await;
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            2,
            ResponseMetadata::EMPTY,
        ))
    }
}

#[test]
fn local_async_cancellation_clears_partial_state_and_is_possibly_sent() {
    assert_eq!(
        ASYNC_CANCELLATION_DELIVERY_PHASE,
        DeliveryPhase::PossiblySent
    );
    let Ok(target) = RequestTarget::new("/local-cancel") else {
        unreachable!("security fixture construction failed");
    };
    let transport = PendingLocalTransport {
        polls: Cell::new(0),
    };
    let mut body = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
    {
        let future = drive_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            response.writer(),
        );
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    assert_eq!(transport.polls.get(), 1);
    let Ok(mut next) = response.writer().begin_attempt() else {
        unreachable!("security fixture construction failed");
    };
    assert!(
        next.body_mut()
            .is_ok_and(|bytes| bytes.iter().all(|byte| *byte == 0))
    );
    assert!(next.headers().is_empty());
}

struct ActiveGuard<'a> {
    active: &'a Cell<u8>,
}

impl Drop for ActiveGuard<'_> {
    fn drop(&mut self) {
        self.active.set(self.active.get().saturating_sub(1));
    }
}

struct CooperativeLocalTransport {
    active: Cell<u8>,
    maximum_active: Cell<u8>,
}

impl LocalAsyncTransport for CooperativeLocalTransport {
    type Error = ();

    async fn send_local<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        let active = self.active.get().checked_add(1).ok_or(())?;
        self.active.set(active);
        self.maximum_active
            .set(core::cmp::max(self.maximum_active.get(), active));
        let _guard = ActiveGuard {
            active: &self.active,
        };
        YieldOnce { yielded: false }.await;
        stage_ok(&mut response)
    }
}

struct YieldOnce {
    yielded: bool,
}

impl Future for YieldOnce {
    type Output = ();

    fn poll(mut self: core::pin::Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        if self.yielded {
            Poll::Ready(())
        } else {
            self.yielded = true;
            context.waker().wake_by_ref();
            Poll::Pending
        }
    }
}

#[test]
fn local_async_allows_bounded_same_thread_concurrency() {
    let Ok(target) = RequestTarget::new("/local-concurrent") else {
        unreachable!("security fixture construction failed");
    };
    let transport = CooperativeLocalTransport {
        active: Cell::new(0),
        maximum_active: Cell::new(0),
    };
    let mut first_body = [0_u8; 2];
    let mut first_headers = [0_u8; 64];
    let mut first_response = ResponseBuffer::new(&mut first_body, 2, &mut first_headers);
    let mut second_body = [0_u8; 2];
    let mut second_headers = [0_u8; 64];
    let mut second_response = ResponseBuffer::new(&mut second_body, 2, &mut second_headers);
    {
        let first = drive_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            first_response.writer(),
        );
        let second = drive_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            second_response.writer(),
        );
        let mut first = core::pin::pin!(first);
        let mut second = core::pin::pin!(second);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(first.as_mut(), &mut context),
            Poll::Pending
        ));
        assert!(matches!(
            Future::poll(second.as_mut(), &mut context),
            Poll::Pending
        ));
        assert_eq!(transport.maximum_active.get(), 2);
        assert!(matches!(
            Future::poll(first.as_mut(), &mut context),
            Poll::Ready(Ok(()))
        ));
        assert!(matches!(
            Future::poll(second.as_mut(), &mut context),
            Poll::Ready(Ok(()))
        ));
    }
    assert_eq!(transport.active.get(), 0);
    assert!(
        first_response
            .with_response(|value| value.body() == b"ok")
            .is_ok_and(core::convert::identity)
    );
    assert!(
        second_response
            .with_response(|value| value.body() == b"ok")
            .is_ok_and(core::convert::identity)
    );
}

struct SendTransport;

impl AsyncTransport for SendTransport {
    type Error = ();

    async fn send<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        stage_ok(&mut response)
    }
}

#[test]
fn send_async_transports_automatically_satisfy_the_local_contract() {
    let Ok(target) = RequestTarget::new("/send-as-local") else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0_u8; 2];
    let mut headers = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut body, 2, &mut headers);
    let future = drive_local(
        &SendTransport,
        TransportRequest::new(Method::Get, target),
        response.writer(),
    );
    let mut future = core::pin::pin!(future);
    let mut context = Context::from_waker(Waker::noop());
    assert!(matches!(
        Future::poll(future.as_mut(), &mut context),
        Poll::Ready(Ok(()))
    ));
}

struct StagedThenPendingSendTransport;

impl AsyncTransport for StagedThenPendingSendTransport {
    type Error = ();

    async fn send<'transport, 'request, 'writer, 'buffer>(
        &'transport self,
        _request: TransportRequest<'request>,
        mut response: AsyncResponseStaging<'writer, 'buffer>,
    ) -> Result<ResponseCompletion, Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
        'buffer: 'writer,
    {
        response.body_mut().map_err(|_| ())?.fill(0x5a);
        response
            .headers_mut()
            .map_err(|_| ())?
            .try_push("x-secret", b"committed", HeaderSensitivity::Sensitive)
            .map_err(|_| ())?;
        pending::<()>().await;
        Ok(ResponseCompletion::new(
            StatusCode::OK,
            2,
            ResponseMetadata::EMPTY,
        ))
    }
}

#[test]
fn send_driver_rolls_back_staging_when_cancelled() {
    let Ok(target) = RequestTarget::new("/stage-then-pending") else {
        unreachable!("security fixture construction failed");
    };
    let mut body = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
    {
        let future = drive_async(
            &StagedThenPendingSendTransport,
            TransportRequest::new(Method::Get, target),
            response.writer(),
        );
        let mut future = core::pin::pin!(future);
        let mut context = Context::from_waker(Waker::noop());
        assert!(matches!(
            Future::poll(future.as_mut(), &mut context),
            Poll::Pending
        ));
    }
    assert!(!response.writer().is_committed());
    drop(response);
    assert_eq!(body, [0_u8; 16]);
    assert_eq!(headers, [0_u8; 256]);
}

fn stage_ok(response: &mut AsyncResponseStaging<'_, '_>) -> Result<ResponseCompletion, ()> {
    response
        .body_mut()
        .map_err(|_| ())?
        .get_mut(..2)
        .ok_or(())?
        .copy_from_slice(b"ok");
    Ok(ResponseCompletion::new(
        StatusCode::OK,
        2,
        ResponseMetadata::EMPTY,
    ))
}
