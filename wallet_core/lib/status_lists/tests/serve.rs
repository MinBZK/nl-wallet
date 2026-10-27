use std::time::Duration;

use http::StatusCode;
use reqwest::header::ACCEPT;
use reqwest::header::CACHE_CONTROL;
use reqwest::header::CONTENT_TYPE;
use reqwest::header::ETAG;
use reqwest::header::IF_NONE_MATCH;
use reqwest::header::VARY;
use rstest::rstest;
use status_lists::publish::PublishDir;
use status_lists::serve::StatusListRouteSource;
use status_lists::serve::create_serve_router;
use tempfile::TempDir;
use tokio::net::TcpListener;
use url::Url;

async fn setup_server(publish_dir: &TempDir, ttl: Option<Duration>) -> anyhow::Result<Url> {
    let publish_dir = PublishDir::try_new(publish_dir.path().to_path_buf())?;
    let route_source = StatusListRouteSource {
        path: "/tsl",
        publish_dir,
        ttl,
    };
    let router = create_serve_router([route_source])?;
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
    assert_eq!(response.status(), StatusCode::OK);
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
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

async fn get_with_accept(url: &Url, accept: Option<&str>) -> reqwest::Response {
    let mut request = reqwest::Client::new().get(url.join("test").unwrap());
    if let Some(accept) = accept {
        request = request.header(ACCEPT, accept);
    };
    request.send().await.unwrap()
}

#[tokio::test]
#[rstest]
#[case::no_accept(None, StatusCode::OK, Some("application/statuslist+jwt"))]
#[case::jwt(
    Some("application/statuslist+jwt"),
    StatusCode::OK,
    Some("application/statuslist+jwt")
)]
#[case::cwt(
    Some("application/statuslist+cwt"),
    StatusCode::OK,
    Some("application/statuslist+cwt")
)]
#[case::cwt_preferred(
    Some("application/statuslist+cwt, application/statuslist+jwt"),
    StatusCode::OK,
    Some("application/statuslist+cwt")
)]
#[case::text(Some("text/plain"), StatusCode::NOT_ACCEPTABLE, None)]
#[case::any(Some("*/*"), StatusCode::OK, Some("application/statuslist+jwt"))]
async fn test_router_media_type(
    #[case] accept: Option<&str>,
    #[case] status_code: StatusCode,
    #[case] expected_content_type: Option<&str>,
) {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    tokio::fs::write(publish_dir.path().join("test.jwt"), "jwt")
        .await
        .unwrap();
    tokio::fs::write(publish_dir.path().join("test.cwt"), "cwt")
        .await
        .unwrap();

    let response = get_with_accept(&url, accept).await;

    assert_eq!(response.headers().get(VARY).unwrap(), "accept");
    assert_eq!(response.status(), status_code);
    if let Some(content_type) = expected_content_type {
        assert_eq!(response.headers().get(CONTENT_TYPE).unwrap(), content_type);
    }
}

#[tokio::test]
#[rstest]
#[case::no_accept(None, StatusCode::OK)]
#[case::any(Some("*/*"), StatusCode::OK)]
#[case::jwt(Some("application/statuslist+jwt"), StatusCode::OK)]
#[case::cwt(Some("application/statuslist+cwt"), StatusCode::NOT_FOUND)]
#[case::cwt_preferred(
    Some("application/statuslist+cwt, application/statuslist+jwt"),
    StatusCode::NOT_FOUND
)]
#[case::text(Some("text/plain"), StatusCode::NOT_ACCEPTABLE)]
async fn test_router_media_type_only_jwt_published(
    #[case] accept: Option<&str>,
    #[case] expected_status_code: StatusCode,
) {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    tokio::fs::write(publish_dir.path().join("test.jwt"), "jwt")
        .await
        .unwrap();

    let response = get_with_accept(&url, accept).await;

    assert_eq!(response.status(), expected_status_code);
    if expected_status_code == StatusCode::OK {
        assert_eq!(
            response.headers().get(CONTENT_TYPE).unwrap(),
            "application/statuslist+jwt"
        );
    }
}

#[tokio::test]
#[rstest]
#[case("\"7-246253568559076899098453655604172054734\"", StatusCode::NOT_MODIFIED)]
#[case("\"7-246253568559076899098453655604172054735\"", StatusCode::OK)]
async fn test_router_etag(#[case] etag: &str, #[case] status_code: StatusCode) {
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

#[tokio::test]
async fn test_router_with_extension() {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    let path = publish_dir.path().join("test.jwt");
    tokio::fs::write(&path, "test123").await.unwrap();

    let response = reqwest::get(url.join("test.jwt").unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_router_external_id_check() {
    let publish_dir = TempDir::new().unwrap();
    let url = setup_server(&publish_dir, None).await.unwrap();

    let path = publish_dir.path().join("teßt.jwt");
    tokio::fs::write(&path, "test123").await.unwrap();

    let response = reqwest::get(url.join("teßt").unwrap()).await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}
