use hyper_rustls::HttpsConnector;
use rustls::ClientConfig;

use crate::{new_trust_dns_http_connector, TrustDnsHttpConnector};

/// A [`HttpsConnector`] that uses a [`TrustDnsHttpConnector`].
#[cfg_attr(docsrs, doc(cfg(any(feature = "rustls-native", feature = "rustls-webpki"))))]
pub type RustlsHttpsConnector = HttpsConnector<TrustDnsHttpConnector>;

// hyper-rustls has no way to build with a given HttpConnector except the HttpsConnector::from implementation
// which requires passing a ClientConfig.
// Code for native/webpki-roots is taken from https://github.com/rustls/hyper-rustls/blob/master/src/connector.rs#L32-L59

#[cfg(feature = "rustls-native")]
fn with_native_roots() -> ClientConfig {
    let mut config = ClientConfig::new();

    config.root_store = match rustls_native_certs::load_native_certs() {
        Ok(store) => store,
        Err((Some(store), err)) => {
            log::warn!("Could not load all certificates: {:?}", err);
            store
        }
        Err((None, err)) => Err(err).expect("cannot access native cert store"),
    };

    if config.root_store.is_empty() {
        panic!("no CA certificates found");
    }

    config
}

#[cfg(feature = "rustls-webpki")]
fn with_webpki_roots() -> ClientConfig {
    let mut config = ClientConfig::new();

    config
        .root_store
        .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

    config
}

/// Create a new [`RustlsHttpsConnector`] using the OS root store.
#[cfg(feature = "rustls-native")]
#[must_use]
pub fn new_rustls_native_https_connector() -> RustlsHttpsConnector {
    let mut http_connector = new_trust_dns_http_connector();
    http_connector.enforce_http(false);

    let mut config = with_native_roots();

    config.alpn_protocols.clear();
    #[cfg(feature = "http2")]
    {
        config.alpn_protocols.push(b"h2".to_vec());
    }

    #[cfg(feature = "http1")]
    {
        config.alpn_protocols.push(b"http/1.1".to_vec());
    }

    config.ct_logs = Some(&ct_logs::LOGS);

    (http_connector, config).into()
}

/// Create a new [`RustlsHttpsConnector`] using the `webpki_roots`.
#[cfg(feature = "rustls-webpki")]
#[must_use]
pub fn new_rustls_webpki_https_connector() -> RustlsHttpsConnector {
    let mut http_connector = new_trust_dns_http_connector();
    http_connector.enforce_http(false);

    let mut config = with_webpki_roots();

    config.alpn_protocols.clear();
    #[cfg(feature = "http2")]
    {
        config.alpn_protocols.push(b"h2".to_vec());
    }

    #[cfg(feature = "http1")]
    {
        config.alpn_protocols.push(b"http/1.1".to_vec());
    }

    config.ct_logs = Some(&ct_logs::LOGS);

    (http_connector, config).into()
}
