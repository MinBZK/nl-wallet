use std::collections::HashMap;
use std::time::Duration;

use reqwest::header::ACCEPT;
use reqwest::header::CACHE_CONTROL;
use reqwest::header::ETAG;
use reqwest::header::IF_NONE_MATCH;
use reqwest::header::VARY;
use rstest::rstest;
use tempfile::TempDir;
use tokio::net::TcpListener;
use url::Url;

use status_lists::publish::PublishDir;
use status_lists::serve::create_serve_router;

async fn setup_server(publish_dir: &TempDir, ttl: Option<Duration>) -> anyhow::Result<Url> {
    let publish_dir = PublishDir::try_new(publish_dir.path().to_path_buf())?;
    let router = create_serve_router([("/tsl", publish_dir)].into_iter().collect::<HashMap<_, _>>(), ttl)?;
    let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
    let port = listener.local_addr()?.port();
    tokio::spawn(async move { axum::serve(listener, router).await.unwrap() });

    Ok(format!("http://127.0.0.1:{}/tsl/", port).parse()?)
}

#[tokio::test]
#[rstest]
#[case(None, "no-cache")]
#[case(Some(Duration::from_secs(300)), "max-age=300; must-revalidate")]
async fn test_router_serve_with_ttl(#[case] ttl: Option<Duration>, #[case] cache_control: &str) {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, ttl).await.unwrap();

    let path = publish_dir.path().join("test.jwt");
    tokio::fs::write(&path, "test123").await.unwrap();

    let response = reqwest::get(url.join("test").unwrap()).await.unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.headers().get(CACHE_CONTROL).unwrap(), cache_control);
    assert_eq!(
        response.headers().get(ETAG).unwrap(),
        "\"7-246253568559076899098453655604172054734\""
    );
    assert_eq!(response.headers().get(VARY).unwrap(), "accept");
    assert_eq!(response.bytes().await.unwrap(), "test123".as_bytes());
}

#[tokio::test]
async fn test_router_not_found() {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    let response = reqwest::get(url.join("test").unwrap()).await.unwrap();
    assert_eq!(response.headers().get(VARY).unwrap(), "accept");
    assert_eq!(response.status(), 404);
}

#[tokio::test]
#[rstest]
#[case("application/statuslist+jwt", 200)]
#[case("application/statuslist+cwt", 415)]
async fn test_router_media_type(#[case] accept: &str, #[case] status_code: u16) {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    let path = publish_dir.path().join("test.jwt");
    tokio::fs::write(&path, "test123").await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .request(reqwest::Method::GET, url.join("test").unwrap())
        .header(ACCEPT, accept)
        .send()
        .await
        .unwrap();

    assert_eq!(response.headers().get(VARY).unwrap(), "accept");
    assert_eq!(response.status(), status_code);
}

#[tokio::test]
#[rstest]
#[case("\"7-246253568559076899098453655604172054734\"", 304)]
#[case("\"7-246253568559076899098453655604172054735\"", 200)]
async fn test_router_etag(#[case] etag: &str, #[case] status_code: u16) {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    let path = publish_dir.path().join("test.jwt");
    tokio::fs::write(&path, "test123").await.unwrap();

    let client = reqwest::Client::new();
    let response = client
        .request(reqwest::Method::GET, url.join("test").unwrap())
        .header(IF_NONE_MATCH, etag)
        .send()
        .await
        .unwrap();

    assert_eq!(response.headers().get(VARY).unwrap(), "accept");
    assert_eq!(response.status(), status_code);
}
