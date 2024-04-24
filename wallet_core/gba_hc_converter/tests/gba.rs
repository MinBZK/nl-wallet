use gba_hc_converter::gba::data::GbaResponse;
use rstest::rstest;

use crate::common::read_file;

mod common;

#[test]
fn test_soap_response_deserialization() {
    let voorkomens = GbaResponse::new(&read_file("gba/frouke.xml"))
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

#[test]
fn test_soap_response_single_categorievoorkomen() {
    let voorkomens = GbaResponse::new(&read_file("gba/single-categorievoorkomen.xml"))
        .unwrap()
        .categorievoorkomens;
    assert_eq!(1, voorkomens.len());

    let first = voorkomens.first().unwrap();
    assert_eq!(1, first.categorienummer);
    assert_eq!(6, first.elementen.map.len());
}

#[test]
fn test_soap_response_multiple_nationalities() {
    let voorkomens = GbaResponse::new(&read_file("gba/mulitple-nationalities.xml"))
        .unwrap()
        .categorievoorkomens;
    dbg!(&voorkomens);
    assert_eq!(7, voorkomens.len());
}

#[rstest]
#[case("gba/error.xml")]
#[case("gba/empty-response.xml")]
fn test_should_be_empty(#[case] xml_file_name: &str) {
    let voorkomens = GbaResponse::new(&read_file(xml_file_name)).unwrap().categorievoorkomens;
    assert!(voorkomens.is_empty());
}

#[rstest]
#[case("gba/error.xml")]
fn test_should_handle_error(#[case] xml_file_name: &str) {
    let response = GbaResponse::new(&read_file(xml_file_name)).unwrap();
    assert_eq!("1", response.result.code);
    assert_eq!("X", response.result.letter);
    assert_eq!("Interne fout.", response.result.description);
}
