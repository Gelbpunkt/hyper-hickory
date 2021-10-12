#![deny(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use std::{
    future::Future,
    net::SocketAddr,
    pin::Pin,
    sync::Arc,
    task::{self, Poll},
};

use hyper::{
    client::{connect::dns::Name, HttpConnector},
    service::Service,
};
use trust_dns_resolver::{
    config::{ResolverConfig, ResolverOpts},
    error::ResolveError,
    lookup_ip::LookupIpIntoIter,
    TokioAsyncResolver, TokioHandle,
};

#[cfg(feature = "native-tls")]
pub use crate::native_tls::HttpsConnector as NativeTlsHttpsConnector;
#[cfg(feature = "__rustls")]
pub use crate::rustls::HttpsConnector as RustlsHttpsConnector;

#[cfg(feature = "native-tls")]
mod native_tls;
#[cfg(feature = "__rustls")]
mod rustls;

#[derive(Clone)]
pub struct TrustDnsResolver {
    resolver: Arc<TokioAsyncResolver>,
}

pub struct SocketAddrs {
    iter: LookupIpIntoIter,
}

impl Iterator for SocketAddrs {
    type Item = SocketAddr;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|ip_addr| SocketAddr::new(ip_addr, 0))
    }
}

impl TrustDnsResolver {
    /// Create a new [`TrustDnsResolver`] with the default config options.
    /// This must be run inside a Tokio runtime context.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn new() -> Self {
        // This unwrap is safe because internally, there is nothing to be unwrapped
        // TokioAsyncResolver::new cannot return Err
        let resolver = Arc::new(
            TokioAsyncResolver::new(
                ResolverConfig::default(),
                ResolverOpts::default(),
                TokioHandle,
            )
            .unwrap(),
        );

        Self { resolver }
    }
}

impl Default for TrustDnsResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Service<Name> for TrustDnsResolver {
    type Response = SocketAddrs;
    type Error = ResolveError;
    #[allow(clippy::type_complexity)]
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _cx: &mut task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, name: Name) -> Self::Future {
        let resolver = self.resolver.clone();

        Box::pin(async move {
            let response = resolver.lookup_ip(name.as_str()).await?;
            let addresses = response.into_iter();

            Ok(SocketAddrs { iter: addresses })
        })
    }
}

pub type TrustDnsHttpConnector = HttpConnector<TrustDnsResolver>;

/// Create a new [`TrustDnsHttpConnector`] that only support HTTP.
#[must_use]
pub fn new_trust_dns_http_connector() -> TrustDnsHttpConnector {
    TrustDnsHttpConnector::new_with_resolver(TrustDnsResolver::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::{Body, Client, Request};

    #[tokio::test]
    async fn test_lookup_works() {
        let connector = new_trust_dns_http_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("http://www.google.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(feature = "rustls-webpki")]
    #[tokio::test]
    async fn test_rustls_webpki_roots_works() {
        let connector = RustlsHttpsConnector::with_webpki_roots();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://www.google.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(feature = "rustls-native")]
    #[tokio::test]
    async fn test_rustls_native_roots_works() {
        let connector = RustlsHttpsConnector::with_native_roots();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://www.google.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(feature = "native-tls")]
    #[tokio::test]
    async fn test_native_tls_works() {
        let connector = NativeTlsHttpsConnector::new();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://www.google.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }
}
