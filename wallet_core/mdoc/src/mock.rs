use chrono::{DateTime, TimeZone, Utc};
use webpki::TrustAnchor;

use wallet_common::{generator::Generator, keys::software::SoftwareEcdsaKey};

use crate::{
    examples::{Example, Examples},
    holder::Mdoc,
    DeviceResponse,
};

/// Some of the certificates in the ISO examples are valid from Oct 1, 2020 to Oct 1, 2021.
/// This generator returns a time in that window.
pub struct IsoCertTimeGenerator;
impl Generator<DateTime<Utc>> for IsoCertTimeGenerator {
    fn generate(&self) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2021, 1, 1, 0, 0, 0).unwrap()
    }
}

/// Out of the example data structures in the standard, assemble an mdoc.
/// The issuer-signed part of the mdoc is based on a [`DeviceResponse`] in which not all attributes of the originating
/// mdoc are disclosed. Consequentially, the issuer signed-part of the resulting mdoc misses some [`IssuerSignedItem`]
/// instances, i.e. attributes.
/// This is because the other attributes are actually nowhere present in the standard so it is impossible to construct
/// the example mdoc with all attributes present.
///
/// Using tests should not rely on all attributes being present.
pub fn mdoc_from_example_device_response(trust_anchors: &[TrustAnchor<'_>]) -> Mdoc {
    // Prepare the mdoc's private key
    let static_device_key = Examples::static_device_key();
    SoftwareEcdsaKey::insert("example_static_device_key", static_device_key);

    let issuer_signed = DeviceResponse::example().documents.as_ref().unwrap()[0]
        .issuer_signed
        .clone();

    Mdoc::new::<SoftwareEcdsaKey>(
        "example_static_device_key".to_string(),
        issuer_signed,
        &IsoCertTimeGenerator,
        trust_anchors,
    )
    .unwrap()
}
