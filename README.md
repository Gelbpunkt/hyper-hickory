# hyper-trust-dns

This crate provides HTTP/HTTPS connectors for [hyper](https://github.com/hyperium/hyper) that use the fast and advanced DNS resolver of [trust-dns](https://github.com/bluejekyll/trust-dns) instead of the default threadpool implementation of hyper.

## Types of connectors

There are 3 connectors:

- `TrustDnsHttpConnector`, a wrapper around `HttpConnector<TrustDnsResolver>`. Created with `hyper_trust_dns::new_trust_dns_http_connector`.
- `RustlsHttpsConnector`, a [hyper-rustls](https://github.com/rustls/hyper-rustls) based connector to work with `TrustDnsHttpConnector`. Created with `hyper_trust_dns::new_rustls_webpki_https_connector` or `hyper_trust_dns::new_rustls_native_https_connector`.
- `NativeTlsHttpsConnector`, a [hyper-tls](https://github.com/hyperium/hyper-tls) based connector to work with `TrustDnsHttpConnector`. Created with `hyper_trust_dns::new_native_tls_https_connector`.

The HTTP connector is always available, the other two can be enabled via the `rustls-webpki` (uses webpki roots)/`rustls-native` (uses OS cert store) and `native-tls` features respectably.

## Trust-DNS options

The crate has other features that toggle functionality in [trust-dns-resolver](https://github.com/bluejekyll/trust-dns/tree/main/crates/resolver), namingly `dns-over-openssl`, `dns-over-native-tls` and `dns-over-rustls` for DNS-over-TLS, `dns-over-https-rustls` for DNS-over-HTTPS and `dnssec-openssl` and `dnssec-ring` for DNSSEC.

## License

MIT
