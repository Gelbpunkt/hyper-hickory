//! This crate provides HTTP/HTTPS connectors for [hyper](https://github.com/hyperium/hyper) that use the fast and advanced DNS resolver of [trust-dns](https://github.com/bluejekyll/trust-dns) instead of the default threadpool implementation of hyper.
//!
//! ## Types of connectors
//!
//! There are 3 connectors:
//!
//! - [`TrustDnsHttpConnector`], a wrapper around [`HttpConnector<TrustDnsResolver>`]. Created with [`new_trust_dns_http_connector`].
//! - [`RustlsHttpsConnector`], a [hyper-rustls](https://github.com/rustls/hyper-rustls) based connector to work with [`TrustDnsHttpConnector`]. Created with [`new_rustls_webpki_https_connector`] or [`new_rustls_native_https_connector`].
//! - [`NativeTlsHttpsConnector`], a [hyper-tls](https://github.com/hyperium/hyper-tls) based connector to work with [`TrustDnsHttpConnector`]. Created with [`new_native_tls_https_connector`].
//!
//! The HTTP connector is always available, the other two can be enabled via the `rustls-webpki` (uses webpki roots)/`rustls-native` (uses OS cert store) and `native-tls` features respectably.
//!
//! ## Trust-DNS options
//!
//! The crate has other features that toggle functionality in [trust-dns-resolver](https://github.com/bluejekyll/trust-dns/tree/main/crates/resolver), namingly `dns-over-openssl`, `dns-over-native-tls` and `dns-over-rustls` for DNS-over-TLS, `dns-over-https-rustls` for DNS-over-HTTPS and `dnssec-openssl` and `dnssec-ring` for DNSSEC.
#![deny(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_cfg))]

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
pub use crate::native_tls::{new_native_tls_https_connector, NativeTlsHttpsConnector};
#[cfg(feature = "__rustls")]
pub use crate::rustls::*;

#[cfg(feature = "native-tls")]
mod native_tls;
#[cfg(feature = "__rustls")]
mod rustls;

/// A hyper resolver using `trust-dns`'s [`TokioAsyncResolver`].
#[derive(Clone)]
pub struct TrustDnsResolver {
    resolver: Arc<TokioAsyncResolver>,
}

/// Iterator over DNS lookup results.
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

/// A [`HttpConnector`] that uses the [`TrustDnsResolver`].
pub type TrustDnsHttpConnector = HttpConnector<TrustDnsResolver>;

/// Create a new [`TrustDnsHttpConnector`] that only supports HTTP.
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
        let connector = new_rustls_webpki_https_connector();
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
        let connector = new_rustls_native_https_connector();
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
        let connector = new_native_tls_https_connector();
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
