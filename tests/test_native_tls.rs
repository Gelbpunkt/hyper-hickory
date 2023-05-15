#[cfg(feature = "native-tls")]
mod tests {
    use hyper::{Body, Client, Request};
    use hyper_trust_dns::TokioTrustDnsResolver;

    #[tokio::test]
    async fn test_native_tls_works() {
        let connector = TrustDnsResolver::default().into_native_tls_https_connector();
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
