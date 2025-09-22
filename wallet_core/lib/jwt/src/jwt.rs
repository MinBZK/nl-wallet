use std::borrow::Cow;
use std::collections::HashMap;
use std::marker::PhantomData;
use std::str::FromStr;
use std::sync::LazyLock;

use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use derive_more::AsRef;
use derive_more::Display;
use itertools::Itertools;
use jsonwebtoken::Algorithm;
use jsonwebtoken::DecodingKey;
use jsonwebtoken::Header;
use jsonwebtoken::Validation;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::CertificateDer;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::serde_as;

use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::error::JwtError;
use crate::error::JwtX5cError;
use crate::headers::HeaderWithJwk;
use crate::headers::HeaderWithTyp;
use crate::headers::HeaderWithX5c;
use crate::headers::JwtHeader;
use crate::jwk::jwk_to_p256;

/// JWT type, generic over its contents.
///
/// This wrapper of the `jsonwebtoken` crate echoes the following aspect of `jsonwebtoken`:
/// Validating one of the a standard fields during verification of the JWT using [`Validation`] does NOT automatically
/// result in enforcement that the field is present. For example, if validation of `exp` is turned on then JWTs without
/// an `exp` fields are still accepted (but not JWTs having an `exp` from the past).
///
/// Presence of the field may be enforced using [`Validation::required_spec_claims`] and/or by including it
/// explicitly as a field in the (de)serialized type.
#[derive(Debug, Clone, PartialEq, Eq, Display, SerializeDisplay, DeserializeFromStr)]
#[display("{serialization}")]
pub struct UnverifiedJwt<T, H = JwtHeader> {
    serialization: String,

    payload_end: usize,

    _jwt_type: PhantomData<(T, H)>,
}

impl<T, H> UnverifiedJwt<T, H> {
    pub fn serialization(&self) -> &str {
        &self.serialization
    }

    pub fn signed_slice(&self) -> &str {
        &self.serialization[..self.payload_end]
    }
}

impl<T, H> FromStr for UnverifiedJwt<T, H> {
    type Err = JwtError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let payload_end = s.rfind(".").ok_or(JwtError::UnexpectedNumberOfParts(1))?;

        Ok(Self {
            serialization: s.to_owned(),
            payload_end,
            _jwt_type: PhantomData,
        })
    }
}

impl<T, H> From<VerifiedJwt<T, H>> for UnverifiedJwt<T, H> {
    fn from(value: VerifiedJwt<T, H>) -> Self {
        value.jwt
    }
}

impl<T, H: DeserializeOwned> UnverifiedJwt<T, H> {
    pub fn dangerous_parse_header_unverified(&self) -> Result<H> {
        let header_end = self
            .signed_slice()
            .find(".")
            .ok_or(JwtError::UnexpectedNumberOfParts(2))?;
        let header: H = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(&self.serialization[..header_end])?)?;
        Ok(header)
    }
}

impl<T: DeserializeOwned, H: DeserializeOwned> UnverifiedJwt<T, H> {
    pub fn dangerous_parse_unverified(&self) -> Result<(H, T)> {
        let parts = self.serialization.split('.').collect_vec();
        if parts.len() != 3 {
            return Err(JwtError::UnexpectedNumberOfParts(parts.len()));
        }

        let header: H = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(parts[0])?)?;
        let payload: T = serde_json::from_slice(&BASE64_URL_SAFE_NO_PAD.decode(parts[1])?)?;

        Ok((header, payload))
    }
}

impl<T, H: DeserializeOwned> UnverifiedJwt<T, HeaderWithX5c<H>> {
    fn extract_x5c_certificates(&self) -> Result<VecNonEmpty<BorrowingCertificate>, JwtX5cError> {
        let header = self.dangerous_parse_header_unverified()?;
        Ok(header.x5c)
    }
}

impl<T, H: DeserializeOwned> UnverifiedJwt<T, HeaderWithJwk<H>> {
    fn extract_jwk(&self) -> Result<VerifyingKey, JwtError> {
        let header = self.dangerous_parse_header_unverified()?;
        Ok(header.verifying_key()?)
    }
}

impl<T, H, E> UnverifiedJwt<T, H>
where
    T: DeserializeOwned,
    H: TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    fn parse_and_verify_with_header(
        &self,
        pubkey: &EcdsaDecodingKey,
        validation_options: &Validation,
    ) -> Result<(H, T)> {
        let token_data = jsonwebtoken::decode::<T>(&self.serialization, &pubkey.0, validation_options)
            .map_err(JwtError::Validation)?;

        let header: H = token_data
            .header
            .try_into()
            .map_err(|e| JwtError::HeaderConversion(Box::new(e)))?;
        Ok((header, token_data.claims))
    }

    pub fn into_verified(
        self,
        pubkey: &EcdsaDecodingKey,
        validation_options: &Validation,
    ) -> Result<VerifiedJwt<T, H>> {
        let (header, payload) = self.parse_and_verify_with_header(pubkey, validation_options)?;

        Ok(VerifiedJwt {
            header,
            payload,
            jwt: self,
        })
    }
}

impl<T: DeserializeOwned, H> UnverifiedJwt<T, H> {
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify(&self, pubkey: &EcdsaDecodingKey, validation_options: &Validation) -> Result<T> {
        let token_data = jsonwebtoken::decode::<T>(&self.serialization, &pubkey.0, validation_options)
            .map_err(JwtError::Validation)?;

        Ok(token_data.claims)
    }
}

impl<T, H, E> UnverifiedJwt<T, HeaderWithX5c<H>>
where
    T: DeserializeOwned,
    H: DeserializeOwned + TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    /// Verify the JWS against the provided trust anchors, using the X.509 certificate(s) present in the `x5c` JWT
    /// header.
    pub fn parse_and_verify_against_trust_anchors(
        &self,
        trust_anchors: &[TrustAnchor],
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: CertificateUsage,
        validation_options: &Validation,
    ) -> Result<(HeaderWithX5c<H>, T), JwtX5cError> {
        let certificates = self.extract_x5c_certificates()?;

        // Verify the certificate chain against the trust anchors.
        let certificates = VecNonEmpty::try_from(certificates).map_err(|_| JwtX5cError::MissingCertificates)?;
        let leaf_cert = certificates.first();

        leaf_cert
            .verify(
                certificate_usage,
                &certificates
                    .iter()
                    .skip(1)
                    .map(AsRef::as_ref)
                    .map(CertificateDer::from_slice)
                    .collect_vec(),
                time,
                trust_anchors,
            )
            .map_err(JwtX5cError::CertificateValidation)?;

        // The leaf certificate is trusted, we can now use its public key to verify the JWS.
        let pubkey = leaf_cert.public_key();
        self.parse_and_verify_with_header(&pubkey.into(), validation_options)
            .map_err(JwtX5cError::Jwt)
    }
}

impl<T, H, E> UnverifiedJwt<T, HeaderWithJwk<H>>
where
    T: DeserializeOwned,
    H: DeserializeOwned + TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn parse_and_verify_with_jwk(
        &self,
        validation_options: &Validation,
    ) -> Result<(HeaderWithJwk<H>, T), JwtError> {
        let pubkey = self.extract_jwk()?;
        self.parse_and_verify_with_header(&(&pubkey).into(), validation_options)
    }

    pub fn parse_and_verify_with_expected_jwk(
        &self,
        expected_verifying_key: &VerifyingKey,
        validation_options: &Validation,
    ) -> Result<(HeaderWithJwk<H>, T), JwtError> {
        let (header, payload) =
            self.parse_and_verify_with_header(&(expected_verifying_key).into(), validation_options)?;

        // Compare the specified key against the one in the JWT header
        let contained_key = jwk_to_p256(&header.jwk)?;
        if contained_key != *expected_verifying_key {
            return Err(JwtError::IncorrectJwkPublicKey(
                Box::new(*expected_verifying_key),
                Box::new(contained_key),
            ));
        }

        Ok((header, payload))
    }
}

impl<T, H, E> UnverifiedJwt<T, HeaderWithX5c<H>>
where
    T: DeserializeOwned,
    H: DeserializeOwned + TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn into_verified_against_trust_anchors(
        self,
        validation_options: &Validation,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: CertificateUsage,
        trust_anchors: &[TrustAnchor],
    ) -> Result<VerifiedJwt<T, HeaderWithX5c<H>>, JwtX5cError> {
        let (header, payload) =
            self.parse_and_verify_against_trust_anchors(trust_anchors, time, certificate_usage, validation_options)?;

        Ok(VerifiedJwt {
            header,
            payload,
            jwt: self,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, AsRef, Display, SerializeDisplay)]
pub struct SignedJwt<T, H = JwtHeader>(UnverifiedJwt<T, H>);

impl<T: Serialize, H: Serialize> SignedJwt<T, H> {
    // TODO make private when SD JWT no longer needs it
    pub async fn sign(header: &H, payload: &T, privkey: &impl EcdsaKey) -> Result<Self> {
        let encoded_header = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(&header)?);
        let encoded_claims = BASE64_URL_SAFE_NO_PAD.encode(serde_json::to_vec(&payload)?);
        let message = [encoded_header, encoded_claims].join(".");
        let payload_end = message.len();

        let signature = privkey
            .try_sign(message.as_bytes())
            .await
            .map_err(|err| JwtError::Signing(Box::new(err)))?;
        let encoded_signature = BASE64_URL_SAFE_NO_PAD.encode(signature.to_vec());

        Ok(SignedJwt(UnverifiedJwt {
            serialization: [message, encoded_signature].join("."),

            payload_end,
            _jwt_type: PhantomData,
        }))
    }
}

impl<T: Serialize, H: Serialize> SignedJwt<T, H> {
    async fn sign_with_header_and_certificate<K: EcdsaKey>(
        header: H,
        payload: &T,
        keypair: &KeyPair<K>,
    ) -> Result<SignedJwt<T, HeaderWithX5c<H>>, JwtError> {
        let header = HeaderWithX5c::new(header, vec_nonempty![keypair.certificate().to_owned()]);
        let jwt = SignedJwt::sign(&header, payload, keypair.private_key()).await?;
        Ok(jwt)
    }
}

impl<T: Serialize> SignedJwt<T> {
    /// Sign a payload into a JWS, and put the certificate of the provided keypair in the `x5c` JWT header.
    /// The resulting JWS can be verified using [`verify_against_trust_anchors()`].
    pub async fn sign_with_certificate<K: EcdsaKey>(
        payload: &T,
        keypair: &KeyPair<K>,
    ) -> Result<SignedJwt<T, HeaderWithX5c>, JwtError> {
        // The `x5c` header supports certificate chains, but ISO 18013-5 doesn't: it requires that issuer
        // and RP certificates are signed directly by the trust anchor. So we don't support certificate chains
        // here (yet).
        SignedJwt::sign_with_header_and_certificate(JwtHeader::default(), payload, keypair).await
    }
}

impl<T: Serialize, H: Serialize> SignedJwt<T, H> {
    async fn sign_with_header_and_jwk(
        header: H,
        payload: &T,
        key: &impl EcdsaKey,
    ) -> Result<SignedJwt<T, HeaderWithJwk<H>>, JwtError> {
        let header = HeaderWithJwk::try_from_verifying_key_with_header(key, header).await?;
        let jwt = SignedJwt::sign(&header, payload, key).await?;
        Ok(jwt)
    }
}

impl<T: Serialize> SignedJwt<T> {
    pub async fn sign_with_jwk(payload: &T, key: &impl EcdsaKey) -> Result<SignedJwt<T, HeaderWithJwk>, JwtError> {
        SignedJwt::sign_with_header_and_jwk(JwtHeader::default(), payload, key).await
    }
}
impl<T, H> From<SignedJwt<T, H>> for UnverifiedJwt<T, H> {
    fn from(value: SignedJwt<T, H>) -> Self {
        value.0
    }
}

impl<T: DeserializeOwned, H: DeserializeOwned> From<SignedJwt<T, H>> for VerifiedJwt<T, H> {
    fn from(value: SignedJwt<T, H>) -> Self {
        // a signed JWT was just signed and therefore valid, so we can just parse it without verifying
        Self::dangerous_parse_unverified(&value.0.serialization).expect("should always parse")
    }
}

/// A verified JWS, along with its header and payload.
#[derive(Debug, Clone, PartialEq, Eq, Display, SerializeDisplay)]
#[display("{jwt}")]
pub struct VerifiedJwt<T, H = JwtHeader> {
    header: H,
    payload: T,

    jwt: UnverifiedJwt<T, H>,
}

/// Dangerously parse a JWT without verifying its signature. These methods should only be used for parsing JWTs that are
/// read from trusted sources, i.e. databases or configuration files.
impl<T: DeserializeOwned, H: DeserializeOwned> VerifiedJwt<T, H> {
    pub fn dangerous_parse_unverified(s: &str) -> Result<Self> {
        let jwt = s.parse::<UnverifiedJwt<T, H>>()?;
        let (header, payload) = jwt.dangerous_parse_unverified()?;

        Ok(Self { header, payload, jwt })
    }

    pub fn dangerous_deserialize<'de, D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::dangerous_parse_unverified(&s).map_err(serde::de::Error::custom)
    }
}

impl<T, H> VerifiedJwt<T, H> {
    pub fn header(&self) -> &H {
        &self.header
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn jwt(&self) -> &UnverifiedJwt<T, H> {
        &self.jwt
    }
}

pub type Result<T, E = JwtError> = std::result::Result<T, E>;

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

pub static DEFAULT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation_options = Validation::new(Algorithm::ES256);

    validation_options.required_spec_claims.clear(); // remove "exp" from required claims
    validation_options.leeway = 60;

    validation_options
});

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait JwtSub {
    const SUB: &'static str;
}

#[derive(Debug, Serialize, Deserialize)]
struct PayloadWithSub<T> {
    #[serde(flatten)]
    payload: T,

    sub: Cow<'static, str>,
}

impl<T: JwtSub> PayloadWithSub<T> {
    pub fn new(payload: T) -> Self {
        PayloadWithSub {
            payload,
            sub: Cow::Borrowed(T::SUB),
        }
    }
}

// "downcast" the payload, the `sub` claim can just be "ignored" when parsing
impl<T, H> From<SignedJwt<PayloadWithSub<T>, H>> for SignedJwt<T, H> {
    fn from(value: SignedJwt<PayloadWithSub<T>, H>) -> Self {
        SignedJwt(UnverifiedJwt {
            serialization: value.0.serialization,
            payload_end: value.0.payload_end,
            _jwt_type: PhantomData,
        })
    }
}

impl<T: JwtSub> JwtSub for PayloadWithSub<T> {
    const SUB: &'static str = T::SUB;
}

static SUB_JWT_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validations = DEFAULT_VALIDATIONS.to_owned();
    validations.required_spec_claims.insert("sub".to_string());
    validations
});

impl<T, H> UnverifiedJwt<T, H>
where
    T: DeserializeOwned + JwtSub,
    H: TryFrom<Header>,
{
    /// Verify the JWT, and parse and return its payload.
    pub fn parse_and_verify_with_sub(&self, pubkey: &EcdsaDecodingKey) -> Result<T> {
        let mut validations = SUB_JWT_VALIDATIONS.to_owned();
        validations.sub = Some(T::SUB.to_owned());
        self.parse_and_verify(pubkey, &validations)
    }
}

impl<T> SignedJwt<T, JwtHeader>
where
    T: Serialize + JwtSub,
{
    pub async fn sign_with_sub(payload: T, privkey: &impl EcdsaKey) -> Result<Self> {
        let claims = PayloadWithSub::new(payload);
        SignedJwt::sign(&JwtHeader::default(), &claims, privkey)
            .await
            .map(Into::into) // TODO should this return SignedJwt<PayloadWithSub<T>, JwtHeader> instead?
    }
}

///////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

pub trait JwtTyp {
    const TYP: &'static str = "jwt";
}

impl<T, H, E> UnverifiedJwt<T, HeaderWithTyp<H>>
where
    T: DeserializeOwned + JwtTyp,
    H: DeserializeOwned + TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn parse_and_verify_with_typ(&self, pubkey: &EcdsaDecodingKey, validation_options: &Validation) -> Result<T> {
        let (header, claims) = self.parse_and_verify_with_header(pubkey, validation_options)?;
        if header.typ != T::TYP {
            return Err(JwtError::UnexpectedTyp(T::TYP.to_owned(), header.typ.into_owned()));
        }

        Ok(claims)
    }

    pub fn into_verified_with_typ(
        self,
        pubkey: &EcdsaDecodingKey,
        validation_options: &Validation,
    ) -> Result<VerifiedJwt<T, HeaderWithTyp<H>>> {
        let jwt = self.into_verified(pubkey, validation_options)?;
        if jwt.header.typ != T::TYP {
            return Err(JwtError::UnexpectedTyp(T::TYP.to_owned(), jwt.header.typ.into_owned()));
        }

        Ok(jwt)
    }
}

impl<T, H, E> UnverifiedJwt<T, HeaderWithX5c<HeaderWithTyp<H>>>
where
    T: DeserializeOwned + JwtTyp,
    H: DeserializeOwned + TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn into_verified_against_trust_anchors_with_typ(
        self,
        validation_options: &Validation,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: CertificateUsage,
        trust_anchors: &[TrustAnchor],
    ) -> Result<VerifiedJwt<T, HeaderWithX5c<HeaderWithTyp<H>>>, JwtX5cError> {
        let jwt =
            self.into_verified_against_trust_anchors(validation_options, time, certificate_usage, trust_anchors)?;
        if jwt.header.inner().typ != T::TYP {
            return Err(JwtError::UnexpectedTyp(T::TYP.to_owned(), jwt.header.inner().typ.to_string()).into());
        }

        Ok(jwt)
    }
}

impl<T, H, E> UnverifiedJwt<T, HeaderWithJwk<HeaderWithTyp<H>>>
where
    T: DeserializeOwned + JwtTyp,
    H: DeserializeOwned + TryFrom<Header, Error = E>,
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn parse_and_verify_with_jwk_and_typ(
        &self,
        validation_options: &Validation,
    ) -> Result<(HeaderWithJwk<HeaderWithTyp<H>>, T)> {
        let (header, claims) = self.parse_and_verify_with_jwk(validation_options)?;
        if header.inner().typ != T::TYP {
            return Err(JwtError::UnexpectedTyp(
                T::TYP.to_owned(),
                header.inner().typ.to_string(),
            ));
        }

        Ok((header, claims))
    }

    pub fn parse_and_verify_with_expected_jwk_and_typ(
        &self,
        expected_verifying_key: &VerifyingKey,
        validation_options: &Validation,
    ) -> Result<(HeaderWithJwk<HeaderWithTyp<H>>, T)> {
        let (header, claims) = self.parse_and_verify_with_expected_jwk(expected_verifying_key, validation_options)?;
        if header.inner().typ != T::TYP {
            return Err(JwtError::UnexpectedTyp(
                T::TYP.to_owned(),
                header.inner().typ.to_string(),
            ));
        }

        Ok((header, claims))
    }
}

impl<T, H> SignedJwt<T, H>
where
    T: Serialize + JwtTyp,
    H: Serialize,
{
    // TODO make private when SD JWT no longer needs it
    pub async fn sign_with_header_and_typ(
        header: H,
        payload: &T,
        privkey: &impl EcdsaKey,
    ) -> Result<SignedJwt<T, HeaderWithTyp<H>>> {
        let header = HeaderWithTyp::new::<T>(header);
        let jwt = SignedJwt::sign(&header, payload, privkey).await?;
        Ok(jwt)
    }
}

impl<T> SignedJwt<T, HeaderWithTyp>
where
    T: Serialize + JwtTyp,
{
    // TODO remove when no longer needed
    pub async fn sign_with_typ(payload: &T, privkey: &impl EcdsaKey) -> Result<SignedJwt<T, HeaderWithTyp>> {
        let header = HeaderWithTyp::new::<T>(JwtHeader::default());
        let jwt = SignedJwt::sign(&header, payload, privkey).await?;
        Ok(jwt)
    }
}

impl<T> SignedJwt<T>
where
    T: Serialize + JwtTyp,
{
    pub async fn sign_with_certificate_and_typ(
        payload: &T,
        keypair: &KeyPair<impl EcdsaKey>,
    ) -> Result<SignedJwt<T, HeaderWithX5c<HeaderWithTyp>>, JwtError> {
        let header = HeaderWithTyp::new::<T>(JwtHeader::default());
        let jwt = SignedJwt::sign_with_header_and_certificate(header, payload, keypair).await?;
        Ok(jwt)
    }

    pub async fn sign_with_jwk_and_typ(
        payload: &T,
        key: &impl EcdsaKey,
    ) -> Result<SignedJwt<T, HeaderWithJwk<HeaderWithTyp>>, JwtError> {
        let header = HeaderWithTyp::new::<T>(JwtHeader::default());
        let jwt = SignedJwt::sign_with_header_and_jwk(header, payload, key).await?;
        Ok(jwt)
    }
}

/// The JWS JSON serialization, see <https://www.rfc-editor.org/rfc/rfc7515.html#section-7.2>,
/// which allows for a single payload to be signed by multiple signatures.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonJwt<T, H = JwtHeader> {
    pub payload: String,
    #[serde(flatten)]
    pub signatures: JsonJwtSignatures,
    #[serde(skip)]
    _phantomdata: PhantomData<(T, H)>,
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

impl<T, H> From<JsonJwt<T, H>> for Vec<UnverifiedJwt<T, H>> {
    fn from(value: JsonJwt<T, H>) -> Self {
        value
            .signatures
            .into_iter()
            .map(|sig| {
                [sig.protected, value.payload.clone(), sig.signature]
                    .join(".")
                    .parse()
                    .expect("should always parse as a JWT") // we just joined these parts, so this cannot fail
            })
            .collect()
    }
}

impl<T, H> TryFrom<VecNonEmpty<UnverifiedJwt<T, H>>> for JsonJwt<T, H> {
    type Error = JwtError;

    fn try_from(jwts: VecNonEmpty<UnverifiedJwt<T, H>>) -> Result<Self, Self::Error> {
        let split_jwts = jwts
            .into_inner()
            .into_iter()
            .map(|jwt| jwt.serialization.split('.').map(str::to_string).collect_vec())
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
    use std::fmt::Debug;

    use assert_matches::assert_matches;
    use base64::prelude::*;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rstest::rstest;
    use serde_json::json;

    use attestation_data::x509::generate::mock::generate_reader_mock_with_registration;
    use crypto::server_keys::generate::Ca;
    use crypto::x509::CertificateConfiguration;
    use crypto::x509::CertificateError;
    use crypto::x509::CertificateUsage;
    use utils::generator::TimeGenerator;

    use super::*;

    #[derive(Debug, PartialEq, Eq, Deserialize)]
    struct EmptyPayload {}

    #[rstest]
    #[case(include_str!("../examples/spec/example.jwt"), "eyJ0eXAiOiJKV1QiLA0KICJhbGciOiJIUzI1NiJ9.eyJpc3MiOiJqb2UiLA0KICJleHAiOjEzMDA4MTkzODAsDQogImh0dHA6Ly9leGFtcGxlLmNvbS9pc19yb290Ijp0cnVlfQ", Algorithm::HS256)]
    #[case(include_str!("../examples/spec/example_jws.jwt"), "eyJhbGciOiJSUzI1NiJ9.eyJpc3MiOiJqb2UiLA0KICJleHAiOjEzMDA4MTkzODAsDQogImh0dHA6Ly9leGFtcGxlLmNvbS9pc19yb290Ijp0cnVlfQ", Algorithm::RS256)]
    fn test_unverified_jwt_parse(#[case] jwt: &str, #[case] signed_slice: &str, #[case] alg: Algorithm) {
        let parsed: UnverifiedJwt<EmptyPayload> = jwt.parse().unwrap();
        assert_eq!(
            parsed,
            UnverifiedJwt {
                serialization: jwt.to_string(),
                payload_end: signed_slice.len(),
                _jwt_type: PhantomData
            }
        );
        assert_eq!(parsed.signed_slice(), signed_slice);

        let header = parsed.dangerous_parse_header_unverified().unwrap();
        assert_eq!(header.alg, alg);
        let (header, _) = parsed.dangerous_parse_unverified().unwrap();
        assert_eq!(header.alg, alg);
    }

    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
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

    impl JwtSub for ToyMessage {
        const SUB: &'static str = "toy_message";
    }

    #[tokio::test]
    async fn test_sign_and_verify_with_sub() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let jwt: UnverifiedJwt<_> = SignedJwt::sign_with_sub(t.clone(), &private_key).await.unwrap().into();

        // the JWT has a `sub` with the expected value
        let jwt_message: HashMap<String, serde_json::Value> = part(1, &jwt.serialization);
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

    impl JwtTyp for ToyMessage {}

    #[tokio::test]
    async fn test_sign_and_verify_with_typ() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let jwt: UnverifiedJwt<_, _> = SignedJwt::sign_with_typ(&t, &private_key).await.unwrap().into();

        // the JWT header has a `typ` with the expected value
        let jwt_header: HashMap<String, serde_json::Value> = part(0, &jwt.serialization);
        assert_eq!(
            *jwt_header.get("typ").unwrap(),
            serde_json::Value::String(ToyMessage::TYP.to_string())
        );

        // the JWT can be verified and parsed back into an identical value
        let parsed = jwt
            .parse_and_verify_with_typ(&private_key.verifying_key().into(), &DEFAULT_VALIDATIONS)
            .unwrap();

        assert_eq!(t, parsed);
    }

    #[tokio::test]
    async fn test_sign_and_verify_with_missing_typ() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        // sign without `typ`, serialize to string and parse to make the type system forget that this JWT doesn't have a
        // `HeaderWithTyp`
        let jwt: UnverifiedJwt<ToyMessage, HeaderWithTyp> = SignedJwt::sign(&JwtHeader::default(), &t, &private_key)
            .await
            .unwrap()
            .to_string()
            .as_str()
            .parse()
            .unwrap();

        // the JWT has no `typ` field
        let jwt_header: HashMap<String, serde_json::Value> = part(1, &jwt.serialization);
        assert!(!jwt_header.contains_key("typ"));

        // the JWT cannot be verified with `parse_and_verify_with_typ()`
        let parsed = jwt
            .parse_and_verify_with_typ(&private_key.verifying_key().into(), &DEFAULT_VALIDATIONS)
            .expect_err("should fail because the JWT has no `typ` field");

        assert_matches!(parsed, JwtError::HeaderConversion(_));
    }

    #[tokio::test]
    async fn test_sign_and_verify_with_wrong_typ() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        #[derive(Serialize, Deserialize)]
        struct OtherMessage;

        impl JwtTyp for OtherMessage {
            const TYP: &'static str = "wrong_typ";
        }

        // sign with a `typ` not corresponding to the payload type
        let jwt: UnverifiedJwt<ToyMessage, HeaderWithTyp> = SignedJwt::sign(
            &HeaderWithTyp::new::<OtherMessage>(JwtHeader::default()),
            &t,
            &private_key,
        )
        .await
        .unwrap()
        .into();

        // the JWT has a `sub` with the wrong value
        let jwt_header: HashMap<String, serde_json::Value> = part(0, &jwt.serialization);
        assert_eq!(
            *jwt_header.get("typ").unwrap(),
            serde_json::Value::String(OtherMessage::TYP.to_string())
        );

        // the JWT cannot be verified with `parse_and_verify_with_typ()`
        let parsed = jwt
            .parse_and_verify_with_typ(&private_key.verifying_key().into(), &DEFAULT_VALIDATIONS)
            .expect_err("should fail because the JWT has the wrong `typ` field");

        assert_matches!(parsed, JwtError::UnexpectedTyp(expected, found) if expected == ToyMessage::TYP && found == OtherMessage::TYP);
    }

    #[tokio::test]
    async fn test_sign_and_verify() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        let jwt: UnverifiedJwt<_> = SignedJwt::sign(&JwtHeader::default(), &t, &private_key)
            .await
            .unwrap()
            .into();

        // the JWT can be verified and parsed back into an identical value
        let parsed = jwt
            .parse_and_verify(&private_key.verifying_key().into(), &DEFAULT_VALIDATIONS)
            .unwrap();

        assert_eq!(t, parsed);
    }

    #[tokio::test]
    async fn test_sub_required() {
        let private_key = SigningKey::random(&mut OsRng);
        let t = ToyMessage::default();

        // create a new JWT without a `sub`
        let jwt: UnverifiedJwt<_> = SignedJwt::sign(&JwtHeader::default(), &t, &private_key)
            .await
            .unwrap()
            .into();
        let jwt_message: HashMap<String, serde_json::Value> = part(1, jwt.serialization());
        assert!(!jwt_message.contains_key("sub"));

        // verification fails because `sub` is required
        jwt.parse_and_verify_with_sub(&private_key.verifying_key().into())
            .unwrap_err();

        // we can parse and verify the JWT if we don't require the `sub` field to be present
        let parsed = jwt
            .parse_and_verify(&private_key.verifying_key().into(), &DEFAULT_VALIDATIONS)
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
    async fn test_json_jwt_serialization() {
        let private_key = SigningKey::random(&mut OsRng);

        let jwt: UnverifiedJwt<_> = SignedJwt::sign(&JwtHeader::default(), &ToyMessage::default(), &private_key)
            .await
            .unwrap()
            .into();

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
        let keypair = generate_reader_mock_with_registration(&ca, None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt: UnverifiedJwt<_, _> = SignedJwt::sign_with_certificate(&payload, &keypair)
            .await
            .unwrap()
            .into();

        let (header, deserialized) = jwt
            .parse_and_verify_against_trust_anchors(
                &[ca.to_trust_anchor()],
                &TimeGenerator,
                CertificateUsage::ReaderAuth,
                &DEFAULT_VALIDATIONS,
            )
            .unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(header.x5c.into_first(), *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_jwk() {
        let signing_key = SigningKey::random(&mut OsRng);

        let payload = json!({"hello": "world"});
        let jwt: UnverifiedJwt<_, _> = SignedJwt::sign_with_jwk(&payload, &signing_key).await.unwrap().into();

        let (header, deserialized) = jwt.parse_and_verify_with_jwk(&DEFAULT_VALIDATIONS).unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(header.verifying_key().unwrap(), *signing_key.verifying_key());

        let (header, deserialized) = jwt
            .parse_and_verify_with_expected_jwk(signing_key.verifying_key(), &DEFAULT_VALIDATIONS)
            .unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(header.verifying_key().unwrap(), *signing_key.verifying_key());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_wrong_jwk() {
        let signing_key = SigningKey::random(&mut OsRng);

        let payload = json!({"hello": "world"});
        let jwt: UnverifiedJwt<_, _> = SignedJwt::sign_with_jwk(&payload, &signing_key).await.unwrap().into();

        let (header, deserialized) = jwt.parse_and_verify_with_jwk(&DEFAULT_VALIDATIONS).unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(header.verifying_key().unwrap(), *signing_key.verifying_key());

        let wrong_key = SigningKey::random(&mut OsRng);
        let error = jwt
            .parse_and_verify_with_expected_jwk(wrong_key.verifying_key(), &DEFAULT_VALIDATIONS)
            .expect_err("should fail because the expected key is different from the actual key");

        assert_matches!(error, JwtError::Validation(_));
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_cert_intermediates() {
        // Generate a chain of certificates
        let ca = Ca::generate_with_intermediate_count("myca", CertificateConfiguration::default(), 3).unwrap();
        let intermediate1 = ca
            .generate_intermediate(
                "myintermediate1",
                CertificateUsage::ReaderAuth.into(),
                CertificateConfiguration::default(),
            )
            .unwrap();
        let intermediate2 = intermediate1
            .generate_intermediate(
                "myintermediate2",
                CertificateUsage::ReaderAuth.into(),
                CertificateConfiguration::default(),
            )
            .unwrap();
        let intermediate3 = intermediate2
            .generate_intermediate(
                "myintermediate3",
                CertificateUsage::ReaderAuth.into(),
                CertificateConfiguration::default(),
            )
            .unwrap();
        let keypair = generate_reader_mock_with_registration(&intermediate3, None).unwrap();

        // Construct a JWT with the `x5c` field containing the X.509 certificates
        // of the leaf certificate and the intermediates, in reverse order.
        let payload = json!({"hello": "world"});
        let certs = vec_nonempty![
            keypair.certificate().to_owned(),
            BorrowingCertificate::from_certificate_der(intermediate3.as_certificate_der().to_owned()).unwrap(),
            BorrowingCertificate::from_certificate_der(intermediate2.as_certificate_der().to_owned()).unwrap(),
            BorrowingCertificate::from_certificate_der(intermediate1.as_certificate_der().to_owned()).unwrap(),
        ];

        let jwt: UnverifiedJwt<_, _> =
            SignedJwt::sign(&HeaderWithX5c::from_certs(certs), &payload, keypair.private_key())
                .await
                .unwrap()
                .into();

        // Verifying this JWT against the CA's trust anchor should succeed.
        let (header, deserialized) = jwt
            .parse_and_verify_against_trust_anchors(
                &[ca.to_trust_anchor()],
                &TimeGenerator,
                CertificateUsage::ReaderAuth,
                &DEFAULT_VALIDATIONS,
            )
            .unwrap();

        assert_eq!(deserialized, payload);
        assert_eq!(header.x5c.into_first(), *keypair.certificate());
    }

    #[tokio::test]
    async fn test_parse_and_verify_jwt_with_wrong_cert() {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let keypair = generate_reader_mock_with_registration(&ca, None).unwrap();

        let payload = json!({"hello": "world"});
        let jwt: UnverifiedJwt<_, _> = SignedJwt::sign_with_certificate(&payload, &keypair)
            .await
            .unwrap()
            .into();

        let other_ca = Ca::generate("myca", Default::default()).unwrap();

        let err = jwt
            .parse_and_verify_against_trust_anchors(
                &[other_ca.to_trust_anchor()],
                &TimeGenerator,
                CertificateUsage::ReaderAuth,
                &DEFAULT_VALIDATIONS,
            )
            .unwrap_err();
        assert_matches!(
            err,
            JwtX5cError::CertificateValidation(CertificateError::Verification(_))
        );
    }

    #[rstest]
    #[case(Header::default())]
    #[case(JwtHeader::default())]
    #[case(ToyMessage { number: 1, string: "a".to_string() })]
    fn test_payload_with_sub_deserialize<U>(#[case] u: U)
    where
        U: Serialize + DeserializeOwned + Debug + PartialEq + Eq,
    {
        let t = PayloadWithSub {
            payload: u,
            sub: Cow::Borrowed("sub"),
        };
        let serialized = serde_json::to_string(&t).unwrap();
        let deserialized: U = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized, t.payload);
        let deserialized: PayloadWithSub<U> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.payload, t.payload);
        assert_eq!(deserialized.sub, t.sub);
    }
}
