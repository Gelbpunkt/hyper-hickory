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

use hickory_resolver::{
    error::ResolveError, lookup_ip::LookupIpIntoIter, name_server::ConnectionProvider,
    AsyncResolver,
};

use hickory_resolver::{
    config::{ResolverConfig, ResolverOpts},
    name_server::TokioConnectionProvider,
    TokioAsyncResolver,
};
use hyper_util::client::legacy::connect::{dns::Name, HttpConnector};
use tower_service::Service;

/// A hyper resolver using `hickory`'s [`TokioAsyncResolver`].
pub type TokioHickoryResolver = HickoryResolver<TokioConnectionProvider>;

/// A hyper resolver using `hickory`'s [`AsyncResolver`] and any implementor of [`ConnectionProvider`].
#[derive(Clone)]
pub struct HickoryResolver<C: ConnectionProvider> {
    resolver: Arc<AsyncResolver<C>>,
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

impl TokioHickoryResolver {
    /// Create a new [`TokioHickoryResolver`] with the default config options.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new [`TokioHickoryResolver`] that uses the Google nameservers.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn google() -> Self {
        Self::with_config_and_options(ResolverConfig::google(), default_opts())
    }

    /// Create a new [`TokioHickoryResolver`] that uses the Cloudflare nameservers.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn cloudflare() -> Self {
        Self::with_config_and_options(ResolverConfig::cloudflare(), default_opts())
    }

    /// Create a new [`TokioHickoryResolver`] that uses the Cloudflare nameservers.
    /// This limits the registered connections to just HTTPS lookups.
    /// This must be run inside a Tokio runtime context.
    #[cfg(feature = "dns-over-https-rustls")]
    #[must_use]
    pub fn cloudflare_https() -> Self {
        Self::with_config_and_options(ResolverConfig::cloudflare_https(), default_opts())
    }

    /// Create a new [`TokioHickoryResolver`] that uses the Cloudflare nameservers.
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

    /// Create a new [`TokioHickoryResolver`] that uses the Quad9 nameservers.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn quad9() -> Self {
        Self::with_config_and_options(ResolverConfig::quad9(), default_opts())
    }

    /// Create a new [`TokioHickoryResolver`] that uses the Quad9 nameservers.
    /// This limits the registered connections to just HTTPS lookups.
    /// This must be run inside a Tokio runtime context.
    #[cfg(feature = "dns-over-https-rustls")]
    #[must_use]
    pub fn quad9_https() -> Self {
        Self::with_config_and_options(ResolverConfig::quad9_https(), default_opts())
    }

    /// Create a new [`TokioHickoryResolver`] that uses the Quad9 nameservers.
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

    /// Create a new [`TokioHickoryResolver`] with the resolver configuration
    /// options specified.
    /// This must be run inside a Tokio runtime context.
    #[must_use]
    pub fn with_config_and_options(config: ResolverConfig, options: ResolverOpts) -> Self {
        Self::from_async_resolver(TokioAsyncResolver::tokio(config, options))
    }

    /// Create a new [`TokioHickoryResolver`] with the system configuration.
    /// This must be run inside a Tokio runtime context.
    ///
    /// # Errors
    ///
    /// This method returns an error if loading the system configuration fails.
    #[cfg(feature = "system-config")]
    pub fn from_system_conf() -> Result<Self, hickory_resolver::error::ResolveError> {
        Ok(Self::from_async_resolver(
            TokioAsyncResolver::tokio_from_system_conf()?,
        ))
    }
}

impl Default for TokioHickoryResolver {
    fn default() -> Self {
        Self::with_config_and_options(ResolverConfig::default(), default_opts())
    }
}

impl<C: ConnectionProvider> HickoryResolver<C> {
    /// Create a [`HickoryResolver`] from the given [`AsyncResolver`]
    #[must_use]
    pub fn from_async_resolver(async_resolver: AsyncResolver<C>) -> Self {
        let resolver = Arc::new(async_resolver);

        Self { resolver }
    }

    /// Create a new [`HickoryHttpConnector`] with this resolver.
    #[must_use]
    pub fn into_http_connector(self) -> HickoryHttpConnector<C> {
        HickoryHttpConnector::new_with_resolver(self)
    }
}

impl<C: ConnectionProvider> Service<Name> for HickoryResolver<C> {
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

/// A [`HttpConnector`] that uses the [`HickoryResolver`].
pub type HickoryHttpConnector<C> = HttpConnector<HickoryResolver<C>>;

/// A [`HttpConnector`] that uses the [`TokioHickoryResolver`].
pub type TokioHickoryHttpConnector = HickoryHttpConnector<TokioConnectionProvider>;
