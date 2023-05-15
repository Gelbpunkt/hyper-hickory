# hyper-trust-dns

This crate provides HTTP/HTTPS connectors for [hyper](https://github.com/hyperium/hyper) that use the fast and advanced DNS resolver of [trust-dns](https://github.com/bluejekyll/trust-dns) instead of the default threadpool implementation of hyper.

## Usage

```rust
use hyper::Client;
use hyper_trust_dns::TrustDnsResolver;

let connector = TrustDnsResolver::default().into_rustls_native_https_connector();
let client: Client<_> = Client::builder().build(connector);
```

## Resolvers

There is a [`TrustDnsResolver`] resolver which can be built from an [`AsyncResolver`] using [`TrustDnsResolver::from_async_resolver`].

For most cases where you are happy to use the standard [`TokioRuntimeProvider`] the [`TokioTrustDnsResolver`] should be used and is able to be built much more easily
(but requires the `tokio` feature to be enabled, which it is by default).


## Types of connectors

There are 6 connectors:

- [`TrustDnsHttpConnector`], a wrapper around [`HttpConnector<TrustDnsResolver>`]. Created with [`TrustDnsResolver::into_http_connector`].
- [`RustlsHttpsConnector`], a [hyper-rustls](https://github.com/rustls/hyper-rustls) based connector to work with [`TrustDnsHttpConnector`]. Created with [`TrustDnsResolver::into_rustls_native_https_connector`] or [`TrustDnsResolver::into_rustls_webpki_https_connector`].
- [`NativeTlsHttpsConnector`], a [hyper-tls](https://github.com/hyperium/hyper-tls) based connector to work with [`TrustDnsHttpConnector`]. Created with [`TrustDnsResolver::into_native_tls_https_connector`].
- [`TokioTrustDnsHttpConnector`], a wrapper around [`TrustDnsHttpConnector<TokioRuntimeProvider>`].
- [`TokioNativeTlsHttpsConnector`], a wrapper around [`NativeTlsHttpsConnector<TokioRuntimeProvider>`]
- [`TokioRustlsHttpsConnector`], a wrapper around [`RustlsHttpsConnector<TokioRuntimeProvider>`]

The HTTP connector is always available, the other two non-tokio ones can be enabled via the `rustls-webpki` (uses webpki roots)/`rustls-native` (uses OS cert store) and `native-tls` features respectably. The `Tokio` prefixed variants also require the `tokio` feature to be enabled (which it is by default).

## Trust-DNS options

The crate has other features that toggle functionality in [trust-dns-resolver](https://github.com/bluejekyll/trust-dns/tree/main/crates/resolver), namingly `dns-over-openssl`, `dns-over-native-tls` and `dns-over-rustls` for DNS-over-TLS, `dns-over-https-rustls` for DNS-over-HTTPS and `dnssec-openssl` and `dnssec-ring` for DNSSEC.

## A note on DNSSEC

DNSSEC functionality was never actually used if enabled prior to version 0.5.0 of this crate. This has been changed since and might result in sudden, breaking behaviour due to trust-dns-resolver failing on unsigned records.

This behaviour will continue until [DNSSEC is improved in trust-dns](https://github.com/bluejekyll/trust-dns/issues/1708).
