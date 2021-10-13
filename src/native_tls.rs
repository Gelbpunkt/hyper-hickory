use hyper_tls::HttpsConnector;

use crate::{new_trust_dns_http_connector, TrustDnsHttpConnector};

pub type NativeTlsHttpsConnector = HttpsConnector<TrustDnsHttpConnector>;

/// Create a new [`NativeTlsHttpsConnector`].
#[must_use]
pub fn new_native_tls_https_connector() -> NativeTlsHttpsConnector {
    let mut http_connector = new_trust_dns_http_connector();
    http_connector.enforce_http(false);

    NativeTlsHttpsConnector::new_with_connector(http_connector)
}
