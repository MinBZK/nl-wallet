use std::collections::BTreeSet;

use chrono::DateTime;
use chrono::Utc;
use ciborium::value::Value;
use coset::AsCborValue;
use coset::CborSerializable;
use coset::ContentType;
use coset::CoseSign1;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::ProtectedHeader;
use coset::cwt::ClaimsSet;
use coset::cwt::Timestamp;
use coset::iana;
use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
use error_category::ErrorCategory;
use serde::Serialize;
use serde::de::DeserializeOwned;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;

use crate::COSE_X5CHAIN_HEADER_LABEL;
use crate::CoseError;
use crate::TypedCose;
use crate::x5chain_from_header;

/// ETSI TS 119 475 content type for a Wallet Relying Party Registration Certificate encoded as a CWT.
pub const WRPRC_CWT_CONTENT_TYPE: &str = "rc-wrp+cwt";

/// CWT Claims COSE header parameter, registered by RFC 9597.
pub const CWT_CLAIMS_HEADER_LABEL: i64 = 15;

const COSE_ALGORITHM_HEADER_LABEL: i64 = 1;
const COSE_CONTENT_TYPE_HEADER_LABEL: i64 = 3;

#[derive(Debug, Clone, PartialEq)]
pub struct CwtHeader {
    pub claims: Option<ClaimsSet>,
    pub x5chain: VecNonEmpty<BorrowingCertificate>,
}

impl CwtHeader {
    pub fn issued_at(&self) -> Option<&Timestamp> {
        self.claims.as_ref().and_then(|claims| claims.issued_at.as_ref())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum CwtError {
    #[error(transparent)]
    Cose(#[from] CoseError),
    #[error("could not decode or encode the COSE_Sign1 object: {0}")]
    #[category(critical)]
    CoseSerialization(#[source] coset::CoseError),
    #[error("invalid COSE_Sign1 structure: {0}")]
    #[category(critical)]
    InvalidCoseSign1(&'static str),
    #[error("unexpected CBOR tag for COSE_Sign1: {0}")]
    #[category(critical)]
    UnexpectedCborTag(u64),
    #[error("missing protected CWT content type")]
    #[category(critical)]
    MissingContentType,
    #[error("unexpected protected CWT content type: {0:?}")]
    #[category(critical)]
    UnexpectedContentType(ContentType),
    #[error("CWT header parameter {0:?} must be protected")]
    #[category(critical)]
    UnprotectedHeaderParameter(Label),
    #[error("CWT iat claim is not a finite NumericDate")]
    #[category(critical)]
    InvalidIssuedAt,
}

/// A parsed, but not yet authenticated, CBOR Web Token.
///
/// Parsing validates the COSE and CWT structure, but does not establish the authenticity of the header or payload.
/// Use [`UnverifiedCwt::into_verified_against_trust_anchors`] before consuming either as trusted input.
#[derive(Debug, PartialEq)]
pub struct UnverifiedCwt<T> {
    cose: TypedCose<CoseSign1, T>,
    unverified_header: CwtHeader,
}

impl<T> Clone for UnverifiedCwt<T> {
    fn clone(&self) -> Self {
        Self {
            cose: self.cose.clone(),
            unverified_header: self.unverified_header.clone(),
        }
    }
}

impl<T> UnverifiedCwt<T> {
    /// Parse an untagged COSE_Sign1 object, or one carrying the optional COSE_Sign1 CBOR tag.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, CwtError> {
        Self::try_from(parse_cose_sign1(bytes)?)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, CwtError> {
        self.cose
            .as_inner()
            .clone()
            .to_vec()
            .map_err(CwtError::CoseSerialization)
    }

    /// Return the structurally valid, but unauthenticated, CWT header.
    ///
    /// Its values originate from untrusted input and must not be used for authorization or identity decisions before
    /// this CWT has been converted into a [`VerifiedCwt`].
    pub fn dangerous_header_unverified(&self) -> &CwtHeader {
        &self.unverified_header
    }

    pub fn as_cose(&self) -> &TypedCose<CoseSign1, T> {
        &self.cose
    }

    pub fn into_cose(self) -> TypedCose<CoseSign1, T> {
        self.cose
    }
}

impl<T> UnverifiedCwt<T>
where
    T: DeserializeOwned,
{
    /// Verify the certificate path and COSE signature, then deserialize the authenticated payload.
    pub fn into_verified_against_trust_anchors(
        self,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: CertificateUsage,
    ) -> Result<VerifiedCwt<T>, CwtError> {
        let payload = self.cose.verify_against_trust_anchors_with_chain(
            self.unverified_header.x5chain.clone(),
            trust_anchors,
            time,
            certificate_usage,
        )?;

        Ok(VerifiedCwt { cwt: self, payload })
    }
}

/// A freshly signed CWT.
///
/// This type cannot be deserialized from untrusted input. Convert it into an [`UnverifiedCwt`] for transport or trust
/// anchor verification.
#[derive(Debug, PartialEq)]
pub struct SignedCwt<T>(UnverifiedCwt<T>);

impl<T> Clone for SignedCwt<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> SignedCwt<T> {
    pub fn to_vec(&self) -> Result<Vec<u8>, CwtError> {
        self.0.to_vec()
    }

    pub fn as_unverified(&self) -> &UnverifiedCwt<T> {
        &self.0
    }

    pub fn into_unverified(self) -> UnverifiedCwt<T> {
        self.0
    }
}

impl<T> From<SignedCwt<T>> for UnverifiedCwt<T> {
    fn from(value: SignedCwt<T>) -> Self {
        value.into_unverified()
    }
}

/// An authenticated CWT together with its parsed header and payload.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedCwt<T> {
    cwt: UnverifiedCwt<T>,
    payload: T,
}

impl<T> VerifiedCwt<T> {
    pub fn header(&self) -> &CwtHeader {
        &self.cwt.unverified_header
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_payload(self) -> T {
        self.payload
    }

    pub fn cwt(&self) -> &UnverifiedCwt<T> {
        &self.cwt
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, CwtError> {
        self.cwt.to_vec()
    }
}

impl<T> From<VerifiedCwt<T>> for UnverifiedCwt<T> {
    fn from(value: VerifiedCwt<T>) -> Self {
        value.cwt
    }
}

impl<T> SignedCwt<T>
where
    T: Serialize,
{
    /// Sign a WRPRC CWT using ES256, including the signing certificate and an `iat` CWT claim in the protected header.
    pub async fn sign_with_certificate<K: EcdsaKey>(
        payload: &T,
        key_pair: &KeyPair<K>,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Self, CwtError> {
        let claims = ClaimsSet {
            issued_at: Some(Timestamp::WholeSeconds(time.generate().timestamp())),
            ..Default::default()
        };

        let protected_header = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .value(
                CWT_CLAIMS_HEADER_LABEL,
                claims.to_cbor_value().map_err(CwtError::CoseSerialization)?,
            )
            .build();

        let cose =
            TypedCose::sign_with_protected_header(payload, protected_header, Header::default(), key_pair, true).await?;

        Ok(Self(UnverifiedCwt::try_from(cose)?))
    }
}

impl<T> TryFrom<CoseSign1> for UnverifiedCwt<T> {
    type Error = CwtError;

    fn try_from(value: CoseSign1) -> Result<Self, Self::Error> {
        Self::try_from(TypedCose::from(value))
    }
}

impl<T> TryFrom<TypedCose<CoseSign1, T>> for UnverifiedCwt<T> {
    type Error = CwtError;

    fn try_from(value: TypedCose<CoseSign1, T>) -> Result<Self, Self::Error> {
        let unverified_header = validate_header(&value)?;
        Ok(Self {
            cose: value,
            unverified_header,
        })
    }
}

fn validate_header<T>(cose: &TypedCose<CoseSign1, T>) -> Result<CwtHeader, CwtError> {
    if cose.as_inner().payload.is_none() {
        return Err(CoseError::MissingPayload.into());
    }

    let protected = cose.protected_header();
    let unprotected = &cose.as_inner().unprotected;

    for label in [
        Label::Int(COSE_ALGORITHM_HEADER_LABEL),
        Label::Int(COSE_CONTENT_TYPE_HEADER_LABEL),
        Label::Int(CWT_CLAIMS_HEADER_LABEL),
        Label::Int(COSE_X5CHAIN_HEADER_LABEL),
    ] {
        if header_contains_label(unprotected, &label) {
            return Err(CwtError::UnprotectedHeaderParameter(label));
        }
    }

    cose.validate_algorithm()?;

    match protected.content_type.as_ref() {
        Some(ContentType::Text(content_type)) if content_type == WRPRC_CWT_CONTENT_TYPE => {}
        Some(content_type) => return Err(CwtError::UnexpectedContentType(content_type.clone())),
        None => return Err(CwtError::MissingContentType),
    }

    let x5chain = x5chain_from_header(protected)?;
    let claims = protected
        .rest
        .iter()
        .find(|(label, _)| label == &Label::Int(CWT_CLAIMS_HEADER_LABEL))
        .map(|(_, value)| ClaimsSet::from_cbor_value(value.clone()).map_err(CwtError::CoseSerialization))
        .transpose()?;

    if claims
        .as_ref()
        .and_then(|claims| claims.issued_at.as_ref())
        .is_some_and(|issued_at| matches!(issued_at, Timestamp::FractionalSeconds(value) if !value.is_finite()))
    {
        return Err(CwtError::InvalidIssuedAt);
    }

    Ok(CwtHeader { claims, x5chain })
}

fn header_contains_label(header: &Header, label: &Label) -> bool {
    match label {
        Label::Int(COSE_ALGORITHM_HEADER_LABEL) => header.alg.is_some(),
        Label::Int(COSE_CONTENT_TYPE_HEADER_LABEL) => header.content_type.is_some(),
        _ => header.rest.iter().any(|(candidate, _)| candidate == label),
    }
}

fn parse_cose_sign1(bytes: &[u8]) -> Result<CoseSign1, CwtError> {
    let value = Value::from_slice(bytes).map_err(CwtError::CoseSerialization)?;
    let value = match value {
        Value::Tag(tag, value) if tag == iana::CborTag::CoseSign1 as u64 => *value,
        Value::Tag(tag, _) => return Err(CwtError::UnexpectedCborTag(tag)),
        value => value,
    };

    let Value::Array(elements) = value else {
        return Err(CwtError::InvalidCoseSign1("top-level CBOR value must be an array"));
    };
    if elements.len() != 4 {
        return Err(CwtError::InvalidCoseSign1("top-level array must contain four items"));
    }

    let mut elements = elements.into_iter();
    let protected = parse_protected_header(
        elements
            .next()
            .ok_or(CwtError::InvalidCoseSign1("missing protected header"))?,
    )?;
    let unprotected = parse_header_map(
        elements
            .next()
            .ok_or(CwtError::InvalidCoseSign1("missing unprotected header"))?,
        "unprotected header must be a CBOR map",
    )?;
    let payload = match elements.next().ok_or(CwtError::InvalidCoseSign1("missing payload"))? {
        Value::Bytes(payload) => Some(payload),
        Value::Null => None,
        _ => return Err(CwtError::InvalidCoseSign1("payload must be a byte string or null")),
    };
    let signature = match elements.next().ok_or(CwtError::InvalidCoseSign1("missing signature"))? {
        Value::Bytes(signature) => signature,
        _ => return Err(CwtError::InvalidCoseSign1("signature must be a byte string")),
    };

    Ok(CoseSign1 {
        protected,
        unprotected,
        payload,
        signature,
    })
}

fn parse_protected_header(value: Value) -> Result<ProtectedHeader, CwtError> {
    let Value::Bytes(original_data) = value else {
        return Err(CwtError::InvalidCoseSign1(
            "protected header must be encoded as a byte string",
        ));
    };

    let header = if original_data.is_empty() {
        Header::default()
    } else {
        let value = Value::from_slice(&original_data).map_err(CwtError::CoseSerialization)?;
        parse_header_map(value, "protected header byte string must contain a CBOR map")?
    };

    Ok(ProtectedHeader {
        original_data: Some(original_data),
        header,
    })
}

/// Parse a COSE header while permitting the ETSI `rc-wrp+cwt` content type. coset 0.3 rejects text content types
/// without a slash, even though COSE permits arbitrary non-empty text strings and ETSI mandates this value.
fn parse_header_map(value: Value, invalid_map_message: &'static str) -> Result<Header, CwtError> {
    let Value::Map(entries) = value else {
        return Err(CwtError::InvalidCoseSign1(invalid_map_message));
    };

    let mut seen = BTreeSet::new();
    let mut content_type = None;
    let mut remaining_entries = Vec::with_capacity(entries.len());

    for (key, value) in entries {
        let label = Label::from_cbor_value(key.clone()).map_err(CwtError::CoseSerialization)?;
        if !seen.insert(label.clone()) {
            return Err(CwtError::CoseSerialization(coset::CoseError::DuplicateMapKey));
        }

        if label == Label::Int(COSE_CONTENT_TYPE_HEADER_LABEL) {
            content_type = Some(ContentType::from_cbor_value(value).map_err(CwtError::CoseSerialization)?);
        } else {
            remaining_entries.push((key, value));
        }
    }

    let mut header = Header::from_cbor_value(Value::Map(remaining_entries)).map_err(CwtError::CoseSerialization)?;
    header.content_type = content_type;
    Ok(header)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use crypto::server_keys::generate::Ca;
    use serde::Deserialize;
    use utils::generator::TimeGenerator;
    use utils::generator::mock::MockTimeGenerator;

    use super::*;
    use crate::serialization::cbor_serialize;

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ToyMessage {
        message: String,
    }

    impl Default for ToyMessage {
        fn default() -> Self {
            Self {
                message: "authenticated payload".to_owned(),
            }
        }
    }

    fn valid_protected_header(key_pair: &KeyPair, claims: Option<Value>) -> Header {
        let mut builder = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()));

        if let Some(claims) = claims {
            builder = builder.value(CWT_CLAIMS_HEADER_LABEL, claims);
        }

        builder.build()
    }

    async fn sign_with_header(header: Header, key_pair: &KeyPair) -> TypedCose<CoseSign1, ToyMessage> {
        TypedCose::sign_with_protected_header(&ToyMessage::default(), header, Header::default(), key_pair, true)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn sign_parse_and_verify_wrprc_cwt() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_reader_mock().unwrap();
        let issued_at = Utc.timestamp_opt(1_700_000_000, 0).unwrap();
        let signed =
            SignedCwt::sign_with_certificate(&ToyMessage::default(), &key_pair, &MockTimeGenerator::new(issued_at))
                .await
                .unwrap();

        let encoded = signed.to_vec().unwrap();
        let parsed = UnverifiedCwt::<ToyMessage>::from_slice(&encoded).unwrap();
        let tagged = Value::Tag(
            iana::CborTag::CoseSign1 as u64,
            Box::new(Value::from_slice(&encoded).unwrap()),
        )
        .to_vec()
        .unwrap();
        UnverifiedCwt::<ToyMessage>::from_slice(&tagged).unwrap();
        let verified = parsed
            .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, CertificateUsage::ReaderAuth)
            .unwrap();

        assert_eq!(verified.payload(), &ToyMessage::default());
        assert_eq!(
            verified.header().issued_at(),
            Some(&Timestamp::WholeSeconds(issued_at.timestamp()))
        );
        assert_eq!(verified.header().x5chain.first(), key_pair.certificate());
    }

    #[tokio::test]
    async fn cwt_claims_and_iat_are_optional_when_parsing() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_reader_mock().unwrap();
        let cose = sign_with_header(valid_protected_header(&key_pair, None), &key_pair).await;
        let cwt = UnverifiedCwt::<ToyMessage>::try_from(cose).unwrap();

        let verified = cwt
            .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, CertificateUsage::ReaderAuth)
            .unwrap();

        assert_eq!(verified.header().claims, None);
        assert_eq!(verified.payload(), &ToyMessage::default());

        let claims_without_iat = ClaimsSet::default().to_cbor_value().unwrap();
        let cose = sign_with_header(valid_protected_header(&key_pair, Some(claims_without_iat)), &key_pair).await;
        let cwt = UnverifiedCwt::try_from(cose).unwrap();
        assert_eq!(cwt.dangerous_header_unverified().issued_at(), None);
    }

    #[tokio::test]
    async fn required_headers_are_rejected_when_missing_or_incorrect() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_reader_mock().unwrap();

        let missing_algorithm = HeaderBuilder::new()
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(missing_algorithm, &key_pair).await),
            Err(CwtError::Cose(CoseError::MissingAlgorithm))
        ));

        let unsupported_algorithm = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES384)
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(unsupported_algorithm, &key_pair).await),
            Err(CwtError::Cose(CoseError::UnsupportedAlgorithm(_)))
        ));

        let missing_content_type = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(missing_content_type, &key_pair).await),
            Err(CwtError::MissingContentType)
        ));

        let incorrect_content_type = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type("application/cwt".to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(incorrect_content_type, &key_pair).await),
            Err(CwtError::UnexpectedContentType(_))
        ));

        let missing_x5chain = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(missing_x5chain, &key_pair).await),
            Err(CwtError::Cose(CoseError::MissingLabel(Label::Int(
                COSE_X5CHAIN_HEADER_LABEL
            ))))
        ));

        let empty_x5chain = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Array(Vec::new()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(empty_x5chain, &key_pair).await),
            Err(CwtError::Cose(CoseError::EmptyCertificateChain))
        ));

        let protected = valid_protected_header(&key_pair, None);
        let unprotected = HeaderBuilder::new()
            .content_type(WRPRC_CWT_CONTENT_TYPE.to_owned())
            .build();
        let cose =
            TypedCose::sign_with_protected_header(&ToyMessage::default(), protected, unprotected, &key_pair, true)
                .await
                .unwrap();
        assert!(matches!(
            UnverifiedCwt::try_from(cose),
            Err(CwtError::UnprotectedHeaderParameter(Label::Int(
                COSE_CONTENT_TYPE_HEADER_LABEL
            )))
        ));
    }

    #[test]
    fn malformed_cose_sign1_is_rejected() {
        let not_an_array = cbor_serialize(&Value::Map(Vec::new())).unwrap();
        assert!(matches!(
            UnverifiedCwt::<ToyMessage>::from_slice(&not_an_array),
            Err(CwtError::InvalidCoseSign1(_))
        ));

        let protected_header_not_bytes = cbor_serialize(&Value::Array(vec![
            Value::Map(Vec::new()),
            Value::Map(Vec::new()),
            Value::Bytes(Vec::new()),
            Value::Bytes(vec![0; 64]),
        ]))
        .unwrap();
        assert!(matches!(
            UnverifiedCwt::<ToyMessage>::from_slice(&protected_header_not_bytes),
            Err(CwtError::InvalidCoseSign1(_))
        ));
    }

    #[tokio::test]
    async fn invalid_cwt_claims_are_rejected() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_reader_mock().unwrap();

        let claims_not_a_map = valid_protected_header(&key_pair, Some(Value::Array(Vec::new())));
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(claims_not_a_map, &key_pair).await),
            Err(CwtError::CoseSerialization(_))
        ));

        let invalid_iat = Value::Map(vec![(Value::from(6), Value::Text("not a date".to_owned()))]);
        assert!(matches!(
            UnverifiedCwt::try_from(
                sign_with_header(valid_protected_header(&key_pair, Some(invalid_iat)), &key_pair,).await
            ),
            Err(CwtError::CoseSerialization(_))
        ));

        let non_finite_iat = Value::Map(vec![(Value::from(6), Value::Float(f64::NAN))]);
        assert!(matches!(
            UnverifiedCwt::try_from(
                sign_with_header(valid_protected_header(&key_pair, Some(non_finite_iat)), &key_pair,).await
            ),
            Err(CwtError::InvalidIssuedAt)
        ));
    }

    #[tokio::test]
    async fn duplicate_protected_header_and_claim_keys_are_rejected() {
        let duplicate_content_types = Value::Map(vec![
            (
                Value::from(COSE_CONTENT_TYPE_HEADER_LABEL),
                Value::Text(WRPRC_CWT_CONTENT_TYPE.to_owned()),
            ),
            (
                Value::from(COSE_CONTENT_TYPE_HEADER_LABEL),
                Value::Text(WRPRC_CWT_CONTENT_TYPE.to_owned()),
            ),
        ]);
        let protected = cbor_serialize(&duplicate_content_types).unwrap();
        let payload = cbor_serialize(&ToyMessage::default()).unwrap();
        let encoded = cbor_serialize(&Value::Array(vec![
            Value::Bytes(protected),
            Value::Map(Vec::new()),
            Value::Bytes(payload),
            Value::Bytes(vec![0; 64]),
        ]))
        .unwrap();
        assert!(matches!(
            UnverifiedCwt::<ToyMessage>::from_slice(&encoded),
            Err(CwtError::CoseSerialization(coset::CoseError::DuplicateMapKey))
        ));

        let ca = Ca::generate_mock();
        let key_pair = ca.generate_reader_mock().unwrap();
        let duplicate_iat = Value::Map(vec![
            (Value::from(6), Value::from(1_700_000_000)),
            (Value::from(6), Value::from(1_700_000_001)),
        ]);
        assert!(matches!(
            UnverifiedCwt::try_from(
                sign_with_header(valid_protected_header(&key_pair, Some(duplicate_iat)), &key_pair,).await
            ),
            Err(CwtError::CoseSerialization(coset::CoseError::DuplicateMapKey))
        ));
    }

    #[tokio::test]
    async fn tampered_payload_is_not_returned() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_reader_mock().unwrap();
        let mut cose =
            SignedCwt::sign_with_certificate(&ToyMessage::default(), &key_pair, &MockTimeGenerator::default())
                .await
                .unwrap()
                .into_unverified()
                .into_cose()
                .into_inner();
        cose.payload.as_mut().unwrap()[0] ^= 1;
        let cwt = UnverifiedCwt::<ToyMessage>::try_from(cose).unwrap();

        assert!(matches!(
            cwt.into_verified_against_trust_anchors(
                &TrustAnchors::from(&ca),
                &TimeGenerator,
                CertificateUsage::ReaderAuth,
            ),
            Err(CwtError::Cose(CoseError::EcdsaSignatureVerificationFailed(_)))
        ));
    }
}
