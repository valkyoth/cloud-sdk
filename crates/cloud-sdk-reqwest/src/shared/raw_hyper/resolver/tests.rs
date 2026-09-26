use super::*;
use std::sync::mpsc;
use std::time::Duration;

type Fixture = std::sync::Arc<dyn Fn() -> io::Result<std::vec::IntoIter<SocketAddr>> + Send + Sync>;
std::thread_local! {
    static FIXTURE: std::cell::RefCell<Option<Fixture>> = const { std::cell::RefCell::new(None) };
}
pub(super) fn fixture() -> Option<Fixture> {
    FIXTURE.with(|slot| slot.borrow().clone())
}

#[cfg(feature = "blocking-rustls")]
mod blocking;

#[test]
fn cancelled_dns_keeps_its_slot_until_the_blocking_job_finishes() {
    static BUDGET: Semaphore = Semaphore::const_new(1);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| unreachable!("runtime: {e}"));
    let (started, ready) = mpsc::channel();
    let (release, blocked) = mpsc::channel();
    let task = runtime.spawn(resolve(&BUDGET, move || {
        started
            .send(())
            .unwrap_or_else(|e| unreachable!("start: {e}"));
        blocked
            .recv_timeout(Duration::from_secs(5))
            .unwrap_or_else(|e| unreachable!("release: {e}"));
        Ok(())
    }));
    runtime.block_on(async { tokio::task::yield_now().await });
    ready
        .recv_timeout(Duration::from_secs(2))
        .unwrap_or_else(|e| unreachable!("start: {e}"));
    task.abort();
    runtime.block_on(async {
        assert!(task.await.is_err());
    });
    assert_eq!(BUDGET.available_permits(), 0);
    for _ in 0..32 {
        let result = runtime.block_on(resolve::<()>(&BUDGET, || unreachable!("saturated DNS ran")));
        assert!(result.is_err_and(|e| e.kind() == io::ErrorKind::WouldBlock));
    }
    runtime.shutdown_background();
    assert_eq!(BUDGET.available_permits(), 0);
    release
        .send(())
        .unwrap_or_else(|e| unreachable!("release: {e}"));
    let deadline = std::time::Instant::now() + Duration::from_secs(2);
    while BUDGET.available_permits() == 0 && std::time::Instant::now() < deadline {
        std::thread::yield_now();
    }
    assert_eq!(BUDGET.available_permits(), 1);
}

#[test]
fn dns_error_and_unwinding_release_the_budget() {
    static BUDGET: Semaphore = Semaphore::const_new(1);
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap_or_else(|e| unreachable!("runtime: {e}"));
    assert!(
        runtime
            .block_on(resolve::<()>(&BUDGET, || Err(io::Error::other("fixture"))))
            .is_err()
    );
    assert_eq!(BUDGET.available_permits(), 1);
    assert!(
        runtime
            .block_on(resolve::<()>(&BUDGET, || unreachable!("fixture unwind")))
            .is_err()
    );
    assert_eq!(BUDGET.available_permits(), 1);
    assert_eq!(runtime.block_on(resolve(&BUDGET, || Ok(42))).ok(), Some(42));
}
