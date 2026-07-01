use std::sync::LazyLock;

use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::CertificateUsage;
use derive_more::Constructor;
use http_utils::urls::BaseUrl;
use jsonwebtoken::Validation;
use jsonwebtoken::errors::ErrorKind;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;
use utils::generator::TimeGenerator;

use crate::DEFAULT_VALIDATIONS;
use crate::EcdsaDecodingKey;
use crate::JwtTyp;
use crate::UnverifiedJwt;
use crate::confirmation::ConfirmationClaim;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::error::JwtX5cError;
use crate::headers::HeaderWithX5c;
use crate::nonce::Nonce;

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
    #[expect(clippy::too_many_arguments, reason = "constructor")]
    pub fn new(
        holder_pubkey: &VerifyingKey,
        iss: String,
        sub: String,
        exp: DateTimeSeconds,
        wallet_info: WiaWalletInfo,
        client_status: ClientStatus,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Self, JwtError> {
        let now = time.generate().into();
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

pub const WIA_HEADER_NAME: &str = "oauth-client-attestation";
pub const WIA_POP_HEADER_NAME: &str = "oauth-client-attestation-pop";

pub const WIA_JWT_TYP: &str = "oauth-client-attestation+jwt";
pub const WIA_POP_JWT_TYP: &str = "oauth-client-attestation-pop+jwt";

impl JwtTyp for WiaClaims {
    const TYP: &'static str = WIA_JWT_TYP;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WiaPopClaims {
    pub iss: String,
    pub aud: String,
    pub iat: DateTimeSeconds,
    pub jti: String,
    pub challenge: Option<Nonce>,
}

pub type Wia = UnverifiedJwt<WiaClaims, HeaderWithX5c>;
pub type WiaPop = UnverifiedJwt<WiaPopClaims>;

impl JwtTyp for WiaPopClaims {
    const TYP: &str = WIA_POP_JWT_TYP;
}

#[derive(Clone, Debug, Serialize, Deserialize, Constructor)]
pub struct WiaDisclosure(Wia, WiaPop);

impl WiaDisclosure {
    pub fn wia(&self) -> &UnverifiedJwt<WiaClaims, HeaderWithX5c> {
        &self.0
    }

    pub fn wia_pop(&self) -> &UnverifiedJwt<WiaPopClaims> {
        &self.1
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WiaError {
    #[error("incorrect nonce")]
    IncorrectNonce,
    #[error("JWK conversion error: {0}")]
    JwkConversion(#[source] JwkConversionError),
    #[error("JWT error: {0}")]
    Jwt(#[source] JwtError),
    #[error("JWT with certificate error: {0}")]
    JwtX5c(#[source] JwtX5cError),
    #[error("incorrect sub field in WIA: found '{0}', expected '{1}'")]
    IncorrectSub(String, String),
    #[error("WIA has expired")]
    Expired,
}

impl WiaDisclosure {
    pub fn verify(
        &self,
        trust_anchors: &TrustAnchors,
        expected_aud: &str,
        accepted_wallet_client_ids: &[String],
        expected_challenge: Option<&Nonce>,
        client_id: Option<&String>,
    ) -> Result<VerifyingKey, WiaError> {
        let (_, verified_wia_claims) = self
            .0
            .parse_and_verify_against_trust_anchors(
                trust_anchors,
                &TimeGenerator,
                CertificateUsage::Wia,
                &WIA_JWT_VALIDATIONS,
            )
            .map_err(|err| match err {
                JwtX5cError::Jwt(JwtError::Validation(e)) if matches!(e.kind(), ErrorKind::ExpiredSignature) => {
                    WiaError::Expired
                }
                _ => WiaError::JwtX5c(err),
            })?;

        // "If a client_id is provided in the request containing the Client Attestation, then this client_id
        // matches the sub claim of the Client Attestation JWT."
        if let Some(client_id) = client_id
            && verified_wia_claims.sub != *client_id
        {
            return Err(WiaError::IncorrectSub(verified_wia_claims.sub, client_id.clone()));
        }

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
            .parse_and_verify(EcdsaDecodingKey::from(&wia_pubkey), &validations)
            .map_err(WiaError::Jwt)?;

        if wia_disclosure_claims.challenge.as_ref() != expected_challenge {
            return Err(WiaError::IncorrectNonce);
        }

        Ok(wia_pubkey)
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
    use std::assert_matches;

    use attestation_types::status_claim::StatusClaim;
    use chrono::DateTime;
    use chrono::Duration;
    use chrono::TimeDelta;
    use chrono::Utc;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use futures::FutureExt;
    use p256::ecdsa::SigningKey;
    use p256::ecdsa::VerifyingKey;
    use rand_core::OsRng;
    use rstest::fixture;
    use rstest::rstest;
    use utils::generator::Generator;
    use utils::generator::mock::MockTimeGenerator;

    use crate::SignedJwt;
    use crate::UnverifiedJwt;
    use crate::error::JwtError;
    use crate::error::JwtX5cError;
    use crate::headers::HeaderWithX5c;
    use crate::nonce::Nonce;
    use crate::wia::ClientStatus;
    use crate::wia::WiaClaims;
    use crate::wia::WiaDisclosure;
    use crate::wia::WiaError;
    use crate::wia::WiaPopClaims;
    use crate::wia::WiaWalletInfo;

    const AUD: &str = "https://issuer.example.com/";
    const ISS: &str = "https://wia-issuer.example.com/";
    const WALLET_CLIENT_ID: &str = "wallet-client";

    #[fixture]
    fn ca() -> Ca {
        Ca::generate("wia.ca.example.com", Default::default()).unwrap()
    }

    #[fixture]
    fn holder_key() -> SigningKey {
        SigningKey::random(&mut OsRng)
    }

    fn make_wia(
        wia_keypair: &KeyPair,
        holder_pubkey: &VerifyingKey,
        time: &impl Generator<DateTime<Utc>>,
    ) -> UnverifiedJwt<WiaClaims, HeaderWithX5c> {
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
            time,
        )
        .unwrap();

        SignedJwt::sign_with_certificate(&wia_claims, wia_keypair)
            .now_or_never()
            .unwrap()
            .unwrap()
            .into()
    }

    fn make_pop(holder_key: &SigningKey, nonce: Option<Nonce>, aud: &str) -> UnverifiedJwt<WiaPopClaims> {
        SignedJwt::sign(
            &WiaPopClaims {
                iss: WALLET_CLIENT_ID.to_string(),
                aud: aud.to_string(),
                iat: Utc::now().into(),
                jti: "jti".to_string(),
                challenge: nonce,
            },
            holder_key,
        )
        .now_or_never()
        .unwrap()
        .unwrap()
        .into()
    }

    fn make_wia_disclosure(ca: &Ca, holder_key: &SigningKey, nonce: Option<Nonce>) -> WiaDisclosure {
        let wia_keypair = ca.generate_wia_mock().unwrap();
        WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key(), &MockTimeGenerator::default()),
            make_pop(holder_key, nonce, AUD),
        )
    }

    #[rstest]
    fn verify_valid(ca: Ca, holder_key: SigningKey, #[values(Some(Nonce::new_random()), None)] nonce: Option<Nonce>) {
        let disclosure = make_wia_disclosure(&ca, &holder_key, nonce.clone());
        disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                nonce.as_ref(),
                None,
            )
            .unwrap();
    }

    #[rstest]
    fn verify_pop_signed_with_wrong_key(ca: Ca, holder_key: SigningKey) {
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let wrong_key = SigningKey::random(&mut OsRng);
        let nonce = Nonce::new_random();
        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key(), &MockTimeGenerator::default()),
            make_pop(&wrong_key, Some(nonce.clone()), AUD),
        );
        let error = disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                Some(&nonce),
                None,
            )
            .unwrap_err();
        assert_matches!(error, WiaError::Jwt(_));
    }

    #[rstest]
    fn verify_pop_wrong_audience(ca: Ca, holder_key: SigningKey) {
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let nonce = Nonce::new_random();
        let disclosure = WiaDisclosure::new(
            make_wia(&wia_keypair, holder_key.verifying_key(), &MockTimeGenerator::default()),
            make_pop(&holder_key, Some(nonce.clone()), "https://wrong.example.com/"),
        );
        let error = disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                Some(&nonce),
                None,
            )
            .unwrap_err();
        assert_matches!(error, WiaError::Jwt(_));
    }

    #[rstest]
    fn verify_unaccepted_wallet_client_id(ca: Ca, holder_key: SigningKey) {
        let nonce = Nonce::new_random();
        let disclosure = make_wia_disclosure(&ca, &holder_key, Some(nonce.clone()));
        let error = disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &["other-client".to_string()],
                Some(&nonce),
                None,
            )
            .unwrap_err();
        assert_matches!(error, WiaError::Jwt(_));
    }

    #[rstest]
    fn verify_missing_nonce(ca: Ca, holder_key: SigningKey) {
        let disclosure = make_wia_disclosure(&ca, &holder_key, None);
        let error = disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                Some(&Nonce::new_random()),
                None,
            )
            .unwrap_err();
        assert_matches!(error, WiaError::IncorrectNonce);
    }

    #[rstest]
    fn verify_wia_not_yet_valid(ca: Ca, holder_key: SigningKey) {
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let nonce = Nonce::new_random();
        let disclosure = WiaDisclosure::new(
            make_wia(
                &wia_keypair,
                holder_key.verifying_key(),
                &MockTimeGenerator::new(Utc::now() + TimeDelta::weeks(1)), // WIA will be valid in a week from now
            ),
            make_pop(&holder_key, Some(nonce.clone()), AUD),
        );
        let error = disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                Some(&nonce),
                None,
            )
            .unwrap_err();
        assert_matches!(
            error,
            WiaError::JwtX5c(JwtX5cError::Jwt(JwtError::Validation(error))) if *error.kind() == jsonwebtoken::errors::ErrorKind::ImmatureSignature
        );
    }

    #[rstest]
    fn verify_correct_client_id(ca: Ca, holder_key: SigningKey) {
        let disclosure = make_wia_disclosure(&ca, &holder_key, None);
        disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                None,
                Some(&WALLET_CLIENT_ID.to_string()),
            )
            .unwrap();
    }

    #[rstest]
    fn verify_incorrect_client_id(ca: Ca, holder_key: SigningKey) {
        let disclosure = make_wia_disclosure(&ca, &holder_key, None);
        let error = disclosure
            .verify(
                &TrustAnchors::from(&ca),
                AUD,
                &[WALLET_CLIENT_ID.to_string()],
                None,
                Some(&"wrong-client-id".to_string()),
            )
            .unwrap_err();
        assert_matches!(error, WiaError::IncorrectSub(found, expected)
            if found == WALLET_CLIENT_ID && expected == "wrong-client-id");
    }
}
