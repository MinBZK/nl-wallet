use josekit::JoseError;
use p256::ecdsa::VerifyingKey;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;

use error_category::ErrorCategory;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::keys::CredentialKeyType;

use crate::jwk_to_p256;
use crate::validations;
use crate::JwkConversionError;
use crate::Jwt;
use crate::JwtCredentialClaims;
use crate::JwtError;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtCredential<T> {
    pub(crate) private_key_id: String,
    pub(crate) key_type: CredentialKeyType,

    pub jwt: Jwt<JwtCredentialClaims<T>>,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum JwtCredentialError {
    #[error("failed to decode JWT body: {0}")]
    JoseDecoding(#[from] JoseError),
    #[error("unknown issuer: {0}")]
    #[category(critical)]
    UnknownIssuer(String),
    #[error("failed to parse trust anchor name: {0}")]
    #[category(critical)]
    TrustAnchorNameParsing(#[source] x509_parser::nom::Err<x509_parser::error::X509Error>),
    #[error("failed to verify JWT: {0}")]
    JwtVerification(#[from] jsonwebtoken::errors::Error),
    #[error("JWT error: {0}")]
    #[category(defer)]
    Jwt(#[from] JwtError),
}

impl<T> JwtCredential<T>
where
    T: DeserializeOwned,
{
    pub fn new<K: CredentialEcdsaKey>(
        private_key_id: String,
        jwt: Jwt<JwtCredentialClaims<T>>,
        pubkey: &VerifyingKey,
    ) -> Result<(Self, JwtCredentialClaims<T>), JwtCredentialError> {
        let claims = jwt.parse_and_verify(&pubkey.into(), &validations())?;

        let cred = Self {
            private_key_id,
            key_type: K::KEY_TYPE,
            jwt,
        };

        Ok((cred, claims))
    }

    #[cfg(any(feature = "test", test))]
    pub fn new_unverified<K: CredentialEcdsaKey>(private_key_id: String, jwt: Jwt<JwtCredentialClaims<T>>) -> Self {
        Self {
            private_key_id,
            key_type: K::KEY_TYPE,
            jwt,
        }
    }

    pub fn jwt_claims(&self) -> JwtCredentialClaims<T> {
        // Unwrapping is safe here because this was checked in new()
        let (_, contents) = self.jwt.dangerous_parse_unverified().unwrap();
        contents
    }

    pub fn private_key<K>(&self, key_factory: &impl KeyFactory<Key = K>) -> Result<K, JwkConversionError> {
        Ok(key_factory.generate_existing(&self.private_key_id, jwk_to_p256(&self.jwt_claims().confirmation.jwk)?))
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexMap;

    use nl_wallet_mdoc::server_keys::generate::Ca;
    use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;

    use crate::JwtCredentialClaims;

    use super::JwtCredential;
    #[tokio::test]
    async fn test_jwt_credential() {
        let holder_key_id = "key";
        let holder_keypair = MockRemoteEcdsaKey::new_random(holder_key_id.to_string());
        let issuer_keypair = Ca::generate_issuer_mock_ca()
            .unwrap()
            .generate_issuer_mock(None)
            .unwrap();

        // Produce a JWT with `JwtCredentialClaims` in it
        let jwt = JwtCredentialClaims::new_signed(
            holder_keypair.verifying_key(),
            issuer_keypair.private_key(),
            issuer_keypair
                .certificate()
                .common_names()
                .unwrap()
                .first()
                .unwrap()
                .to_string(),
            None,
            IndexMap::<String, serde_json::Value>::default(),
        )
        .await
        .unwrap();

        let (cred, claims) = JwtCredential::new::<MockRemoteEcdsaKey>(
            holder_key_id.to_string(),
            jwt,
            issuer_keypair.certificate().public_key(),
        )
        .unwrap();

        assert_eq!(cred.jwt_claims(), claims);
    }
}
