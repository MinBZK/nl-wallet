use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::LazyLock;

use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde::Serialize;
use serde_with::serde_as;
use x509_parser::der_parser::asn1_rs::BitString;
use x509_parser::der_parser::Oid;
use x509_parser::prelude::FromDer;
use x509_parser::x509::AlgorithmIdentifier;

use mdoc::server_keys::KeyPair;
use mdoc::utils::x509::BorrowingCertificate;
use mdoc::utils::x509::CertificateUsage;
use wallet_common::generator::Generator;
use wallet_common::keys::factory::KeyFactory;
use wallet_common::keys::CredentialEcdsaKey;
use wallet_common::keys::EcdsaKey;
use wallet_common::keys::SecureEcdsaKey;
use wallet_common::vec_at_least::VecNonEmpty;

use crate::error::JwtError;
use crate::error::JwtX5cError;

/// JWT type, generic over its contents.
///
/// This wrapper of the `jsonwebtoken` crate echoes the following aspect of `jsonwebtoken`:
/// Validating one of the a standard fields during verification of the JWT using [`Validation`] does NOT automatically
/// result in enforcement that the field is present. For example, if validation of `exp` is turned on then JWTs without
/// an `exp` fields are still accepted (but not JWTs having an `exp` from the past).
///
/// Presence of the field may be enforced using [`Validation::required_spec_claims`] and/or by including it
/// explicitly as a field in the (de)serialized type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Jwt<T>(pub String, PhantomData<T>);
impl<T, S: Into<String>> From<S> for Jwt<T> {
    fn from(val: S) -> Self {
        Jwt(val.into(), PhantomData)
    }
}

impl<T> FromStr for Jwt<T> {
    type Err = JwtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Jwt(s.into(), PhantomData))
    }
}

/// A verified JWS, along with its header and payload.
pub struct VerifiedJwt<T> {
    header: Header,
    payload: T,
    jwt: Jwt<T>,
}

impl<T> VerifiedJwt<T>
where
    T: DeserializeOwned,
{
    pub fn try_new(jwt: Jwt<T>, pubkey: &EcdsaDecodingKey, validation_options: &Validation) -> Result<Self> {
        let (header, payload) = jwt.parse_and_verify_with_header(pubkey, validation_options)?;

        Ok(Self { header, payload, jwt })
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
    pub fn payload(&self) -> &T {
        &self.payload
    }
    pub fn jwt(&self) -> &Jwt<T> {
        &self.jwt
    }
}

pub type Result<T, E = JwtError> = std::result::Result<T, E>;

pub trait JwtSubject {
    const SUB: &'static str;
}

/// EcdsaDecodingKey is an ECDSA public key for use with the `jsonwebtoken` crate. It wraps [`DecodingKey`] and aims to
/// solve a confusing aspect of the [`DecodingKey`] API: the functions [`DecodingKey::from_ec_der()`] and
/// [`DecodingKey::from_ec_pem()`] do not really do what their name suggests, and they are not equivalent apart from
/// taking DER and PEM encodings.
///
/// There are two commonly used encodings for ECDSA public keys:
///
/// * SEC1: this encodes the two public key coordinates (i.e. numbers) `x` and `y` that an ECDSA public key consists of
///   as `04 || x || y` where `||` is bitwise concatenation. Note that this encodes just the public key, and it does not
///   include any information on the particular curve that is used, of which the public key is an element. In case of
///   JWTs this is okay, because in that case that information is transmitted elsewhere: in the `alg` field of the JWT
///   header, which in our case is `ES256` - meaning the `secp256r` curve. This encoding is what
///   [`DecodingKey::from_ec_der()`] requires as input - even though it is not in fact DER.
/// * PKIX: this uses DER to encode an identifier for the curve (`secp256r` in our case), as well as the public key
///   coordinates in SEC1 form. This is the encoding that is used in X509 certificates (hence the name). The function
///   [`DecodingKey::from_ec_pem()`] accepts this encoding, in PEM form (although it also accepts SEC1-encoded keys in
///   PEM form).
///
/// This type solves the unclarity by explicitly naming the SEC1 encoding in [`EcdsaDecodingKey::from_sec1()`] that it
/// takes to construct it. From a `VerifyingKey` of the `ecdsa` crate, this encoding may be obtained by calling
/// `public_key.to_encoded_point(false).as_bytes()`.
#[derive(Clone)]
pub struct EcdsaDecodingKey(pub DecodingKey);

impl From<DecodingKey> for EcdsaDecodingKey {
    fn from(value: DecodingKey) -> Self {
        EcdsaDecodingKey(value)
    }
}

impl From<&VerifyingKey> for EcdsaDecodingKey {
    fn from(value: &VerifyingKey) -> Self {
        EcdsaDecodingKey::from_sec1(value.to_encoded_point(false).as_bytes())
    }
}

impl EcdsaDecodingKey {
    pub fn from_sec1(key: &[u8]) -> Self {
        DecodingKey::from_ec_der(key).into()
    }
}

/// The OID of Elliptic Curve public keys.
static OID_EC_PUBKEY: LazyLock<Oid<'static>> = LazyLock::new(|| Oid::from_str("1.2.840.10045.2.1").unwrap());

impl<T> Jwt<T>
where
    T: DeserializeOwned,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify(&self, pubkey: &EcdsaDecodingKey, validation_options: &Validation) -> Result<T> {
        let (_, claims) = self.parse_and_verify_with_header(pubkey, validation_options)?;

        Ok(claims)
    }

    pub fn parse_and_verify_with_header(
        &self,
        pubkey: &EcdsaDecodingKey,
        validation_options: &Validation,
    ) -> Result<(Header, T)> {
        let payload =
            jsonwebtoken::decode::<T>(&self.0, &pubkey.0, validation_options).map_err(JwtError::Validation)?;

        Ok((payload.header, payload.claims))
    }

    /// Verify a JWT against the `subjectPublicKeyInfo` of a trust anchor.
    pub fn verify_against_spki(&self, spki: &[u8]) -> Result<T> {
        let (key_bytes, algorithm) =
            AlgorithmIdentifier::from_der(spki).map_err(JwtError::TrustAnchorAlgorithmParsing)?;
        if algorithm.algorithm != *OID_EC_PUBKEY {
            return Err(JwtError::TrustAnchorKeyFormat(algorithm.oid().to_id_string()));
        }

        let (_, key_bytes) = BitString::from_der(key_bytes)?;
        let key = DecodingKey::from_ec_der(&key_bytes.data); // this is actually SEC1, not DER

        let claims = jsonwebtoken::decode(&self.0, &key, &validations())
            .map_err(JwtError::Validation)?
            .claims;

        Ok(claims)
    }

    pub fn dangerous_parse_unverified(&self) -> Result<(Header, T)> {
        let parts = self.0.split('.').collect_vec();
        if parts.len() != 3 {
            return Err(JwtError::UnexpectedNumberOfParts(parts.len()));
        }

        let header: Header = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(parts[0])?)?;
        let body: T = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(parts[1])?)?;

        Ok((header, body))
    }

    /// Verify the JWS against the provided trust anchors, using the X.509 certificate(s) present in the `x5c` JWT
    /// header.
    pub fn verify_against_trust_anchors<A: ToString>(
        &self,
        audience: &[A],
        trust_anchors: &[TrustAnchor],
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<(T, BorrowingCertificate), JwtX5cError> {
        let header = jsonwebtoken::decode_header(&self.0).map_err(JwtError::Validation)?;
        let mut certs = header
            .x5c
            .ok_or(JwtX5cError::MissingCertificates)?
            .into_iter()
            .map(|cert_base64| {
                let cert = CertificateDer::from(
                    BASE64_STANDARD
                        .decode(cert_base64)
                        .map_err(JwtX5cError::CertificateBase64)?,
                );
                Ok(cert)
            })
            .collect::<Result<VecDeque<_>, JwtX5cError>>()?;

        // Verify the certificate chain against the trust anchors.
        let leaf_cert =
            BorrowingCertificate::from_certificate_der(certs.pop_front().ok_or(JwtX5cError::MissingCertificates)?)
                .map_err(JwtX5cError::CertificateParsing)?;
        // The `VecDeque` containing the certificates will be contiguous at this point, so the second value is empty.
        let (intermediates, _) = certs.as_slices();
        leaf_cert
            .verify(CertificateUsage::ReaderAuth, intermediates, time, trust_anchors)
            .map_err(JwtX5cError::CertificateValidation)?;

        // The leaf certificate is trusted, we can now use its public key to verify the JWS.
        let pubkey = leaf_cert.public_key();

        let validation_options = {
            let mut validation = Validation::new(Algorithm::ES256);

            validation.required_spec_claims = HashSet::default();
            validation.set_audience(audience);

            validation
        };

        let payload = self.parse_and_verify(&pubkey.into(), &validation_options)?;

        Ok((payload, leaf_cert))
    }
}

impl<T> Jwt<T>
where
    T: Serialize,
{
    pub async fn sign(payload: &T, header: &Header, privkey: &impl EcdsaKey) -> Result<Jwt<T>> {
        let encoded_header = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(header)?);
        let encoded_claims = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(payload)?);
        let message = [encoded_header, encoded_claims].join(".");

        let signature = privkey
            .try_sign(message.as_bytes())
            .await
            .map_err(|err| JwtError::Signing(Box::new(err)))?;
        let encoded_signature = BASE64_URL_SAFE_NO_PAD.encode(signature.to_vec());

        Ok([message, encoded_signature].join(".").into())
    }

    /// Bulk-sign the keys and JWT payloads into JWTs.
    pub async fn sign_bulk<K: CredentialEcdsaKey>(
        keys_and_messages: Vec<(K, (T, jsonwebtoken::Header))>,
        key_factory: &impl KeyFactory<Key = K>,
    ) -> Result<Vec<(K, Jwt<T>)>, JwtError> {
        let (keys, to_sign): (Vec<_>, Vec<_>) = keys_and_messages.into_iter().unzip();

        // Construct a Vec containing the strings to be signed with the private keys, i.e. schematically "header.body"
        let messages = to_sign
            .iter()
            .map(|(message, header)| {
                Ok([
                    BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(header)?),
                    BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(message)?),
                ]
                .join("."))
            })
            .collect::<Result<Vec<_>, JwtError>>()?;

        // Have the WP sign our messages.
        let signatures = key_factory
            .sign_multiple_with_existing_keys(
                messages
                    .iter()
                    .map(|msg| msg.clone().into_bytes())
                    .zip(keys.iter().map(|key| vec![key]))
                    .collect_vec(),
            )
            .await
            .map_err(|err| JwtError::Signing(Box::new(err)))?;

        let jwts = signatures
            .into_iter()
            .zip(keys)
            .zip(messages)
            .map(|((sigs, key), msg)| {
                // unwrap: we sent `vec![key]` above, i.e. a single key, so we will get a single signature back
                let jwt = [msg, BASE64_URL_SAFE_NO_PAD.encode(sigs.first().unwrap().to_vec())]
                    .join(".")
                    .into();
                (key, jwt)
            })
            .collect();

        Ok(jwts)
    }

    /// Sign a payload into a JWS, and put the certificate of the provided keypair in the `x5c` JWT header.
    /// The resulting JWS can be verified using [`verify_against_trust_anchors()`].
    pub async fn sign_with_certificate<K: EcdsaKey>(payload: &T, keypair: &KeyPair<K>) -> Result<Self, JwtError> {
        // The `x5c` header supports certificate chains, but ISO 18013-5 doesn't: it requires that issuer
        // and RP certificates are signed directly by the trust anchor. So we don't support certificate chains
        // here (yet).
        let certs = vec![BASE64_STANDARD.encode(keypair.certificate().as_ref())];

        let jwt = Jwt::sign(
            payload,
            &Header {
                alg: jsonwebtoken::Algorithm::ES256,
                x5c: Some(certs),
                ..Default::default()
            },
            keypair.private_key(),
        )
        .await?;

        Ok(jwt)
    }
}

pub fn validations() -> Validation {
    let mut validation_options = Validation::new(Algorithm::ES256);

    validation_options.required_spec_claims.clear(); // we generally don't use `exp`, don't require it
    validation_options.leeway = 60;

    validation_options
}

pub fn header() -> Header {
    Header {
        alg: Algorithm::ES256,
        ..Default::default()
    }
}

impl<T> Jwt<T>
where
    T: Serialize + DeserializeOwned + JwtSubject,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify_with_sub(&self, pubkey: &EcdsaDecodingKey) -> Result<T> {
        let mut validation_options = validations();
        validation_options.required_spec_claims.insert("sub".to_string());
        self.parse_and_verify(pubkey, &validation_options)
    }

    pub async fn sign_with_sub(payload: &T, privkey: &impl SecureEcdsaKey) -> Result<Jwt<T>> {
        let header = &Header {
            alg: Algorithm::ES256,
            kid: "0".to_owned().into(),
            ..Default::default()
        };
        let claims = &JwtPayload {
            payload,
            sub: T::SUB.to_owned(),
        };

        let jwt = Jwt::sign(claims, header, privkey).await?.0;
        Ok(jwt.into())
    }
}

#[derive(Serialize, Deserialize, Debug)]
struct JwtPayload<T> {
    #[serde(flatten)]
    payload: T,
    sub: String,
}

impl<T> Serialize for Jwt<T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        String::serialize(&self.0, serializer)
    }
}
impl<'de, T> Deserialize<'de> for Jwt<T> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        String::deserialize(deserializer).map(Jwt::from)
    }
}

/// The JWS JSON serialization, see <https://www.rfc-editor.org/rfc/rfc7515.html#section-7.2>,
/// which allows for a single payload to be signed by multiple signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonJwt<T> {
    pub payload: String,
    #[serde(flatten)]
    pub signatures: JsonJwtSignatures,
    #[serde(skip)]
    _phantomdata: PhantomData<T>,
}

/// Contains the JWS signatures, supporting both the "general" and "flattened" syntaxes.
///
/// The "general" syntax uses `NonEmpty` so this type always contains at least one `JsonJwtSignature`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonJwtSignatures {
    General {
        signatures: VecNonEmpty<JsonJwtSignature>,
    },
    Flattened {
        #[serde(flatten)]
        signature: JsonJwtSignature,
    },
}

impl IntoIterator for JsonJwtSignatures {
    type Item = JsonJwtSignature;

    type IntoIter = std::vec::IntoIter<JsonJwtSignature>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            JsonJwtSignatures::General { signatures } => signatures.into_inner().into_iter(),
            JsonJwtSignatures::Flattened { signature } => vec![signature].into_iter(),
        }
    }
}

impl From<VecNonEmpty<JsonJwtSignature>> for JsonJwtSignatures {
    fn from(signatures: VecNonEmpty<JsonJwtSignature>) -> Self {
        match signatures.len().get() {
            1 => Self::Flattened {
                signature: signatures.into_inner().pop().unwrap(),
            },
            _ => Self::General { signatures },
        }
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonJwtSignature {
    /// Base64-enoded JWS header, the same as the header of a normal JWS. `alg` is required.
    pub protected: String,

    /// Unsigned JWS header (optional). May contain any of the fields of a normal JWS header, but none of them are
    /// required. Unlike the `protected` header, this field is not included when signing the JWS.
    /// (which is also why it is not Base64-encoded, unlike `protected` and the `payload` of [`JsonJwt<T>`]).
    #[serde(default)]
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    pub header: HashMap<String, serde_json::Value>,

    /// Signature of the JWS. When (1) the `protected` of this struct, (2) the `payload` of [`JsonJwt<T>`]
    /// and (3) this `signature` are concatenated with a `.` in between, then the result is a valid normal JWS.
    pub signature: String,
}

impl<T> From<JsonJwt<T>> for Vec<Jwt<T>> {
    fn from(value: JsonJwt<T>) -> Self {
        value
            .signatures
            .into_iter()
            .map(|sig| [sig.protected, value.payload.clone(), sig.signature].join(".").into())
            .collect()
    }
}

impl<T> TryFrom<VecNonEmpty<Jwt<T>>> for JsonJwt<T> {
    type Error = JwtError;

    fn try_from(jwts: VecNonEmpty<Jwt<T>>) -> Result<Self, Self::Error> {
        let split_jwts = jwts
            .into_inner()
            .into_iter()
            .map(|jwt| jwt.0.split('.').map(str::to_string).collect_vec())
            .collect_vec();

        let mut first = split_jwts.first().unwrap().clone(); // this came from a NonEmpty<>
        if first.len() != 3 {
            return Err(JwtError::UnexpectedNumberOfParts(first.len()));
        }
        let payload = first.remove(1); // `remove` is like `get`, but also moves out of the vec, so we can avoid cloning

        let signatures: VecNonEmpty<_> = split_jwts
            .into_iter()
            .map(|mut split_jwt| {
                if split_jwt.len() != 3 {
                    return Err(JwtError::UnexpectedNumberOfParts(split_jwt.len()));
                }
                if split_jwt[1] != payload {
                    return Err(JwtError::DifferentPayloads(split_jwt.remove(1), payload.clone()));
                }
                Ok(JsonJwtSignature {
                    signature: split_jwt.remove(2),
                    protected: split_jwt.remove(0),
                    header: HashMap::default(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .unwrap(); // our iterable `split_jwts` came from a `NonEmpty`

        let json_jwt = Self {
            payload: payload.clone(),
            signatures: signatures.into(),
            _phantomdata: PhantomData,
        };

        Ok(json_jwt)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use assert_matches::assert_matches;
    use base64::prelude::*;
    use futures::StreamExt;
    use jsonwebtoken::Header;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use serde_json::json;

    use mdoc::server_keys::generate::Ca;
    use mdoc::utils::x509::CertificateConfiguration;
    use mdoc::utils::x509::CertificateError;
    use mdoc::utils::x509::CertificateUsage;
    use wallet_common::generator::TimeGenerator;
    use wallet_common::keys::mock_remote::MockRemoteKeyFactory;

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct ToyMessage {
        number: u8,
        string: String,
    }

    impl Default for ToyMessage {
        fn default() -> Self {
            Self {
                number: 42,
                string: "Hello, world!".to_string(),
            }
        }
    }

    impl JwtSubject for ToyMessage {
        const SUB: &'static str = "toy_message";
    }

    #[tokio::test]
    async fn test_sign_and_verify_with_sub() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let jwt = Jwt::sign_with_sub(&t, &private_key).await.unwrap();

        // the JWT has a `sub` with the expected value
        let jwt_message: HashMap<String, serde_json::Value> = part(1, &jwt.0);
        assert_eq!(
            *jwt_message.get("sub").unwrap(),
            serde_json::Value::String(ToyMessage::SUB.to_string())
        );

        // the JWT can be verified and parsed back into an identical value
        let parsed = jwt
            .parse_and_verify_with_sub(&private_key.verifying_key().into())
            .unwrap();

        assert_eq!(t, parsed);
    }

    #[tokio::test]
    async fn test_sign_and_verify() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let header = header();
        let jwt = Jwt::sign(&t, &header, &private_key).await.unwrap();

        // the JWT can be verified and parsed back into an identical value
        let parsed = jwt
            .parse_and_verify(&private_key.verifying_key().into(), &validations())
            .unwrap();

        assert_eq!(t, parsed);
    }

    #[tokio::test]
    async fn test_sub_required() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        // create a new JWT without a `sub`
        let header = header();
        let jwt = Jwt::sign(&t, &header, &private_key).await.unwrap();
        let jwt_message: HashMap<String, serde_json::Value> = part(1, &jwt.0);
        assert!(!jwt_message.contains_key("sub"));

        // verification fails because `sub` is required
        jwt.parse_and_verify_with_sub(&private_key.verifying_key().into())
            .unwrap_err();

        // we can parse and verify the JWT if we don't require the `sub` field to be present
        let parsed = jwt
            .parse_and_verify(&private_key.verifying_key().into(), &validations())
            .unwrap();

        assert_eq!(t, parsed);
    }

    /// Decode and deserialize the specified part of the JWT.
    fn part<T: DeserializeOwned>(i: u8, jwt: &str) -> T {
        let bts = BASE64_URL_SAFE_NO_PAD
            .decode(jwt.split('.').take((i + 1) as usize).last().unwrap())
            .unwrap();
        serde_json::from_slice(&bts).unwrap()
    }

    #[tokio::test]
    async fn test_sign_jwts() {
        bulk_jwt_sign(&MockRemoteKeyFactory::default()).await;
    }

    pub async fn bulk_jwt_sign<K: CredentialEcdsaKey>(key_factory: &impl KeyFactory<Key = K>) {
        // Generate keys to sign with and messages to sign
        let keys = key_factory.generate_new_multiple(4).await.unwrap();
        let keys_and_messages = keys
            .into_iter()
            .enumerate()
            .map(|(number, key)| {
                (
                    key,
                    (
                        ToyMessage {
                            number: number as u8,
                            ..Default::default()
                        },
                        Header::new(Algorithm::ES256),
                    ),
                )
            })
            .collect();

        let jwts = Jwt::sign_bulk(keys_and_messages, key_factory).await.unwrap();

        // Verify JWTs. (futures::stream supports async for_each closures.)
        futures::stream::iter(jwts)
            .for_each(|(key, jwt)| async move {
                jwt.parse_and_verify(
                    &EcdsaDecodingKey::from(&key.verifying_key().await.unwrap()),
                    &validations(),
                )
                .unwrap();
            })
            .await;
    }

    #[tokio::test]
    async fn test_json_jwt_serialization() {
        let private_key = SigningKey::random(&mut OsRng);

        let jwt = Jwt::sign(&ToyMessage::default(), &header(), &private_key)
            .await
            .unwrap();

        let json_jwt_one: JsonJwt<_> = VecNonEmpty::try_from(vec![jwt.clone()]).unwrap().try_into().unwrap();
        assert_matches!(json_jwt_one.signatures, JsonJwtSignatures::Flattened { .. });
        let serialized = serde_json::to_string(&json_jwt_one).unwrap();

        let deserialized: JsonJwt<ToyMessage> = serde_json::from_str(&serialized).unwrap();
        assert_matches!(deserialized.signatures, JsonJwtSignatures::Flattened { .. });

        let json_jwt_two: JsonJwt<_> = VecNonEmpty::try_from(vec![jwt.clone(), jwt.clone()])
            .unwrap()
            .try_into()
            .unwrap();
        assert_matches!(json_jwt_two.signatures, JsonJwtSignatures::General { .. });
        let serialized = serde_json::to_string(&json_jwt_two).unwrap();
        let deserialized: JsonJwt<ToyMessage> = serde_json::from_str(&serialized).unwrap();
        assert_matches!(deserialized.signatures, JsonJwtSignatures::General { .. });

        // Construct a JsonJwt having one signature but which uses JsonJwtSignatures::General
        let JsonJwtSignatures::General { signatures } = json_jwt_two.signatures else {
            panic!("expected the JsonJwtSignatures::General variant") // we actually already checked this above
        };
        let mut signatures = signatures.into_inner();
        signatures.pop();
        let json_jwt_mixed = JsonJwt::<ToyMessage> {
            payload: json_jwt_two.payload,
            signatures: JsonJwtSignatures::General {
                signatures: signatures.try_into().unwrap(),
            },
            _phantomdata: PhantomData,
        };

        // We can (de)serialize such instances even though we don't produce them ourselves
        let serialized = serde_json::to_string(&json_jwt_mixed).unwrap();
        let deserialized: JsonJwt<ToyMessage> = serde_json::from_str(&serialized).unwrap();
        assert_matches!(deserialized.signatures, JsonJwtSignatures::General { .. });
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_cert() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let keypair = ca.generate_reader_mock(None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt = Jwt::sign_with_certificate(&payload, &keypair).await.unwrap();

        let audience: &[String] = &[];
        let (deserialized, leaf_cert) = jwt
            .verify_against_trust_anchors(audience, &[ca.to_trust_anchor()], &TimeGenerator)
            .unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(leaf_cert, *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_cert_intermediates() {
        // Generate a chain of certificates
        let ca = Ca::generate_with_intermediate_count("myca", CertificateConfiguration::default(), 3).unwrap();
        let intermediate1 = ca
            .generate_intermediate(
                "myintermediate1",
                CertificateUsage::ReaderAuth,
                CertificateConfiguration::default(),
            )
            .unwrap();
        let intermediate2 = intermediate1
            .generate_intermediate(
                "myintermediate2",
                CertificateUsage::ReaderAuth,
                CertificateConfiguration::default(),
            )
            .unwrap();
        let intermediate3 = intermediate2
            .generate_intermediate(
                "myintermediate3",
                CertificateUsage::ReaderAuth,
                CertificateConfiguration::default(),
            )
            .unwrap();
        let keypair = intermediate3.generate_reader_mock(None).unwrap();

        // Construct a JWT with the `x5c` field containing the X.509 certificates
        // of the leaf certificate and the intermediates, in reverse order.
        let payload = json!({"hello": "world"});
        let certs = vec![
            keypair.certificate().as_ref(),
            intermediate3.as_certificate_der().as_ref(),
            intermediate2.as_certificate_der().as_ref(),
            intermediate1.as_certificate_der().as_ref(),
        ]
        .into_iter()
        .map(|der| BASE64_STANDARD.encode(der))
        .collect();

        let jwt = Jwt::sign(
            &payload,
            &Header {
                alg: jsonwebtoken::Algorithm::ES256,
                x5c: Some(certs),
                ..Default::default()
            },
            keypair.private_key(),
        )
        .await
        .unwrap();

        // Verifying this JWT against the CA's trust anchor should succeed.
        let audience: &[String] = &[];
        let (deserialized, leaf_cert) = jwt
            .verify_against_trust_anchors(audience, &[ca.to_trust_anchor()], &TimeGenerator)
            .unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(leaf_cert, *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_wrong_cert() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let keypair = ca.generate_reader_mock(None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt = Jwt::sign_with_certificate(&payload, &keypair).await.unwrap();

        let other_ca = Ca::generate("myca", Default::default()).unwrap();

        let audience: &[String] = &[];
        let err = jwt
            .verify_against_trust_anchors(audience, &[other_ca.to_trust_anchor()], &TimeGenerator)
            .unwrap_err();
        assert_matches!(
            err,
            JwtX5cError::CertificateValidation(CertificateError::Verification(_))
        );
    }
}
