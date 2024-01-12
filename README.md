# hyper-hickory

This crate provides a HTTP connector for [hyper](https://github.com/hyperium/hyper) that uses the fast and advanced DNS resolver of [hickory](https://github.com/hickory-dns/hickory-dns) instead of the default threadpool implementation of hyper.

## Usage

```rust
use http_body_util::Full; // Or your preferred Body implementation
use hyper::body::Bytes;
use hyper_hickory::HickoryResolver;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};

let connector = HickoryResolver::default().into_http_connector();
let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(connector);
```

## Resolvers

There is a [`HickoryResolver`] resolver which can be built from an [`AsyncResolver`] using [`HickoryResolver::from_async_resolver`].

For most cases where you are happy to use the standard [`TokioRuntimeProvider`](https://docs.rs/hickory-resolver/latest/hickory_resolver/name_server/struct.TokioRuntimeProvider.html), the [`TokioHickoryResolver`] should be used and is able to be built much more easily.


## Types of connectors

There are 2 connectors:

- [`HickoryHttpConnector<C>`], a wrapper around [`HttpConnector<HickoryResolver<C>>`]. Created with [`HickoryResolver::into_http_connector`].
- [`TokioHickoryHttpConnector`], an alias to [`HickoryHttpConnector<TokioConnectionProvider>`].

## Hickory options

The crate has other features that toggle functionality in [hickory-resolver](https://github.com/hickory-dns/hickory-dns/tree/main/crates/resolver), namingly `dns-over-openssl`, `dns-over-native-tls` and `dns-over-rustls` (combined with `rustls-webpki` or `rustls-native`) for DNS-over-TLS, `dns-over-https-rustls` for DNS-over-HTTPS and `dnssec-openssl` and `dnssec-ring` for DNSSEC.

## A note on DNSSEC

DNSSEC functionality was never actually used if enabled prior to version 0.5.0 of this crate. This has been changed since and might result in sudden, breaking behaviour due to hickory-resolver failing on unsigned records.

This behaviour will continue until [DNSSEC is improved in hickory](https://github.com/hickory-dns/hickory-dns/issues/1708).
