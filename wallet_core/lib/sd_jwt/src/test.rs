use futures::FutureExt;

use attestation_types::claim_path::ClaimPath;
use crypto::server_keys::KeyPair;
use utils::vec_at_least::VecNonEmpty;

use crate::builder::SdJwtBuilder;
use crate::builder::SignedSdJwt;
use crate::disclosure::Disclosure;
use crate::disclosure::DisclosureContent;
use crate::hasher::Hasher;
use crate::sd_alg::SdAlg;
use crate::sd_jwt::SdJwtVcClaims;
use crate::sd_jwt::UnsignedSdJwtPresentation;
use crate::sd_jwt::VerifiedSdJwt;

pub const DIGESTS_KEY: &str = "_sd";
pub const ARRAY_DIGEST_KEY: &str = "...";

pub fn prepare_disclosure(content: DisclosureContent) -> (String, Disclosure) {
    let disclosure = Disclosure::try_new(content).unwrap();
    let hasher = SdAlg::Sha256.hasher().unwrap();
    let digest = hasher.encoded_digest(disclosure.encoded());
    (digest, disclosure)
}

pub fn object_disclosure(key: &'static str, value: serde_json::Value) -> (String, Disclosure) {
    prepare_disclosure(DisclosureContent::ObjectProperty(
        crypto::utils::random_string(16),
        key.parse().unwrap(),
        serde_json::from_value(value).unwrap(),
    ))
}

pub fn array_disclosure(value: serde_json::Value) -> (String, Disclosure) {
    prepare_disclosure(DisclosureContent::ArrayElement(
        crypto::utils::random_string(16),
        serde_json::from_value(value).unwrap(),
    ))
}

pub fn conceal_and_sign(
    issuer_keypair: &KeyPair,
    input: SdJwtVcClaims,
    claims_to_conceal: Vec<VecNonEmpty<ClaimPath>>,
) -> SignedSdJwt {
    let mut builder = SdJwtBuilder::new(input);

    for claim_to_conceal in claims_to_conceal {
        builder = builder.make_concealable(claim_to_conceal).unwrap();
    }

    builder.finish(issuer_keypair).now_or_never().unwrap().unwrap()
}

pub fn disclose_claims(
    verified_sd_jwt: VerifiedSdJwt,
    all_claims: &[VecNonEmpty<ClaimPath>],
) -> UnsignedSdJwtPresentation {
    let mut presentation_builder = verified_sd_jwt.into_presentation_builder();

    for claim_path in all_claims {
        presentation_builder = presentation_builder.disclose(claim_path).unwrap();
    }

    presentation_builder.finish()
}
