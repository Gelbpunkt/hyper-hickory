use hyper::{client::connect::Connection, service::Service, Uri};
use rustls::ClientConfig;
use std::{
    fmt,
    future::Future,
    io,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_rustls::TlsConnector;
use webpki::DNSNameRef;

use super::MaybeHttpsStream;
use crate::{new_trust_dns_http_connector, TrustDnsHttpConnector};

type BoxError = Box<dyn std::error::Error + Send + Sync>;

/// A Connector for the `https` scheme.
#[derive(Clone)]
pub struct HttpsConnector<T> {
    force_https: bool,
    http: T,
    tls_config: Arc<ClientConfig>,
}

impl HttpsConnector<TrustDnsHttpConnector> {
    /// Construct a new `HttpsConnector` using the OS root store
    ///
    /// # Panics
    ///
    /// This method will panic if the native certificate store cannot be accessed
    /// or it is empty.
    #[cfg(feature = "rustls-native")]
    #[must_use]
    pub fn with_native_roots() -> Self {
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
        Self::build(config)
    }

    /// Construct a new `HttpsConnector` using the `webpki_roots`
    #[cfg(feature = "webpki-roots")]
    #[must_use]
    pub fn with_webpki_roots() -> Self {
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
        Self::build(config)
    }

    /// Force the use of HTTPS when connecting.
    ///
    /// If a URL is not `https` when connecting, an error is returned. Disabled by default.
    pub fn https_only(&mut self, enable: bool) {
        self.force_https = enable;
    }

    fn build(mut config: ClientConfig) -> Self {
        let mut http = new_trust_dns_http_connector();
        http.enforce_http(false);

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

        (http, config).into()
    }
}

impl<T> fmt::Debug for HttpsConnector<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("HttpsConnector")
            .field("force_https", &self.force_https)
            .finish()
    }
}

impl<H, C> From<(H, C)> for HttpsConnector<H>
where
    C: Into<Arc<ClientConfig>>,
{
    fn from((http, cfg): (H, C)) -> Self {
        HttpsConnector {
            force_https: false,
            http,
            tls_config: cfg.into(),
        }
    }
}

impl<T> Service<Uri> for HttpsConnector<T>
where
    T: Service<Uri>,
    T::Response: Connection + AsyncRead + AsyncWrite + Send + Unpin + 'static,
    T::Future: Send + 'static,
    T::Error: Into<BoxError>,
{
    type Response = MaybeHttpsStream<T::Response>;
    type Error = BoxError;

    #[allow(clippy::type_complexity)]
    type Future =
        Pin<Box<dyn Future<Output = Result<MaybeHttpsStream<T::Response>, BoxError>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        match self.http.poll_ready(cx) {
            Poll::Ready(Ok(())) => Poll::Ready(Ok(())),
            Poll::Ready(Err(e)) => Poll::Ready(Err(e.into())),
            Poll::Pending => Poll::Pending,
        }
    }

    fn call(&mut self, dst: Uri) -> Self::Future {
        let is_https = dst.scheme_str() == Some("https");

        if is_https {
            let cfg = self.tls_config.clone();
            let hostname = dst.host().unwrap_or_default().to_string();
            let connecting_future = self.http.call(dst);

            let f = async move {
                let tcp = connecting_future.await.map_err(Into::into)?;
                let connector = TlsConnector::from(cfg);
                let dnsname = DNSNameRef::try_from_ascii_str(&hostname)
                    .map_err(|_| io::Error::new(io::ErrorKind::Other, "invalid dnsname"))?;
                let tls = connector
                    .connect(dnsname, tcp)
                    .await
                    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
                Ok(MaybeHttpsStream::Https(tls))
            };
            Box::pin(f)
        } else if self.force_https {
            // Early abort if HTTPS is forced but can't be used
            let err = io::Error::new(io::ErrorKind::Other, "https required but URI was not https");
            Box::pin(async move { Err(err.into()) })
        } else {
            let connecting_future = self.http.call(dst);

            let f = async move {
                let tcp = connecting_future.await.map_err(Into::into)?;

                Ok(MaybeHttpsStream::Http(tcp))
            };
            Box::pin(f)
        }
    }
}