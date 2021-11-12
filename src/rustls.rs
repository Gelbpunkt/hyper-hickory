use hyper_rustls::{HttpsConnector, HttpsConnectorBuilder};

use crate::{new_trust_dns_http_connector, TrustDnsHttpConnector};

#[cfg(all(not(feature = "rustls-http1"), not(feature = "rustls-http2")))]
compile_error!("Either the rustls-http1 or the rustls-http2 feature must be enabled");

/// A [`HttpsConnector`] that uses a [`TrustDnsHttpConnector`].
#[cfg_attr(
    docsrs,
    doc(cfg(any(feature = "rustls-native", feature = "rustls-webpki")))
)]
pub type RustlsHttpsConnector = HttpsConnector<TrustDnsHttpConnector>;

// hyper-rustls has no way to build with a given HttpConnector except the HttpsConnector::from implementation
// which requires passing a ClientConfig.
// Code for native/webpki-roots is taken from https://github.com/rustls/hyper-rustls/blob/master/src/connector.rs#L32-L59

/// Create a new [`RustlsHttpsConnector`] using the OS root store.
#[cfg(feature = "rustls-native")]
#[must_use]
pub fn new_rustls_native_https_connector() -> RustlsHttpsConnector {
    let mut http_connector = new_trust_dns_http_connector();
    http_connector.enforce_http(false);

    let builder = HttpsConnectorBuilder::new().with_native_roots();

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
pub fn new_rustls_webpki_https_connector() -> RustlsHttpsConnector {
    let mut http_connector = new_trust_dns_http_connector();
    http_connector.enforce_http(false);

    let builder = HttpsConnectorBuilder::new().with_webpki_roots();

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
