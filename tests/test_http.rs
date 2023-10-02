use hyper::{Body, Client, Request};
use hyper_trust_dns::TokioTrustDnsResolver;

#[tokio::test]
async fn test_lookup_works() {
    let connector = TokioTrustDnsResolver::default().into_http_connector();
    let client = Client::builder().build(connector);

    let request = Request::builder()
        .method("GET")
        .uri("http://example.com/")
        .body(Body::empty())
        .unwrap();

    let response = client.request(request).await.unwrap();

    assert_eq!(response.status(), 200);
}
