# hyper-hickory

This crate provides HTTP/HTTPS connectors for [hyper](https://github.com/hyperium/hyper) that use the fast and advanced DNS resolver of [hickory](https://github.com/hickory-dns/hickory-dns) instead of the default threadpool implementation of hyper.

## Usage

```rust
use hyper::Client;
use hyper_hickory::HickoryResolver;

let connector = HickoryResolver::default().into_rustls_webpki_https_connector();
let client: Client<_> = Client::builder().build(connector);
```

## Resolvers

There is a [`HickoryResolver`] resolver which can be built from an [`AsyncResolver`] using [`HickoryResolver::from_async_resolver`].

For most cases where you are happy to use the standard [`TokioRuntimeProvider`](https://docs.rs/hickory-resolver/latest/hickory_resolver/name_server/struct.TokioRuntimeProvider.html), the [`TokioHickoryResolver`] should be used and is able to be built much more easily
(but requires the `tokio` feature to be enabled, which it is by default).


## Types of connectors

There are 6 connectors:

- [`HickoryHttpConnector`], a wrapper around [`HttpConnector<HickoryResolver>`]. Created with [`HickoryResolver::into_http_connector`].
- [`RustlsHttpsConnector`], a [hyper-rustls](https://github.com/rustls/hyper-rustls) based connector to work with [`HickoryHttpConnector`]. Created with [`HickoryResolver::into_rustls_native_https_connector`] or [`HickoryResolver::into_rustls_webpki_https_connector`].
- [`NativeTlsHttpsConnector`], a [hyper-tls](https://github.com/hyperium/hyper-tls) based connector to work with [`HickoryHttpConnector`]. Created with [`HickoryResolver::into_native_tls_https_connector`].
- [`TokioHickoryHttpConnector`], a wrapper around [`HickoryHttpConnector<TokioRuntimeProvider>`].
- [`TokioNativeTlsHttpsConnector`], a wrapper around [`NativeTlsHttpsConnector<TokioRuntimeProvider>`]
- [`TokioRustlsHttpsConnector`], a wrapper around [`RustlsHttpsConnector<TokioRuntimeProvider>`]

The HTTP connector is always available, the other two non-tokio ones can be enabled via the `hyper-rustls` feature combined with `rustls-webpki` (uses webpki roots)/`rustls-native` (uses OS cert store) and `native-tls` features respectably. The `Tokio` prefixed variants also require the `tokio` feature to be enabled (which it is by default).

## Hickory options

The crate has other features that toggle functionality in [hickory-resolver](https://github.com/hickory-dns/hickory-dns/tree/main/crates/resolver), namingly `dns-over-openssl`, `dns-over-native-tls` and `dns-over-rustls` (combined with `rustls-webpki` or `rustls-native`) for DNS-over-TLS, `dns-over-https-rustls` for DNS-over-HTTPS and `dnssec-openssl` and `dnssec-ring` for DNSSEC.

## A note on DNSSEC

DNSSEC functionality was never actually used if enabled prior to version 0.5.0 of this crate. This has been changed since and might result in sudden, breaking behaviour due to hickory-resolver failing on unsigned records.

This behaviour will continue until [DNSSEC is improved in hickory](https://github.com/hickory-dns/hickory-dns/issues/1708).
