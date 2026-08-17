use super::{
    ClientExecutionError, ClientKernel, ClientOperation, ClientResponse, ClientResult,
    ClientWorkspaceLease,
};
use crate::authentication::{
    AsyncAuthenticatedTransport, BlockingAuthenticatedTransport, LocalAsyncAuthenticatedTransport,
};
use crate::diagnostics::{
    DiagnosticContext, DiagnosticErrorCategory, DiagnosticEvent, DiagnosticObserver,
    NoopDiagnosticObserver,
};
use crate::operation::{PreparationStorage, PreparedExecutionError};
use crate::transport::BoundTransport;

const NOOP_OBSERVER: NoopDiagnosticObserver = NoopDiagnosticObserver;

impl<T> ClientKernel<T>
where
    T: BlockingAuthenticatedTransport + BoundTransport,
{
    /// Prepares, authenticates, sends once, and checked-decodes synchronously.
    pub fn execute_blocking<O, const N: usize>(
        &self,
        operation: &O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> ClientResult<O::Output, O::Error, T::Error, O::DecodeError>
    where
        O: ClientOperation,
    {
        self.execute_blocking_observed(operation, lease, &NOOP_OBSERVER)
    }

    /// Executes synchronously while emitting opt-in payload-free lifecycle events.
    pub fn execute_blocking_observed<O, V, const N: usize>(
        &self,
        operation: &O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
        observer: &V,
    ) -> ClientResult<O::Output, O::Error, T::Error, O::DecodeError>
    where
        O: ClientOperation,
        V: DiagnosticObserver + ?Sized,
    {
        let mut parts = lease.parts_mut();
        parts.clear();
        notify(observer, DiagnosticEvent::PreparationStarted);
        let prepared =
            match operation.prepare(PreparationStorage::new(parts.target, parts.request_body)) {
                Ok(prepared) => prepared,
                Err(error) => {
                    notify(observer, preparation_failed());
                    return Err(ClientExecutionError::Preparation(error));
                }
            };
        let context = DiagnosticContext::from_prepared(&prepared);
        notify(observer, DiagnosticEvent::RequestPrepared { context });
        notify(observer, DiagnosticEvent::DispatchStarted { context });
        let response = match prepared.send_blocking(
            &self.transport,
            parts.response_body,
            parts.response_headers,
        ) {
            Ok(response) => response,
            Err(error) => {
                notify(observer, execution_failed(context, &error));
                return Err(ClientExecutionError::Execution(error));
            }
        };
        decode_observed(operation, prepared, response, context, observer)
    }
}

impl<T> ClientKernel<T>
where
    T: AsyncAuthenticatedTransport + BoundTransport + Sync,
{
    /// Prepares, authenticates, sends once, and checked-decodes with a Send transport.
    // The explicit opaque return makes the cross-thread execution guarantee
    // part of the public API instead of relying on async-future inference.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async<O, const N: usize>(
        &self,
        operation: &O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> impl core::future::Future<
        Output = ClientResult<O::Output, O::Error, T::Error, O::DecodeError>,
    > + Send
    where
        O: ClientOperation + Sync,
        O::Output: Send,
        O::Error: Send,
        O::DecodeError: Send,
        T::Error: Send,
    {
        self.execute_async_observed(operation, lease, &NOOP_OBSERVER)
    }

    /// Executes with a Send transport and opt-in payload-free lifecycle events.
    #[allow(clippy::manual_async_fn)]
    pub fn execute_async_observed<O, V, const N: usize>(
        &self,
        operation: &O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
        observer: &V,
    ) -> impl core::future::Future<
        Output = ClientResult<O::Output, O::Error, T::Error, O::DecodeError>,
    > + Send
    where
        O: ClientOperation + Sync,
        O::Output: Send,
        O::Error: Send,
        O::DecodeError: Send,
        T::Error: Send,
        V: DiagnosticObserver + Sync + ?Sized,
    {
        async move {
            let mut parts = lease.parts_mut();
            parts.clear();
            notify(observer, DiagnosticEvent::PreparationStarted);
            let prepared = match operation
                .prepare(PreparationStorage::new(parts.target, parts.request_body))
            {
                Ok(prepared) => prepared,
                Err(error) => {
                    notify(observer, preparation_failed());
                    return Err(ClientExecutionError::Preparation(error));
                }
            };
            let context = DiagnosticContext::from_prepared(&prepared);
            notify(observer, DiagnosticEvent::RequestPrepared { context });
            notify(observer, DiagnosticEvent::DispatchStarted { context });
            let response = match prepared
                .send_async(&self.transport, parts.response_body, parts.response_headers)
                .await
            {
                Ok(response) => response,
                Err(error) => {
                    notify(observer, execution_failed(context, &error));
                    return Err(ClientExecutionError::Execution(error));
                }
            };
            decode_observed(operation, prepared, response, context, observer)
        }
    }
}

impl<T> ClientKernel<T>
where
    T: LocalAsyncAuthenticatedTransport + BoundTransport,
{
    /// Prepares, authenticates, sends once, and checked-decodes locally.
    pub async fn execute_local_async<O, const N: usize>(
        &self,
        operation: &O,
        lease: ClientWorkspaceLease<'_, '_, N>,
    ) -> ClientResult<O::Output, O::Error, T::Error, O::DecodeError>
    where
        O: ClientOperation,
    {
        self.execute_local_async_observed(operation, lease, &NOOP_OBSERVER)
            .await
    }

    /// Executes locally while emitting opt-in payload-free lifecycle events.
    pub async fn execute_local_async_observed<O, V, const N: usize>(
        &self,
        operation: &O,
        mut lease: ClientWorkspaceLease<'_, '_, N>,
        observer: &V,
    ) -> ClientResult<O::Output, O::Error, T::Error, O::DecodeError>
    where
        O: ClientOperation,
        V: DiagnosticObserver + ?Sized,
    {
        let mut parts = lease.parts_mut();
        parts.clear();
        notify(observer, DiagnosticEvent::PreparationStarted);
        let prepared =
            match operation.prepare(PreparationStorage::new(parts.target, parts.request_body)) {
                Ok(prepared) => prepared,
                Err(error) => {
                    notify(observer, preparation_failed());
                    return Err(ClientExecutionError::Preparation(error));
                }
            };
        let context = DiagnosticContext::from_prepared(&prepared);
        notify(observer, DiagnosticEvent::RequestPrepared { context });
        notify(observer, DiagnosticEvent::DispatchStarted { context });
        let response = match prepared
            .send_local_async(&self.transport, parts.response_body, parts.response_headers)
            .await
        {
            Ok(response) => response,
            Err(error) => {
                notify(observer, execution_failed(context, &error));
                return Err(ClientExecutionError::Execution(error));
            }
        };
        decode_observed(operation, prepared, response, context, observer)
    }
}

#[allow(clippy::large_types_passed_by_value)]
fn decode_observed<O, V, T>(
    operation: &O,
    prepared: crate::operation::PreparedRequest<'_>,
    response: crate::transport::ResponseBuffer<'_>,
    context: DiagnosticContext,
    observer: &V,
) -> ClientResult<O::Output, O::Error, T, O::DecodeError>
where
    O: ClientOperation,
    V: DiagnosticObserver + ?Sized,
{
    let response = ClientResponse::new(prepared, response);
    let diagnostic = response.diagnostic_response().ok();
    if let Some(response) = diagnostic {
        notify(
            observer,
            DiagnosticEvent::ResponseReceived { context, response },
        );
    }
    match operation.decode_response(response) {
        Ok(output) => {
            notify(
                observer,
                DiagnosticEvent::Completed {
                    context,
                    response: diagnostic,
                },
            );
            Ok(output)
        }
        Err(error) => {
            notify(
                observer,
                DiagnosticEvent::DecodeFailed {
                    context,
                    response: diagnostic,
                    error: DiagnosticErrorCategory::Decode,
                },
            );
            Err(ClientExecutionError::Decode(error))
        }
    }
}

fn notify<O>(observer: &O, event: DiagnosticEvent)
where
    O: DiagnosticObserver + ?Sized,
{
    crate::diagnostics::notify(observer, event);
}

const fn preparation_failed() -> DiagnosticEvent {
    DiagnosticEvent::PreparationFailed {
        error: DiagnosticErrorCategory::Preparation,
    }
}

fn execution_failed<E>(
    context: DiagnosticContext,
    error: &PreparedExecutionError<E>,
) -> DiagnosticEvent {
    let error = match error {
        PreparedExecutionError::AuthorizationRequired
        | PreparedExecutionError::AuthorizationInvalid(_) => DiagnosticErrorCategory::Authorization,
        PreparedExecutionError::EndpointIdentity(_) | PreparedExecutionError::EndpointMismatch => {
            DiagnosticErrorCategory::Endpoint
        }
        PreparedExecutionError::Transport(_) => DiagnosticErrorCategory::Transport,
        PreparedExecutionError::ResponseWriter(_) => DiagnosticErrorCategory::ResponseTransaction,
        PreparedExecutionError::UnexpectedStatus(_) => DiagnosticErrorCategory::ResponsePolicy,
        PreparedExecutionError::ResponsePolicy(_) => DiagnosticErrorCategory::ResponsePolicy,
    };
    DiagnosticEvent::ExecutionFailed { context, error }
}
