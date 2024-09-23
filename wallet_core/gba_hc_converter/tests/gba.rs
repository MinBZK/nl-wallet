use gba_hc_converter::gba::data::GbaResponse;
use rstest::rstest;

use crate::common::read_file;

pub mod common;

#[tokio::test]
async fn test_soap_response_deserialization() {
    let voorkomens = GbaResponse::new(&read_file("gba/frouke.xml").await)
        .unwrap()
        .categorievoorkomens;
    assert_eq!(3, voorkomens.len());

    let first = voorkomens.first().unwrap();
    assert_eq!(1, first.categorienummer);
    assert_eq!(8, first.elementen.map.len());

    let second = voorkomens.get(1).unwrap();
    assert_eq!(4, second.categorienummer);
    assert_eq!(4, second.elementen.map.len());

    let third = voorkomens.get(2).unwrap();
    assert_eq!(8, third.categorienummer);
    assert_eq!(6, third.elementen.map.len());
}

#[tokio::test]
async fn test_soap_response_single_categorievoorkomen() {
    let voorkomens = GbaResponse::new(&read_file("gba/single-categorievoorkomen.xml").await)
        .unwrap()
        .categorievoorkomens;
    assert_eq!(1, voorkomens.len());

    let first = voorkomens.first().unwrap();
    assert_eq!(1, first.categorienummer);
    assert_eq!(6, first.elementen.map.len());
}

#[tokio::test]
async fn test_soap_response_multiple_nationalities() {
    let voorkomens = GbaResponse::new(&read_file("gba/mulitple-nationalities.xml").await)
        .unwrap()
        .categorievoorkomens;
    dbg!(&voorkomens);
    assert_eq!(7, voorkomens.len());
}

#[tokio::test]
#[rstest]
#[case("gba/error.xml")]
#[case("gba/empty-response.xml")]
async fn test_should_be_empty(#[case] xml_file_name: &str) {
    let voorkomens = GbaResponse::new(&read_file(xml_file_name).await)
        .unwrap()
        .categorievoorkomens;
    assert!(voorkomens.is_empty());
}

#[tokio::test]
#[rstest]
#[case("gba/error.xml")]
async fn test_should_handle_error(#[case] xml_file_name: &str) {
    let response = GbaResponse::new(&read_file(xml_file_name).await).unwrap();
    assert_eq!("1", response.result.code);
    assert_eq!("X", response.result.letter);
    assert_eq!("Interne fout.", response.result.description);
}
