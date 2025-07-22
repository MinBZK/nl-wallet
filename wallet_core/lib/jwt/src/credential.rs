use jsonwebtoken::Header;
use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::Deserialize;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::skip_serializing_none;

use crypto::keys::CredentialEcdsaKey;
use crypto::keys::CredentialKeyType;
use crypto::keys::EcdsaKey;

use crate::Jwt;
use crate::error::JwkConversionError;
use crate::error::JwtError;
use crate::jwk::jwk_from_p256;
use crate::validations;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JwtCredential<T> {
    pub(crate) private_key_id: String,
    pub(crate) key_type: CredentialKeyType,

    pub jwt: Jwt<JwtCredentialClaims<T>>,
}

impl<T> JwtCredential<T>
where
    T: DeserializeOwned,
{
    pub fn new<K: CredentialEcdsaKey>(
        private_key_id: String,
        jwt: Jwt<JwtCredentialClaims<T>>,
        pubkey: &VerifyingKey,
    ) -> Result<(Self, JwtCredentialClaims<T>), JwtError> {
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
}

/// Claims of a `JwtCredential`: the body of the JWT.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialClaims<T> {
    #[serde(rename = "cnf")]
    pub confirmation: JwtCredentialConfirmation,

    #[serde(flatten)]
    pub contents: JwtCredentialContents<T>,
}

impl<T> JwtCredentialClaims<T>
where
    T: Serialize,
{
    pub fn new(pubkey: &VerifyingKey, iss: String, attributes: T) -> Result<Self, JwkConversionError> {
        let claims = Self {
            confirmation: JwtCredentialConfirmation {
                jwk: jwk_from_p256(pubkey)?,
            },
            contents: JwtCredentialContents { iss, attributes },
        };

        Ok(claims)
    }

    pub async fn new_signed(
        holder_pubkey: &VerifyingKey,
        issuer_privkey: &impl EcdsaKey,
        iss: String,
        typ: Option<String>,
        attributes: T,
    ) -> Result<Jwt<JwtCredentialClaims<T>>, JwtError> {
        let jwt = Jwt::<JwtCredentialClaims<T>>::sign(
            &JwtCredentialClaims::<T>::new(holder_pubkey, iss, attributes)?,
            &Header {
                typ: typ.or(Some("jwt".to_string())),
                ..Header::new(jsonwebtoken::Algorithm::ES256)
            },
            issuer_privkey,
        )
        .await?;

        Ok(jwt)
    }
}

/// Contents of a `JwtCredential`, containing everything of the [`JwtCredentialClaims`] except the holder public
/// key (`Cnf`): the attributes and metadata of the credential.
#[skip_serializing_none]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialContents<T> {
    pub iss: String,

    #[serde(flatten)]
    pub attributes: T,
}

/// Contains the holder public key of a `JwtCredential`.
/// ("Cnf" stands for "confirmation", see <https://datatracker.ietf.org/doc/html/rfc7800>.)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct JwtCredentialConfirmation {
    pub jwk: Jwk,
}

// #[cfg(test)]
// mod tests {
//     use indexmap::IndexMap;

//     use attestation_data::x509::generate::mock::generate_issuer_mock;
//     use crypto::server_keys::generate::Ca;
//     use p256::ecdsa::SigningKey;
//     use rand_core::OsRng;

//     use super::JwtCredential;
//     use super::JwtCredentialClaims;

//     #[tokio::test]
//     async fn test_jwt_credential() {
//         let holder_key_id = "key";
//         let holder_keypair = SigningKey::random(&mut OsRng);
//         let ca = Ca::generate_issuer_mock_ca().unwrap();
//         let issuer_keypair = generate_issuer_mock(&ca, None).unwrap();

//         // Produce a JWT with `JwtCredentialClaims` in it
//         let jwt = JwtCredentialClaims::new_signed(
//             holder_keypair.verifying_key(),
//             issuer_keypair.private_key(),
//             issuer_keypair
//                 .certificate()
//                 .common_names()
//                 .unwrap()
//                 .first()
//                 .unwrap()
//                 .to_string(),
//             None,
//             IndexMap::<String, serde_json::Value>::default(),
//         )
//         .await
//         .unwrap();

//         let (cred, claims) = JwtCredential::new::<SigningKey>(
//             holder_key_id.to_string(),
//             jwt,
//             issuer_keypair.certificate().public_key(),
//         )
//         .unwrap();

//         assert_eq!(cred.jwt_claims(), claims);
//     }
// }
