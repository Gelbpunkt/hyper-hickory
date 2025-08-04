# hyper-hickory

This crate provides a HTTP connector for [hyper](https://github.com/hyperium/hyper) that uses the fast and advanced DNS resolver of [hickory](https://github.com/hickory-dns/hickory-dns) instead of the default threadpool implementation of hyper.

## Usage

```rust
# #![cfg(feature = "tokio")]

# fn main() {
use http_body_util::Full; // Or your preferred Body implementation
use hyper::body::Bytes;
use hyper_hickory::TokioHickoryResolver;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};

let connector = TokioHickoryResolver::default().into_http_connector();
let client: Client<_, Full<Bytes>> = Client::builder(TokioExecutor::new()).build(connector);
# }
```

## Resolvers

There is a [`HickoryResolver`] resolver which can be built from an [`Resolver`] using [`HickoryResolver::from_resolver`].

For most cases where you are happy to use the standard [`TokioRuntimeProvider`](https://docs.rs/hickory-resolver/latest/hickory_resolver/name_server/struct.TokioRuntimeProvider.html), the [`TokioHickoryResolver`] should be used and is able to be built much more easily. It requires enabling the `tokio` feature flag.


## Types of connectors

There are 2 connectors:

- [`HickoryHttpConnector<C>`], a wrapper around [`HttpConnector<HickoryResolver<C>>`]. Created with [`HickoryResolver::into_http_connector`].
- [`TokioHickoryHttpConnector`], an alias to [`HickoryHttpConnector<TokioConnectionProvider>`].

## Hickory options

The crate has other features that toggle functionality in [hickory-resolver](https://github.com/hickory-dns/hickory-dns/tree/main/crates/resolver), such as DNSSEC or DOH / DOT.