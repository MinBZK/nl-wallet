use std::error::Error;

use attestation_types::status_claim::StatusClaim;
use chrono::DateTime;
use chrono::Utc;
use crypto::keys::SecureEcdsaKey;
use derive_more::Constructor;
use hsm::keys::HsmEcdsaKey;
use hsm::model::wrapped_key::WrappedKey;
use hsm::service::HsmError;
use hsm::service::Pkcs11Client;
use jwt::SignedJwt;
use jwt::UnverifiedJwt;
use jwt::error::JwtError;
use jwt::wia::WiaClaims;
use p256::ecdsa::VerifyingKey;

// used as the identifier for a WIA specific token status list
pub const WIA_ATTESTATION_TYPE_IDENTIFIER: &str = "wia";

pub trait WiaIssuer {
    type Error: Error + Send + Sync + 'static;

    async fn issue_wia(
        &self,
        exp: DateTime<Utc>,
        status_claim: StatusClaim,
    ) -> Result<(WrappedKey, UnverifiedJwt<WiaClaims>), Self::Error>;
    async fn public_key(&self) -> Result<VerifyingKey, Self::Error>;
}

#[derive(Constructor)]
pub struct HsmWiaIssuer<H, K = HsmEcdsaKey> {
    private_key: K,
    iss: String,
    hsm: H,
    wrapping_key_identifier: String,
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
        exp: DateTime<Utc>,
        status_claim: StatusClaim,
    ) -> Result<(WrappedKey, UnverifiedJwt<WiaClaims>), Self::Error> {
        let wrapped_privkey = self.hsm.generate_wrapped_key(&self.wrapping_key_identifier).await?;
        let pubkey = *wrapped_privkey.public_key();

        let jwt = SignedJwt::sign(
            &WiaClaims::new(&pubkey, self.iss.clone(), exp, status_claim)?,
            &self.private_key,
        )
        .await?
        .into();

        Ok((wrapped_privkey, jwt))
    }

    async fn public_key(&self) -> Result<VerifyingKey, Self::Error> {
        self.private_key
            .verifying_key()
            .await
            .map_err(|e| HsmWiaIssuerError::PublicKeyError(Box::new(e)))
    }
}

#[cfg(any(test, feature = "mock"))]
pub mod mock {
    use std::convert::Infallible;

    use attestation_types::status_claim::StatusClaim;
    use chrono::DateTime;
    use chrono::Utc;
    use hsm::model::wrapped_key::WrappedKey;
    use jwt::SignedJwt;
    use jwt::UnverifiedJwt;
    use jwt::wia::WiaClaims;
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
        ) -> Result<(WrappedKey, UnverifiedJwt<WiaClaims>), Self::Error> {
            let privkey = SigningKey::random(&mut OsRng);
            let pubkey = privkey.verifying_key();

            let jwt = SignedJwt::sign(
                &WiaClaims::new(pubkey, "iss".to_string(), exp, status_claim).unwrap(),
                &privkey, // Sign the WIA with its own private key in this test
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
    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use jwt::DEFAULT_VALIDATIONS;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;

    use super::HsmWiaIssuer;
    use super::WiaIssuer;

    #[tokio::test]
    async fn it_works() {
        let hsm = MockPkcs11Client::<HsmError>::default();
        let wia_signing_key = SigningKey::random(&mut OsRng);
        let wia_verifying_key = wia_signing_key.verifying_key();
        let iss = "iss";
        let wrapping_key_identifier = "my-wrapping-key-identifier";

        let wia_issuer = HsmWiaIssuer {
            private_key: wia_signing_key.clone(),
            iss: iss.to_string(),
            hsm,
            wrapping_key_identifier: wrapping_key_identifier.to_string(),
        };

        let (wia_privkey, wia) = wia_issuer
            .issue_wia(Utc::now() + Duration::from_secs(600), StatusClaim::new_mock())
            .await
            .unwrap();

        let (_, wia_claims) = wia
            .parse_and_verify(&wia_verifying_key.into(), &DEFAULT_VALIDATIONS)
            .unwrap();

        assert_eq!(wia_privkey.public_key(), &wia_claims.cnf.verifying_key().unwrap());

        // Check that the fields have the expected contents
        assert_eq!(wia_claims.iss, iss.to_string());
        assert!(wia_claims.exp > Utc::now());
    }
}
