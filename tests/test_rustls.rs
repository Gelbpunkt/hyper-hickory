mod tests {
    use hyper::{Body, Client, Request};

    #[cfg(feature = "rustls-webpki")]
    #[tokio::test]
    async fn test_rustls_webpki_roots_works() {
        use hyper_trust_dns::new_rustls_webpki_https_connector;

        let connector = new_rustls_webpki_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://www.google.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }

    #[cfg(feature = "rustls-native")]
    #[tokio::test]
    async fn test_rustls_native_roots_works() {
        use hyper_trust_dns::new_rustls_native_https_connector;

        let connector = new_rustls_native_https_connector();
        let client = Client::builder().build(connector);

        let request = Request::builder()
            .method("GET")
            .uri("https://www.google.com/")
            .body(Body::empty())
            .unwrap();

        let response = client.request(request).await.unwrap();

        assert_eq!(response.status(), 200);
    }
}
