# hyper-trust-dns

This crate provides HTTP/HTTPS connectors for [hyper](https://github.com/hyperium/hyper) that use the fast and advanced DNS resolver of [trust-dns](https://github.com/bluejekyll/trust-dns) instead of the default threadpool implementation of hyper.

## Types of connectors

There are 3 connectors:

- `TrustDnsHttpConnector`, a wrapper around `HttpConnector<TrustDnsResolver>`
- `RustlsHttpsConnector`, a modified version of [hyper-rustls](https://github.com/rustls/hyper-rustls)'s connector to work with `TrustDnsHttpConnector`
- `NativeTlsHttpsConnector`, a modified version of [hyper-tls](https://github.com/hyperium/hyper-tls)'s connector to work with `TrustDnsHttpConnector`

The HTTP connector is always available, the other two can be enabled via the `rustls-webpki` (uses webpki roots)/`rustls-native` (uses OS cert store) and `native-tls` features respectably.

## Trust-DNS options

The crate has other features that toggle functionality in [trust-dns-resolver](https://github.com/bluejekyll/trust-dns/tree/main/crates/resolver), namingly `dns-over-openssl`, `dns-over-native-tls` and `dns-over-rustls` for DNS-over-TLS, `dns-over-https-rustls` for DNS-over-HTTPS and `dnssec-openssl` and `dnssec-ring` for DNSSEC.

## License

MIT. The licenses for `hyper-rustls` and `hyper-tls` have been added in the respectable directories of the source code; if a maintainer of these crates deems this to be violation of the MIT license, feel free to contact me to sort it out.
