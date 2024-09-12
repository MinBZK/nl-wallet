use std::path::Path;

use gba_hc_converter::{
    gba::{
        client::{FileGbavClient, GbavClient},
        data::GbaResponse,
        error::Error,
    },
    haal_centraal::{Bsn, Element},
};

use crate::common::read_file;

mod common;

struct EmptyGbavClient {}

impl GbavClient for EmptyGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<GbaResponse, Error> {
        GbaResponse::new(&read_file("gba/empty-response.xml"))
    }
}

#[tokio::test]
async fn should_return_preloaded_xml() {
    let client = FileGbavClient::new(Path::new("tests/resources/gba"), EmptyGbavClient {});
    let response = client.vraag(&Bsn::try_new("999991772").unwrap()).await.unwrap();
    assert_eq!(
        "Froukje",
        &response.categorievoorkomens[0]
            .elementen
            .get_mandatory(Element::Voornamen.code())
            .unwrap()
    );
}

#[tokio::test]
async fn should_return_empty() {
    let client = FileGbavClient::new(Path::new("tests/resources/gba"), EmptyGbavClient {});
    let response = client.vraag(&Bsn::try_new("12345678").unwrap()).await.unwrap();
    assert!(response.categorievoorkomens.is_empty());
}
