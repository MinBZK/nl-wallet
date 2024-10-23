use std::{
    net::{IpAddr, TcpListener},
    str::FromStr,
};

use assert_json_diff::{assert_json_matches, CompareMode, Config};
use ctor::ctor;
use http::StatusCode;
use reqwest::Response;
use serde_json::{json, Value};

use gba_hc_converter::{
    gba::{client::GbavClient, error::Error},
    haal_centraal::{Bsn, Element, PersonQuery, PersonsResponse},
    server,
};
use tests_integration::common::wait_for_server;
use wallet_common::reqwest::default_reqwest_client_builder;

use crate::common::read_file;

pub mod common;

#[ctor]
fn init_logging() {
    let _ = tracing::subscriber::set_global_default(
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_test_writer()
            .finish(),
    );
}

fn find_listener_port() -> u16 {
    TcpListener::bind("localhost:0")
        .expect("Could not find TCP port")
        .local_addr()
        .expect("Could not get local address from TCP listener")
        .port()
}

async fn start_server_with_mock<T>(gbav_client: T) -> u16
where
    T: GbavClient + Send + Sync + 'static,
{
    let port = find_listener_port();

    tokio::spawn(async move {
        server::serve(IpAddr::from_str("0.0.0.0").unwrap(), port, gbav_client)
            .await
            .unwrap();
    });

    wait_for_server(format!("http://localhost:{port}").parse().unwrap(), vec![]).await;
    port
}

async fn query_personen(port: u16) -> Response {
    let query = PersonQuery {
        r#type: String::from("RaadpleegMetBurgerservicenummer"),
        bsn: vec![Bsn::try_new("11122146").unwrap()],
        registration_municipality: None,
        fields: vec![],
    };

    let client = default_reqwest_client_builder().build().unwrap();
    client
        .post(format!("http://localhost:{port}/haalcentraal/api/brp/personen"))
        .json(&query)
        .send()
        .await
        .unwrap()
}

struct MockGbavClient {
    xml_file: String,
}

impl GbavClient for MockGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<Option<String>, Error> {
        Ok(Some(read_file(&self.xml_file).await))
    }
}

struct ErrorGbavClient {}
impl GbavClient for ErrorGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<Option<String>, Error> {
        Err(Error::MissingElement(Element::Nationality.code()))
    }
}

#[tokio::test]
async fn test_ok_response() {
    let port = start_server_with_mock(MockGbavClient {
        xml_file: String::from("gba/frouke.xml"),
    })
    .await;

    let response = query_personen(port).await;
    assert_eq!(StatusCode::OK, response.status());

    let json: PersonsResponse = response.json().await.unwrap();
    assert_eq!(1, json.persons.len());
}

#[tokio::test]
async fn test_error_response() {
    let port = start_server_with_mock(ErrorGbavClient {}).await;

    let response = query_personen(port).await;
    assert_eq!(StatusCode::PRECONDITION_FAILED, response.status());
    assert_eq!(
        "application/problem+json",
        response.headers().get("Content-Type").unwrap().to_str().unwrap()
    );

    assert_json_matches!(
        serde_json::from_str::<Value>(&response.text().await.unwrap()).unwrap(),
        json!({
            "type": "gba",
            "title": "GBA error",
            "status": 412,
            "detail": "GBA error: Element number 510 is mandatory but missing"
        }),
        Config::new(CompareMode::Inclusive)
    );
}

#[tokio::test]
async fn test_received_error_response() {
    let port = start_server_with_mock(MockGbavClient {
        xml_file: String::from("gba/error.xml"),
    })
    .await;

    let response = query_personen(port).await;
    assert_eq!(StatusCode::PRECONDITION_FAILED, response.status());
    assert_eq!(
        "application/problem+json",
        response.headers().get("Content-Type").unwrap().to_str().unwrap()
    );
    assert_json_matches!(
        serde_json::from_str::<Value>(&response.text().await.unwrap()).unwrap(),
        json!({
            "type": "gba",
            "title": "GBA error",
            "status": 412,
            "detail":
                "GBA error: Received error response: foutcode: X001, description: Interne fout., reference: \
                 a00d961b-dd58-4f1c-bd48-964a46d2708b"
        }),
        Config::new(CompareMode::Inclusive)
    );
}
