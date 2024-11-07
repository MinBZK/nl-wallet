use std::sync::LazyLock;

use const_decoder::Pem;
use webpki::{
    self,
    types::{CertificateDer, TrustAnchor},
};

// Source: https://www.apple.com/certificateauthority/Apple_App_Attestation_Root_CA.pem
const APPLE_ROOT_CA: [u8; 549] = Pem::decode(include_bytes!("../assets/Apple_App_Attestation_Root_CA.pem"));

pub static APPLE_TRUST_ANCHORS: LazyLock<Vec<TrustAnchor>> = LazyLock::new(|| {
    let cert = Box::new(CertificateDer::from(APPLE_ROOT_CA.as_slice()));

    // As this happens at most once, leaking the `CertificateDer` to make it static should
    // be acceptable, as is panicking if this hardcoded certificate fails to parse.
    let trust_anchor = webpki::anchor_from_trusted_cert(Box::leak(cert)).unwrap();

    vec![trust_anchor]
});
