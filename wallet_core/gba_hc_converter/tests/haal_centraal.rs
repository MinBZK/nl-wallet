use assert_json_diff::{assert_json_matches, CompareMode, Config};
use rstest::rstest;
use serde_json::Value;

use gba_hc_converter::{gba::data::GbaResponse, haal_centraal::PersonsResponse};

use crate::common::read_file;

pub mod common;

#[tokio::test]
#[rstest]
#[case("gba/frouke.xml", "haal_centraal/frouke.json")]
#[case("gba/mulitple-nationalities.xml", "haal_centraal/multiple-nationalities.json")]
#[case("gba/partner.xml", "haal_centraal/partner.json")]
#[case("gba/empty-response.xml", "haal_centraal/empty.json")]
async fn test_conversion(#[case] xml: &str, #[case] json: &str) {
    let gba_response = GbaResponse::new(&read_file(xml).await).unwrap();
    let mut personen_response = PersonsResponse::create(gba_response).unwrap();
    personen_response.filter_terminated_nationalities();

    println!("{}", serde_json::to_string_pretty(&personen_response).unwrap());

    let actual_json = serde_json::to_value(&personen_response).unwrap();
    let expected_json: Value = serde_json::from_str(&read_file(json).await).unwrap();

    assert_json_matches!(actual_json, expected_json, Config::new(CompareMode::Inclusive));
}
