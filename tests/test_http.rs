use http::Uri;
use http_body_util::Empty;
use hyper::body::Bytes;
use hyper_hickory::TokioHickoryResolver;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};

#[tokio::test]
async fn test_lookup_works() {
    let connector = TokioHickoryResolver::default().into_http_connector();
    let client: Client<_, Empty<Bytes>> = Client::builder(TokioExecutor::new()).build(connector);

    let response = client
        .get(Uri::from_static("http://example.com/"))
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
}
