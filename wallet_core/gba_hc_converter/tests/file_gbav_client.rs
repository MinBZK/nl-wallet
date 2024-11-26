use gba_hc_converter::gba::client::FileGbavClient;
use gba_hc_converter::gba::client::GbavClient;
use gba_hc_converter::gba::data::GbaResponse;
use gba_hc_converter::gba::error::Error;
use gba_hc_converter::haal_centraal::Bsn;
use gba_hc_converter::haal_centraal::Element;

use crate::common::encrypt_xmls;
use crate::common::read_file;

pub mod common;

struct EmptyGbavClient {}

impl GbavClient for EmptyGbavClient {
    async fn vraag(&self, _bsn: &Bsn) -> Result<Option<String>, Error> {
        Ok(Some(read_file("gba/empty-response.xml").await))
    }
}

#[tokio::test]
async fn should_return_preloaded_xml() {
    let (encryption_key, hmac_key, dir) = encrypt_xmls().await;

    let client = FileGbavClient::new(dir.path(), encryption_key, hmac_key, EmptyGbavClient {});
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
    let (encryption_key, hmac_key, dir) = encrypt_xmls().await;

    let client = FileGbavClient::new(dir.path(), encryption_key, hmac_key, EmptyGbavClient {});
    let response = client.vraag(&Bsn::try_new("11122146").unwrap()).await.unwrap().unwrap();
    let gba_response = GbaResponse::new(&response).unwrap();
    assert!(gba_response.is_empty());
}
