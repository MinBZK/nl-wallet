use gba_hc_converter::{
    gba::{
        client::{FileGbavClient, GbavClient},
        data::GbaResponse,
        error::Error,
    },
    haal_centraal::{Bsn, Element},
};

use crate::common::{encrypt_xmls, read_file};

pub mod common;

struct EmptyGbavClient {}

impl GbavClient for EmptyGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<Option<String>, Error> {
        Ok(Some(read_file("gba/empty-response.xml").await))
    }
}

#[tokio::test]
async fn should_return_preloaded_xml() {
    let (key, dir) = encrypt_xmls().await;

    let client = FileGbavClient::new(dir.path(), key, EmptyGbavClient {});
    let response = client
        .vraag(&Bsn::try_new("999991772").unwrap())
        .await
        .unwrap()
        .unwrap();

    let gba_response = GbaResponse::new(&response).unwrap();

    assert_eq!(
        "Froukje",
        &gba_response.categorievoorkomens[0]
            .elementen
            .get_mandatory(Element::Voornamen.code())
            .unwrap()
    );
}

#[tokio::test]
async fn should_return_empty() {
    let (key, dir) = encrypt_xmls().await;

    let client = FileGbavClient::new(dir.path(), key, EmptyGbavClient {});
    let response = client.vraag(&Bsn::try_new("12345678").unwrap()).await.unwrap().unwrap();
    let gba_response = GbaResponse::new(&response).unwrap();
    assert!(gba_response.is_empty());
}
