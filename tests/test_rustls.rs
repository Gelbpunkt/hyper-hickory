mod tests {
    use hyper::{Body, Client, Request};
    use hyper_trust_dns::TokioTrustDnsResolver;

    #[cfg(feature = "rustls-webpki")]
    #[tokio::test]
    async fn test_rustls_webpki_roots_works() {
        let connector = TokioTrustDnsResolver::default().into_rustls_webpki_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://example.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(feature = "rustls-native")]
    #[tokio::test]
    async fn test_rustls_native_roots_works() {
        let connector = TokioTrustDnsResolver::default().into_rustls_native_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://example.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(all(feature = "system-config", feature = "rustls-native"))]
    #[tokio::test]
    async fn test_sytem_config_works() {
        let connector =
            TokioTrustDnsResolver::from_system_conf().into_rustls_native_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://example.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(all(feature = "dns-over-rustls", feature = "rustls-native"))]
    #[tokio::test]
    async fn test_dns_over_rustls_works() {
        let connector =
            TokioTrustDnsResolver::cloudflare_tls().into_rustls_native_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://example.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(all(feature = "dns-over-https-rustls", feature = "rustls-native"))]
    #[tokio::test]
    async fn test_dns_over_https_rustls_works() {
        let connector =
            TokioTrustDnsResolver::cloudflare_https().into_rustls_native_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://example.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }
}
