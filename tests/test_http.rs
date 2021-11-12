use hyper::{Body, Client, Request};
use hyper_trust_dns::new_trust_dns_http_connector;

#[tokio::test]
async fn test_lookup_works() {
    let connector = new_trust_dns_http_connector();
    let client = Client::builder().build(connector);

    let request = Request::builder()
        .method("GET")
        .uri("http://www.google.com/")
        .body(Body::empty())
        .unwrap();

    let response = client.request(request).await.unwrap();

    assert_eq!(response.status(), 200);
}
