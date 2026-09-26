use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use std::boxed::Box;
use std::io;
use std::net::{SocketAddr, ToSocketAddrs};

use hyper_util::client::legacy::connect::dns::Name;
use tokio::sync::Semaphore;
use tower_service::Service;

// One process-wide budget shared across raw clients and private runtimes.
// The blocking closure owns its permit, even after timeout or runtime shutdown.
static DNS_JOBS: Semaphore = Semaphore::const_new(8);

#[derive(Clone, Debug)]
pub(super) struct BoundedResolver;

impl Service<Name> for BoundedResolver {
    type Response = std::vec::IntoIter<SocketAddr>;
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = io::Result<Self::Response>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        #[cfg(test)]
        let fixture = tests::fixture();
        Box::pin(resolve(&DNS_JOBS, move || {
            #[cfg(test)]
            if let Some(fixture) = fixture {
                return fixture();
            }
            (name.as_str(), 0).to_socket_addrs()
        }))
    }
}

async fn resolve<T: Send + 'static>(
    budget: &'static Semaphore,
    job: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> io::Result<T> {
    let permit = budget
        .try_acquire()
        .map_err(|_| io::Error::new(io::ErrorKind::WouldBlock, "DNS job limit reached"))?;
    tokio::task::spawn_blocking(move || {
        let _permit = permit;
        job()
    })
    .await
    .map_err(|_| io::Error::other("DNS job failed"))?
}

#[cfg(test)]
mod tests;
