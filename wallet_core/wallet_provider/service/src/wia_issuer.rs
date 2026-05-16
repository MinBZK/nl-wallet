use std::error::Error;

use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Duration;
use chrono::Utc;
use crypto::keys::SecureEcdsaKey;
use crypto::server_keys::KeyPair;
use derive_more::Constructor;
use hsm::keys::HsmEcdsaKey;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Client;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwtError;
use jwt::headers::HeaderWithX5c;
use jwt::wia::ClientStatus;
use jwt::wia::WiaClaims;
use jwt::wia::WiaWalletInfo;
use p256::ecdsa::VerifyingKey;

// used as the identifier for a WIA specific token status list
pub const WIA_ATTESTATION_TYPE_IDENTIFIER: &str = "wia";

/// How long issued WIAs will be valid (the token itself, not the wallet it represents).
const WIA_VALIDITY: Duration = Duration::hours(10);

pub trait WiaIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wia(
        &self,
        exp: DateTime<Utc>,
        status_claim: StatusClaim,
    ) -> Result<(WrappedKey, UnverifiedJwt<WiaClaims, HeaderWithX5c>), Self::Error>;
    async fn public_key(&self) -> Result<VerifyingKey, Self::Error>;
}

#[derive(Constructor)]
pub struct HsmWiaIssuer<H, K = HsmEcdsaKey> {
    keypair: KeyPair<K>,
    sub: String,
    hsm: H,
    wrapping_key_identifier: String,
    wallet_info: WiaWalletInfo,
}

#[derive(Debug, thiserror::Error, strum::IntoStaticStr)]
pub enum HsmWiaIssuerError {
    #[error("HSM error: {0}")]
    Hsm(#[from] HsmError),
    #[error("JWT error: {0}")]
    KeyConversion(#[from] JwtError),
    #[error("public key error: {0}")]
    PublicKeyError(Box<dyn Error + Send + Sync + 'static>),
}

impl<H, K> WiaIssuer for HsmWiaIssuer<H, K>
where
    H: Pkcs11Client,
    K: SecureEcdsaKey,
{
    type Error = HsmWiaIssuerError;

    async fn issue_wia(
        &self,
        wallet_exp: DateTime<Utc>,
        status_claim: StatusClaim,
    ) -> Result<(WrappedKey, UnverifiedJwt<WiaClaims, HeaderWithX5c>), Self::Error> {
        let wrapped_privkey = self.hsm.generate_wrapped_key(&self.wrapping_key_identifier).await?;
        let pubkey = *wrapped_privkey.public_key();

        let wia_exp = Utc::now() + WIA_VALIDITY;

        let jwt = SignedJwt::sign_with_certificate(
            &WiaClaims::new(
                &pubkey,
                self.sub.clone(),
                wia_exp,
                self.wallet_info.clone(),
                ClientStatus {
                    status: status_claim,
                    exp: wallet_exp,
                },
            )?,
            &self.keypair,
        )
        .await?
        .into();

        Ok((wrapped_privkey, jwt))
    }

    async fn public_key(&self) -> Result<VerifyingKey, Self::Error> {
        Ok(*self.keypair.certificate().public_key())
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use attestation_types::status_claim::StatusClaim;
    use chrono::DateTime;
    use chrono::Utc;
    use crypto::server_keys::generate::Ca;
    use hsm::model::wrapped_key::WrappedKey;
    use jwt::SignedJwt;
    use jwt::UnverifiedJwt;
    use jwt::headers::HeaderWithX5c;
    use jwt::wia::ClientStatus;
    use jwt::wia::WiaClaims;
    use jwt::wia::WiaWalletInfo;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::WiaIssuer;

    pub struct MockWiaIssuer;

    impl WiaIssuer for MockWiaIssuer {
        type Error = Infallible;

        async fn issue_wia(
            &self,
            exp: DateTime<Utc>,
            status_claim: StatusClaim,
        ) -> Result<(WrappedKey, UnverifiedJwt<WiaClaims, HeaderWithX5c>), Self::Error> {
            let privkey = SigningKey::random(&mut OsRng);
            let pubkey = privkey.verifying_key();

            let keypair = Ca::generate_issuer_mock_ca().unwrap().generate_wia_mock().unwrap();

            let jwt = SignedJwt::sign_with_certificate(
                &WiaClaims::new(
                    pubkey,
                    "sub".to_string(),
                    exp,
                    WiaWalletInfo::new_mock(),
                    ClientStatus {
                        status: status_claim,
                        exp,
                    },
                )
                .unwrap(),
                &keypair,
            )
            .await
            .unwrap()
            .into();

            Ok((
                WrappedKey::new(privkey.to_bytes().to_vec(), *privkey.verifying_key()),
                jwt,
            ))
        }

        async fn public_key(&self) -> Result<p256::ecdsa::VerifyingKey, Self::Error> {
            unimplemented!()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use attestation_types::status_claim::StatusClaim;
    use chrono::Utc;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateUsage;
    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use jwt::wia::WIA_JWT_VALIDATIONS;
    use jwt::wia::WiaWalletInfo;
    use utils::generator::TimeGenerator;

    use super::HsmWiaIssuer;
    use super::WiaIssuer;

    #[tokio::test]
    async fn it_works() {
        let hsm = MockPkcs11Client::<HsmError>::default();
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let wia_keypair = ca.generate_wia_mock().unwrap();
        let sub = "sub";
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let wia_issuer = HsmWiaIssuer {
            keypair: wia_keypair,
            sub: sub.to_string(),
            hsm,
            wrapping_key_identifier: wrapping_key_identifier.to_string(),
            wallet_info: WiaWalletInfo::new_mock(),
        };

        let (wia_privkey, wia) = wia_issuer
            .issue_wia(Utc::now() + Duration::from_secs(600), StatusClaim::new_mock())
            .await
            .unwrap();

        let (_, wia_claims) = wia
            .parse_and_verify_against_trust_anchors(
                &[ca.to_borrowing_trust_anchor()],
                &TimeGenerator,
                CertificateUsage::Wia,
                &WIA_JWT_VALIDATIONS,
            )
            .unwrap();

        assert_eq!(wia_privkey.public_key(), &wia_claims.cnf.verifying_key().unwrap());

        // Check that the fields have the expected contents
        assert_eq!(wia_claims.sub, sub.to_string());
        assert!(wia_claims.exp > Utc::now());
    }
}
