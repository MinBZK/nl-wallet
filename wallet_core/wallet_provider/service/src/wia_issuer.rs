use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use crypto::keys::SecureEcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::x509::CertificateError;
use derive_more::Constructor;
use hsm::keys::HsmEcdsaKey;
use jwt::SignedJwt;
use jwt::error::JwkConversionError;
use jwt::error::JwtSignError;
use jwt::wia::ClientStatus;
use jwt::wia::WiaClaims;
use jwt::wia::WiaDisclosure;
use jwt::wia::WiaPopClaims;
use jwt::wia::WiaWalletInfo;
use p256::ecdsa::SigningKey;
use p256::ecdsa::VerifyingKey;
use rand_core::OsRng;
use utils::date_time_seconds::DateTimeSeconds;
use utils::generator::Generator;

// used as the identifier for a WIA specific token status list
pub const WIA_ATTESTATION_TYPE_IDENTIFIER: &str = "wia";

/// How long issued WIAs will be valid (the token itself, not the wallet it represents).
const WIA_VALIDITY: Duration = Duration::hours(10);

#[derive(Constructor)]
pub struct HsmWiaIssuer<K = HsmEcdsaKey> {
    keypair: KeyPair<K>,
    sub: String,
    wallet_info: WiaWalletInfo,
}

#[derive(Debug, thiserror::Error, strum::IntoStaticStr)]
pub enum HsmWiaIssuerError {
    #[error("JWK conversion error: {0}")]
    KeyConversion(#[source] JwkConversionError),

    #[error("sign error: {0}")]
    SignError(#[source] JwtSignError),

    #[error("Missing Common Name in WIA issuance certificate")]
    MissingCommonName,

    #[error("WIA issuance certificate error: {0}")]
    WiaCertificateError(#[source] CertificateError),

    #[error("signing PoP failed: {0}")]
    PopSignError(#[source] JwtSignError),
}

impl<K> HsmWiaIssuer<K>
where
    K: SecureEcdsaKey,
{
    pub async fn issue_wia(
        &self,
        wallet_exp: DateTimeSeconds,
        status_claim: StatusClaim,
        pop_claims: &WiaPopClaims,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<WiaDisclosure, HsmWiaIssuerError> {
        let wia_exp = (time.generate() + WIA_VALIDITY).into();
        let iss = self
            .keypair
            .certificate()
            .common_name()
            .map_err(HsmWiaIssuerError::WiaCertificateError)?
            .ok_or(HsmWiaIssuerError::MissingCommonName)?
            .to_string();

        // The WIA private key is not persisted in the WP database: it is generated here, used here, and forgotten
        // immediately afterwards. There is therefore no need to protect it by generating it in the HSM, so
        // we use an ordinary in-memory private key here.
        let wia_privkey = SigningKey::random(&mut OsRng);

        let wia = SignedJwt::sign_with_certificate(
            &WiaClaims::new(
                wia_privkey.verifying_key(),
                iss,
                self.sub.clone(),
                wia_exp,
                self.wallet_info.clone(),
                ClientStatus {
                    status: status_claim,
                    exp: wallet_exp,
                },
                time,
            )
            .map_err(HsmWiaIssuerError::KeyConversion)?,
            &self.keypair,
        )
        .await
        .map_err(HsmWiaIssuerError::SignError)?
        .into();

        let wia_pop = SignedJwt::sign(pop_claims, &wia_privkey)
            .await
            .map_err(HsmWiaIssuerError::PopSignError)?
            .into();

        Ok(WiaDisclosure::new(wia, wia_pop))
    }

    pub fn public_key(&self) -> VerifyingKey {
        *self.keypair.certificate().public_key()
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use crypto::server_keys::generate::Ca;
    use crypto::x509::DistinguishedName;
    use jwt::wia::WiaWalletInfo;
    use p256::ecdsa::SigningKey;

    use crate::wia_issuer::HsmWiaIssuer;

    pub type MockWiaIssuer = HsmWiaIssuer<SigningKey>;

    impl MockWiaIssuer {
        pub fn new_mock() -> Self {
            Self {
                keypair: Ca::generate(DistinguishedName::create_mock("wia.ca.example.com"), Default::default())
                    .unwrap()
                    .generate_wia_mock()
                    .unwrap(),
                sub: "sub".to_string(),
                wallet_info: WiaWalletInfo::new_mock(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use attestation_types::status_claim::StatusClaim;
    use chrono::Utc;
    use crypto::server_keys::generate::Ca;
    use crypto::trust_anchor::TrustAnchors;
    use crypto::x509::CertificateUsage;
    use jwt::wia::WIA_JWT_VALIDATIONS;
    use jwt::wia::WiaPopClaims;
    use jwt::wia::WiaWalletInfo;
    use utils::generator::TimeGenerator;
    use utils::generator::mock::MockTimeGenerator;

    use super::HsmWiaIssuer;

    #[tokio::test]
    async fn it_works() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let sub = "sub";

        let wia_issuer = HsmWiaIssuer::new(wia_keypair, sub.to_string(), WiaWalletInfo::new_mock());

        let pop_claims = WiaPopClaims {
            iss: "wallet".to_string(),
            aud: "verifier".to_string(),
            iat: Utc::now().into(),
            jti: "jti".to_string(),
            challenge: None,
        };

        let wia_disclosure = wia_issuer
            .issue_wia(
                (Utc::now() + Duration::from_secs(600)).into(),
                StatusClaim::new_mock(),
                &pop_claims,
                &MockTimeGenerator::default(),
            )
            .await
            .unwrap();

        let (_, wia_claims) = wia_disclosure
            .wia()
            .parse_and_verify_against_trust_anchors(
                &TrustAnchors::from(&ca),
                &TimeGenerator,
                CertificateUsage::Wia,
                &WIA_JWT_VALIDATIONS,
            )
            .unwrap();

        assert_eq!(wia_claims.sub, sub.to_string());
        assert!(*wia_claims.exp.as_ref() > Utc::now());
    }
}
