use std::sync::LazyLock;

use attestation_types::status_claim::StatusClaim;
use chrono::Utc;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateUsage;
use derive_more::Constructor;
use http_utils::urls::BaseUrl;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::TimeGenerator;

use crate::DEFAULT_VALIDATIONS;
use crate::JwtTyp;
use crate::UnverifiedJwt;
use crate::confirmation::ConfirmationClaim;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::error::JwtX5cError;
use crate::headers::HeaderWithX5c;
use crate::nonce::Nonce;
use crate::pop::JwtPopClaims;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WiaClaims {
    pub cnf: ConfirmationClaim,

    // Standard JWT fields.
    pub iss: String,
    pub sub: String,
    pub exp: DateTimeSeconds,
    pub iat: Option<DateTimeSeconds>,
    pub nbf: Option<DateTimeSeconds>,

    #[serde(flatten)]
    pub wallet_info: WiaWalletInfo,

    pub client_status: ClientStatus,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WiaWalletInfo {
    pub wallet_name: String,
    pub wallet_version: String,
    #[serde(default)]
    pub wallet_link: Option<BaseUrl>,

    // The structure (and therefore) type of this field is not yet defined, but the example in TS3 shows a string.
    pub wallet_solution_certification_information: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ClientStatus {
    // Revocation status of the Wallet Instance that presented the WIA.
    pub status: StatusClaim,

    // The duration for which the WP will track revocation status in the `status` URL.
    // (Distinct in terms of semantics as well as value from the top level WIA `exp` claim, which is max 24h.)
    pub exp: DateTimeSeconds,
}

impl WiaClaims {
    pub fn new(
        holder_pubkey: &VerifyingKey,
        iss: String,
        sub: String,
        exp: DateTimeSeconds,
        wallet_info: WiaWalletInfo,
        client_status: ClientStatus,
    ) -> Result<Self, JwtError> {
        let now = Utc::now().into();

        Ok(Self {
            cnf: ConfirmationClaim::from_verifying_key(holder_pubkey)?,
            iss,
            sub,
            exp,
            iat: Some(now),
            nbf: Some(now),
            wallet_info,
            client_status,
        })
    }
}

pub const WIA_JWT_TYP: &str = "oauth-client-attestation+jwt";

impl JwtTyp for WiaClaims {
    const TYP: &'static str = WIA_JWT_TYP;
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WiaDisclosure(UnverifiedJwt<WiaClaims, HeaderWithX5c>, UnverifiedJwt<JwtPopClaims>);

#[cfg(feature = "test")]
impl WiaDisclosure {
    pub fn wia(&self) -> &UnverifiedJwt<WiaClaims, HeaderWithX5c> {
        &self.0
    }

    pub fn wia_pop(&self) -> &UnverifiedJwt<JwtPopClaims> {
        &self.1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WiaError {
    #[error("nonce is missing from WIA payload")]
    MissingNonce,
    #[error("JWK conversion error: {0}")]
    JwkConversion(#[source] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[source] JwtError),
    #[error("JWT with certificate error: {0}")]
    JwtX5c(#[source] JwtX5cError),
}

impl WiaDisclosure {
    pub fn verify(
        &self,
        trust_anchors: &TrustAnchors,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
    ) -> Result<(VerifyingKey, Nonce), WiaError> {
        let (_, verified_wia_claims) = self
            .0
            .parse_and_verify_against_trust_anchors(
                trust_anchors,
                &TimeGenerator,
                CertificateUsage::Wia,
                &WIA_JWT_VALIDATIONS,
            )
            .map_err(WiaError::JwtX5c)?;
        let wia_pubkey = verified_wia_claims
            .cnf
            .verifying_key()
            .map_err(WiaError::JwkConversion)?;
        tracing::debug!("WIA status claim: {:?}", verified_wia_claims.client_status.status);

        let mut validations = DEFAULT_VALIDATIONS.to_owned();
        validations.set_audience(&[expected_aud]);
        validations.set_issuer(accepted_wallet_client_ids);
        let (_, wia_disclosure_claims) = self
            .1
            .parse_and_verify(&(&wia_pubkey).into(), &validations)
            .map_err(WiaError::Jwt)?;

        let nonce = wia_disclosure_claims.nonce.ok_or(WiaError::MissingNonce)?;

        Ok((wia_pubkey, nonce))
    }
}

// Returns the JWS validations for WIA verification.
//
// NOTE: the returned validation allows for no clock drift: time-based claims such as `exp` are validated
// without leeway. There must be no clock drift between the WIA issuer and the caller.
pub static WIA_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = DEFAULT_VALIDATIONS.to_owned();
    validations.leeway = 0;

    // Enforce validity of exp and nbf, and presence of exp.
    // (nbf is optional, but if it is present, it needs to be valid.)
    validations.set_required_spec_claims(&["exp"]);
    validations.validate_exp = true;
    validations.validate_nbf = true;

    validations
});

#[cfg(any(test, feature = "mock"))]
mod mock {
    use crate::wia::WiaWalletInfo;

    impl WiaWalletInfo {
        pub fn new_mock() -> WiaWalletInfo {
            WiaWalletInfo {
                wallet_name: "Test Wallet".to_string(),
                wallet_version: "1.0.0".to_string(),
                wallet_link: None,
                wallet_solution_certification_information: "https://cert.example.com".to_string(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use attestation_types::status_claim::StatusClaim;
    use chrono::Duration;
    use chrono::Utc;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand_core::OsRng;

    use crate::SignedJwt;
    use crate::UnverifiedJwt;
    use crate::headers::HeaderWithX5c;
    use crate::nonce::Nonce;
    use crate::pop::JwtPopClaims;
    use crate::wia::ClientStatus;
    use crate::wia::WiaClaims;
    use crate::wia::WiaDisclosure;
    use crate::wia::WiaError;
    use crate::wia::WiaWalletInfo;

    const AUD: &str = "https://issuer.example.com/";
    const ISS: &str = "https://wia-issuer.example.com/";
    const WALLET_CLIENT_ID: &str = "wallet-client";

    fn make_wia(wia_keypair: &KeyPair, holder_pubkey: &VerifyingKey) -> UnverifiedJwt<WiaClaims, HeaderWithX5c> {
        let wia_claims = WiaClaims::new(
            holder_pubkey,
            ISS.to_string(),
            WALLET_CLIENT_ID.to_string(),
            (Utc::now() + Duration::hours(1)).into(),
            WiaWalletInfo::new_mock(),
            ClientStatus {
                status: StatusClaim::new_mock(),
                exp: (Utc::now() + Duration::days(365)).into(),
            },
        )
        .unwrap();

        SignedJwt::sign_with_certificate(&wia_claims, wia_keypair)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into()
    }

    fn make_pop(holder_key: &SigningKey, nonce: Option<Nonce>, aud: &str) -> UnverifiedJwt<JwtPopClaims> {
        SignedJwt::sign(
            &JwtPopClaims::new(nonce, WALLET_CLIENT_ID.to_string(), aud.to_string()),
            holder_key,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
        .into()
    }

    #[test]
    fn verify_valid() {
        let ca = Ca::generate("wia.ca.example.com", Default::default()).unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);

        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key()),
            make_pop(&holder_key, Some(Nonce::new_random()), AUD),
        );

        let (_, nonce) = disclosure
            .verify(&TrustAnchors::from(&ca), AUD, &[WALLET_CLIENT_ID.to_string()])
            .unwrap();

        assert!(!nonce.as_ref().is_empty());
    }

    #[test]
    fn verify_pop_signed_with_wrong_key() {
        let ca = Ca::generate("wia.ca.example.com", Default::default()).unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);
        let wrong_key = SigningKey::random(&mut OsRng);

        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key()),
            make_pop(&wrong_key, Some(Nonce::new_random()), AUD),
        );

        let error = disclosure
            .verify(&TrustAnchors::from(&ca), AUD, &[WALLET_CLIENT_ID.to_string()])
            .unwrap_err();

        assert_matches!(error, WiaError::Jwt(_));
    }

    #[test]
    fn verify_pop_wrong_audience() {
        let ca = Ca::generate("wia.ca.example.com", Default::default()).unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);

        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key()),
            make_pop(&holder_key, Some(Nonce::new_random()), "https://wrong.example.com/"),
        );

        let error = disclosure
            .verify(&TrustAnchors::from(&ca), AUD, &[WALLET_CLIENT_ID.to_string()])
            .unwrap_err();

        assert_matches!(error, WiaError::Jwt(_));
    }

    #[test]
    fn verify_unaccepted_wallet_client_id() {
        let ca = Ca::generate("wia.ca.example.com", Default::default()).unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);

        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key()),
            make_pop(&holder_key, Some(Nonce::new_random()), AUD),
        );

        let error = disclosure
            .verify(&TrustAnchors::from(&ca), AUD, &["other-client".to_string()])
            .unwrap_err();

        assert_matches!(error, WiaError::Jwt(_));
    }

    #[test]
    fn verify_missing_nonce() {
        let ca = Ca::generate("wia.ca.example.com", Default::default()).unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let holder_key = SigningKey::random(&mut OsRng);

        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key()),
            make_pop(&holder_key, None, AUD),
        );

        let error = disclosure
            .verify(&TrustAnchors::from(&ca), AUD, &[WALLET_CLIENT_ID.to_string()])
            .unwrap_err();

        assert_matches!(error, WiaError::MissingNonce);
    }
}
