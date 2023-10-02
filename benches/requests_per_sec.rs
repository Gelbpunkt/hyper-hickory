use hyper::{Body, Client, Request};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

async fn hyper_threadpool_request(
    client: Client<hyper_rustls::HttpsConnector<hyper::client::HttpConnector>>,
) {
    let request = Request::builder()
        .method("GET")
        .uri("https://travitia.xyz/")
        .body(Body::empty())
        .unwrap();

    let response = client.request(request).await.unwrap();
    assert_eq!(response.status(), 200);
}

async fn hyper_trust_dns_request(client: Client<hyper_trust_dns::RustlsHttpsConnector>) {
    let request = Request::builder()
        .method("GET")
        .uri("https://travitia.xyz/")
        .body(Body::empty())
        .unwrap();

    let response = client.request(request).await.unwrap();
    assert_eq!(response.status(), 200);
}

fn hyper_threadpool(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_only()
        .enable_http1()
        .enable_http2()
        .build();
    let client: Client<_> = Client::builder().build(https_connector);

    c.bench_with_input(
        BenchmarkId::new("hyper_threadpool", "Client"),
        &client,
        |b, c| {
            b.to_async(&rt).iter(|| hyper_threadpool_request(c.clone()));
        },
    );
}

fn hyper_trust_dns(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    let https_connector =
        hyper_trust_dns::TokioTrustDnsResolver::new().into_rustls_webpki_https_connector();
    let client: Client<_> = Client::builder().build(https_connector);

    c.bench_with_input(
        BenchmarkId::new("hyper_trust_dns", "Client"),
        &client,
        |b, c| {
            b.to_async(&rt).iter(|| hyper_trust_dns_request(c.clone()));
        },
    );
}

criterion_group!(benches, hyper_trust_dns, hyper_threadpool);
criterion_main!(benches);
