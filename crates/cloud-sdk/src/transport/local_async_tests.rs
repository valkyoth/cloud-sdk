use core::cell::Cell;
use core::future::{Future, pending};
use core::task::{Context, Poll, Waker};

use crate::Method;

use super::{
    ASYNC_CANCELLATION_DELIVERY_PHASE, AsyncTransport, DeliveryPhase, HeaderSensitivity,
    LocalAsyncTransport, RequestTarget, ResponseBuffer, ResponseMetadata, ResponseWriter,
    StatusCode, TransportRequest,
};

struct PendingLocalTransport {
    polls: Cell<u8>,
}

impl LocalAsyncTransport for PendingLocalTransport {
    type Error = ();

    async fn send_local<'transport, 'request, 'writer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
    {
        self.polls.set(self.polls.get().saturating_add(1));
        let mut attempt = response.begin_attempt().map_err(|_| ())?;
        attempt.body_mut().map_err(|_| ())?.fill(0x5a);
        attempt
            .headers_mut()
            .map_err(|_| ())?
            .try_push("x-secret", b"partial", HeaderSensitivity::Sensitive)
            .map_err(|_| ())?;
        pending::<()>().await;
        Ok(())
    }
}

#[test]
fn local_async_cancellation_clears_partial_state_and_is_possibly_sent() {
    assert_eq!(
        ASYNC_CANCELLATION_DELIVERY_PHASE,
        DeliveryPhase::PossiblySent
    );
    let Ok(target) = RequestTarget::new("/local-cancel") else {
        return;
    };
    let transport = PendingLocalTransport {
        polls: Cell::new(0),
    };
    let mut body = [0xa5_u8; 16];
    let mut headers = [0xa5_u8; 256];
    let mut response = ResponseBuffer::new(&mut body, 16, &mut headers);
    {
        let future = LocalAsyncTransport::send_local(
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
        return;
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

    async fn send_local<'transport, 'request, 'writer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
    {
        let active = self.active.get().checked_add(1).ok_or(())?;
        self.active.set(active);
        self.maximum_active
            .set(core::cmp::max(self.maximum_active.get(), active));
        let _guard = ActiveGuard {
            active: &self.active,
        };
        YieldOnce { yielded: false }.await;
        commit_ok(response)
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
        return;
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
        let first = LocalAsyncTransport::send_local(
            &transport,
            TransportRequest::new(Method::Get, target),
            first_response.writer(),
        );
        let second = LocalAsyncTransport::send_local(
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

    async fn send<'transport, 'request, 'writer>(
        &'transport self,
        _request: TransportRequest<'request>,
        response: &'writer mut ResponseWriter<'_>,
    ) -> Result<(), Self::Error>
    where
        'transport: 'writer,
        'request: 'writer,
    {
        commit_ok(response)
    }
}

#[test]
fn send_async_transports_automatically_satisfy_the_local_contract() {
    let Ok(target) = RequestTarget::new("/send-as-local") else {
        return;
    };
    let mut body = [0_u8; 2];
    let mut headers = [0_u8; 64];
    let mut response = ResponseBuffer::new(&mut body, 2, &mut headers);
    let future = LocalAsyncTransport::send_local(
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

fn commit_ok(response: &mut ResponseWriter<'_>) -> Result<(), ()> {
    let mut attempt = response.begin_attempt().map_err(|_| ())?;
    attempt
        .body_mut()
        .map_err(|_| ())?
        .get_mut(..2)
        .ok_or(())?
        .copy_from_slice(b"ok");
    attempt
        .commit(StatusCode::OK, 2, ResponseMetadata::EMPTY)
        .map_err(|_| ())
}
