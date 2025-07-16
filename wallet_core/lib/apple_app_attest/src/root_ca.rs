use std::sync::LazyLock;

use const_decoder::Pem;
use const_decoder::decode;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;

// Source: https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem
pub const APPLE_ROOT_CA: &[u8] = &decode!(Pem, include_bytes!("../assets/Apple_App_Attestation_Root_CA.pem"));

// Valid until: Nov 28 13:49:46 2124 GMT
// Generated with the following command:
// openssl req -subj "/C=NL/CN=Mock Apple App Attestation Root CA" -nodes -x509 -sha384 -days 36524 \
// -newkey ec -pkeyopt ec_paramgen_curve:secp384r1 -keyout mock_ca.key.pem -out mock_ca.crt.pem
#[cfg(feature = "mock_ca_root")]
pub const MOCK_APPLE_ROOT_CA: &[u8] = &decode!(Pem, include_bytes!("../assets/mock_ca.crt.pem"));

#[cfg(feature = "mock_ca_root")]
pub const MOCK_APPLE_ROOT_CA_KEY: &[u8] = &decode!(Pem, include_bytes!("../assets/mock_ca.key.pem"));

fn static_trust_anchors(der: &[u8]) -> Vec<TrustAnchor> {
    let certificate = Box::new(CertificateDer::from(der));

    // As this happens at most once, leaking the `CertificateDer` to make it static should
    // be acceptable, as is panicking if this hardcoded certificate fails to parse.
    let trust_anchor = webpki::anchor_from_trusted_cert(Box::leak(certificate)).unwrap();

    vec![trust_anchor]
}

pub static APPLE_TRUST_ANCHORS: LazyLock<Vec<TrustAnchor>> = LazyLock::new(|| static_trust_anchors(APPLE_ROOT_CA));

#[cfg(feature = "mock_ca_root")]
pub static MOCK_APPLE_TRUST_ANCHORS: LazyLock<Vec<TrustAnchor>> =
    LazyLock::new(|| static_trust_anchors(MOCK_APPLE_ROOT_CA));
