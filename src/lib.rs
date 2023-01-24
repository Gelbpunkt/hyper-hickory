#![doc = include_str!("../README.md")]
#![deny(clippy::pedantic, missing_docs)]
#![allow(clippy::module_name_repetitions)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

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
    name_server::{RuntimeProvider, TokioRuntimeProvider},
    AsyncResolver, TokioAsyncResolver,
};

/// A hyper resolver using `trust-dns`'s [`TokioAsyncResolver`].
pub type TrustDnsResolver = GenericTrustDnsResolver<TokioRuntimeProvider>;

/// A hyper resolver using `trust-dns`'s [`TokioAsyncResolver`].
#[derive(Clone)]
pub struct GenericTrustDnsResolver<R: RuntimeProvider> {
    resolver: Arc<AsyncResolver<R>>,
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

/// Get the default resolver options as configured per crate features.
/// This allows us to enable DNSSEC conditionally.
fn default_opts() -> ResolverOpts {
    #[cfg(any(feature = "dnssec-openssl", feature = "dnssec-ring"))]
    let mut opts = ResolverOpts::default();
    #[cfg(not(any(feature = "dnssec-openssl", feature = "dnssec-ring")))]
    let opts = ResolverOpts::default();

    #[cfg(any(feature = "dnssec-openssl", feature = "dnssec-ring"))]
    {
        opts.validate = true;
    }

    opts
}

impl TrustDnsResolver {
    /// Create a new [`TrustDnsResolver`] with the default config options.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`TrustDnsResolver`] that uses the Google nameservers.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn google() -> Self {
        Self::with_config_and_options(ResolverConfig::google(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] that uses the Cloudflare nameservers.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn cloudflare() -> Self {
        Self::with_config_and_options(ResolverConfig::cloudflare(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] that uses the Cloudflare nameservers.
    /// This limits the registered connections to just HTTPS lookups.
    /// This must be run inside a Tokio runtime context.
    #[cfg(feature = "dns-over-https-rustls")]
    #[must_use]
    pub fn cloudflare_https() -> Self {
        Self::with_config_and_options(ResolverConfig::cloudflare_https(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] that uses the Cloudflare nameservers.
    /// This limits the registered connections to just TLS lookups.
    /// This must be run inside a Tokio runtime context.
    #[cfg(any(
        feature = "dns-over-rustls",
        feature = "dns-over-native-tls",
        feature = "dns-over-openssl"
    ))]
    #[must_use]
    pub fn cloudflare_tls() -> Self {
        Self::with_config_and_options(ResolverConfig::cloudflare_tls(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] that uses the Quad9 nameservers.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn quad9() -> Self {
        Self::with_config_and_options(ResolverConfig::quad9(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] that uses the Quad9 nameservers.
    /// This limits the registered connections to just HTTPS lookups.
    /// This must be run inside a Tokio runtime context.
    #[cfg(feature = "dns-over-https-rustls")]
    #[must_use]
    pub fn quad9_https() -> Self {
        Self::with_config_and_options(ResolverConfig::quad9_https(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] that uses the Quad9 nameservers.
    /// This limits the registered connections to just TLS lookups.
    /// This must be run inside a Tokio runtime context.
    #[cfg(any(
        feature = "dns-over-rustls",
        feature = "dns-over-native-tls",
        feature = "dns-over-openssl"
    ))]
    #[must_use]
    pub fn quad9_tls() -> Self {
        Self::with_config_and_options(ResolverConfig::quad9_tls(), default_opts())
    }

    /// Create a new [`TrustDnsResolver`] with the resolver configuration
    /// options specified.
    /// This must be run inside a Tokio runtime context.
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn with_config_and_options(config: ResolverConfig, options: ResolverOpts) -> Self {
        // This unwrap is safe because internally, there is nothing to be unwrapped
        // TokioAsyncResolver::new cannot return Err
        let resolver = Arc::new(TokioAsyncResolver::tokio(config, options).unwrap());

        Self { resolver }
    }

    /// Create a new [`TrustDnsResolver`] with the system configuration.
    /// This must be run inside a Tokio runtime context.
    #[cfg(feature = "system-config")]
    #[allow(clippy::missing_panics_doc)]
    #[must_use]
    pub fn from_system_conf() -> Self {
        // This unwrap is safe because internally, there is nothing to be unwrapped
        // TokioAsyncResolver::new cannot return Err
        let resolver = Arc::new(TokioAsyncResolver::tokio_from_system_conf().unwrap());

        Self { resolver }
    }

    /// Create a new [`NativeTlsHttpsConnector`].
    #[cfg(feature = "native-tls")]
    #[must_use]
    pub fn into_native_tls_https_connector(self) -> NativeTlsHttpsConnector {
        let mut http_connector = self.into_http_connector();
        http_connector.enforce_http(false);

        let mut native_https_connector =
            NativeTlsHttpsConnector::new_with_connector(http_connector);

        #[cfg(feature = "https-only")]
        native_https_connector.https_only(true);

        #[cfg(not(feature = "https-only"))]
        native_https_connector.https_only(false);

        native_https_connector
    }

    /// Create a new [`RustlsHttpsConnector`] using the OS root store.
    #[cfg(feature = "rustls-native")]
    #[must_use]
    pub fn into_rustls_native_https_connector(self) -> RustlsHttpsConnector {
        let mut http_connector = self.into_http_connector();
        http_connector.enforce_http(false);

        let builder = hyper_rustls::HttpsConnectorBuilder::new().with_native_roots();

        #[cfg(feature = "https-only")]
        let builder = builder.https_only();

        #[cfg(not(feature = "https-only"))]
        let builder = builder.https_or_http();

        #[cfg(feature = "rustls-http1")]
        let builder = builder.enable_http1();

        #[cfg(feature = "rustls-http2")]
        let builder = builder.enable_http2();

        builder.wrap_connector(http_connector)
    }

    /// Create a new [`RustlsHttpsConnector`] using the `webpki_roots`.
    #[cfg(feature = "rustls-webpki")]
    #[must_use]
    pub fn into_rustls_webpki_https_connector(self) -> RustlsHttpsConnector {
        let mut http_connector = self.into_http_connector();
        http_connector.enforce_http(false);

        let builder = hyper_rustls::HttpsConnectorBuilder::new().with_webpki_roots();

        #[cfg(feature = "https-only")]
        let builder = builder.https_only();

        #[cfg(not(feature = "https-only"))]
        let builder = builder.https_or_http();

        #[cfg(feature = "rustls-http1")]
        let builder = builder.enable_http1();

        #[cfg(feature = "rustls-http2")]
        let builder = builder.enable_http2();

        builder.wrap_connector(http_connector)
    }
}

impl<R: RuntimeProvider> GenericTrustDnsResolver<R> {
    /// TODO
    #[must_use]
    pub fn from_async_resolver(async_resolver: AsyncResolver<R>) -> Self {
        let resolver = Arc::new(async_resolver);

        Self { resolver }
    }

    /// Create a new [`TrustDnsHttpConnector`] with this resolver.
    #[must_use]
    pub fn into_http_connector(self) -> GenericTrustDnsHttpConnector<R> {
        GenericTrustDnsHttpConnector::new_with_resolver(self)
    }
}

impl Default for TrustDnsResolver {
    fn default() -> Self {
        Self::with_config_and_options(ResolverConfig::default(), default_opts())
    }
}

impl<R: RuntimeProvider> Service<Name> for GenericTrustDnsResolver<R> {
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

/// A [`HttpConnector`] that uses the [`GenericTrustDnsResolver`].
pub type GenericTrustDnsHttpConnector<R> = HttpConnector<GenericTrustDnsResolver<R>>;

/// A [`hyper_tls::HttpsConnector`] that uses a [`TrustDnsHttpConnector`].
#[cfg(feature = "native-tls")]
pub type NativeTlsHttpsConnector = hyper_tls::HttpsConnector<TrustDnsHttpConnector>;

/// A [`hyper_rustls::HttpsConnector`] that uses a [`TrustDnsHttpConnector`].
#[cfg(any(feature = "rustls-native", feature = "rustls-webpki"))]
pub type RustlsHttpsConnector = hyper_rustls::HttpsConnector<TrustDnsHttpConnector>;
