use core::future::Future;

use crate::shared::{RawHttpError, RawTransportFailure};
use cloud_sdk::transport::TransportFailure;

// Runtime::drop waits for uncancellable system DNS. Always detach shutdown,
// including early errors and unwinding; the resolver separately bounds jobs.
pub(super) struct Runtime {
    inner: Option<tokio::runtime::Runtime>,
}

impl Runtime {
    pub(super) fn new() -> Result<Self, RawTransportFailure> {
        if tokio::runtime::Handle::try_current().is_ok() {
            return Err(TransportFailure::not_sent(
                RawHttpError::BlockingRuntimeContext,
            ));
        }
        let inner = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| TransportFailure::not_sent(RawHttpError::RuntimeInitializationFailed))?;
        Ok(Self { inner: Some(inner) })
    }

    pub(super) fn block_on<F: Future>(&self, future: F) -> Result<F::Output, RawTransportFailure> {
        let runtime = self
            .inner
            .as_ref()
            .ok_or_else(|| TransportFailure::not_sent(RawHttpError::RuntimeInitializationFailed))?;
        Ok(runtime.block_on(future))
    }
}

impl Drop for Runtime {
    fn drop(&mut self) {
        if let Some(runtime) = self.inner.take() {
            runtime.shutdown_background();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::time::Duration;

    #[test]
    fn shutdown_and_unwinding_do_not_wait_for_blocking_work() {
        for unwind in [false, true] {
            let (release, blocked) = mpsc::channel();
            let (started, ready) = mpsc::channel();
            let (finished, completion) = mpsc::channel();
            let worker = std::thread::spawn(move || {
                let result = std::panic::catch_unwind(|| {
                    let runtime = Runtime::new().unwrap_or_else(|e| unreachable!("runtime: {e}"));
                    runtime
                        .block_on(async {
                            drop(tokio::task::spawn_blocking(move || {
                                started
                                    .send(())
                                    .unwrap_or_else(|e| unreachable!("start: {e}"));
                                blocked
                                    .recv_timeout(Duration::from_secs(5))
                                    .unwrap_or_else(|e| unreachable!("release: {e}"));
                            }));
                        })
                        .unwrap_or_else(|e| unreachable!("runtime: {e}"));
                    ready
                        .recv_timeout(Duration::from_secs(2))
                        .unwrap_or_else(|e| unreachable!("start: {e}"));
                    if unwind {
                        unreachable!("fixture unwind");
                    }
                });
                finished
                    .send(result.is_err())
                    .unwrap_or_else(|e| unreachable!("complete: {e}"));
            });
            let result = completion.recv_timeout(Duration::from_secs(2));
            let _ = release.send(());
            assert_eq!(result.ok(), Some(unwind));
            assert!(worker.join().is_ok());
        }
    }
}
