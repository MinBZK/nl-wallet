use chrono::DateTime;
use chrono::Utc;
use ciborium::value::Value;
use coset::AsCborValue;
use coset::CborSerializable;
use coset::CoseSign1;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::ProtectedHeader;
use coset::RegisteredLabel;
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

/// ETSI TS 119 475 type for a Wallet Relying Party Registration Certificate encoded as a CWT.
pub const WRPRC_CWT_TYPE: &str = "rc-wrp+cwt";

/// CWT Claims COSE header parameter, registered by RFC 9597.
pub const CWT_CLAIMS_HEADER_LABEL: i64 = 15;

const COSE_ALGORITHM_HEADER_LABEL: i64 = 1;
const COSE_CRITICAL_HEADER_LABEL: i64 = 2;
/// COSE `typ` header parameter, registered by RFC 9596.
const COSE_TYPE_HEADER_LABEL: i64 = 16;

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
    #[error("COSE error: {0}")]
    Cose(#[source] CoseError),
    #[error("could not decode or encode the COSE_Sign1 object: {0}")]
    #[category(critical)]
    CoseSerialization(#[source] coset::CoseError),
    #[error("invalid COSE_Sign1 structure: {0}")]
    #[category(critical)]
    InvalidCoseSign1(&'static str),
    #[error("unexpected CBOR tag for COSE_Sign1: {0}")]
    #[category(critical)]
    UnexpectedCborTag(u64),
    #[error("missing protected CWT type")]
    #[category(critical)]
    MissingType,
    #[error("unexpected protected CWT type: {0:?}")]
    #[category(pd)]
    UnexpectedType(Value),
    #[error("CWT header parameter {0:?} must be protected")]
    #[category(pd)]
    UnprotectedHeaderParameter(Label),
    #[error("unsupported critical COSE header parameter: {0:?}")]
    #[category(pd)]
    UnsupportedCriticalHeader(RegisteredLabel<iana::HeaderParameter>),
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
    /// Parse a COSE_Sign1 object that is untagged, tagged as COSE_Sign1, or tagged as CWT around a tagged COSE_Sign1.
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
}

impl<T> UnverifiedCwt<T>
where
    T: DeserializeOwned,
{
    /// Verify the certificate path and COSE signature, then deserialize the authenticated payload.
    ///
    /// A `None` certificate usage still validates the certificate path, but does not require a profile-specific
    /// extended key usage.
    pub fn into_verified_against_trust_anchors(
        self,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
    ) -> Result<VerifiedCwt<T>, CwtError> {
        let payload = self
            .cose
            .verify_against_trust_anchors_with_chain(
                self.unverified_header.x5chain.clone(),
                trust_anchors,
                time,
                certificate_usage,
            )
            .map_err(CwtError::Cose)?;

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
    /// Sign a WRPRC CWT using ES256, including the signing certificate and an `iat` claimed signing time in the
    /// protected CWT Claims header parameter.
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
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .value(
                CWT_CLAIMS_HEADER_LABEL,
                claims.to_cbor_value().map_err(CwtError::CoseSerialization)?,
            )
            .build();

        let cose = TypedCose::sign_with_protected_header(payload, protected_header, Header::default(), key_pair, true)
            .await
            .map_err(CwtError::Cose)?;

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
        return Err(CwtError::Cose(CoseError::MissingPayload));
    }

    let protected = cose.protected_header();
    let unprotected = &cose.as_inner().unprotected;

    for label in [
        Label::Int(COSE_ALGORITHM_HEADER_LABEL),
        Label::Int(COSE_CRITICAL_HEADER_LABEL),
        Label::Int(COSE_TYPE_HEADER_LABEL),
        Label::Int(CWT_CLAIMS_HEADER_LABEL),
        Label::Int(COSE_X5CHAIN_HEADER_LABEL),
    ] {
        if header_contains_label(unprotected, &label) {
            return Err(CwtError::UnprotectedHeaderParameter(label));
        }
    }

    for critical_header in &protected.crit {
        if critical_header != &RegisteredLabel::Assigned(iana::HeaderParameter::Alg) {
            return Err(CwtError::UnsupportedCriticalHeader(critical_header.clone()));
        }
    }

    cose.validate_algorithm().map_err(CwtError::Cose)?;

    match protected
        .rest
        .iter()
        .find(|(label, _)| label == &Label::Int(COSE_TYPE_HEADER_LABEL))
        .map(|(_, value)| value)
    {
        Some(Value::Text(cwt_type)) if cwt_type == WRPRC_CWT_TYPE => {}
        Some(value) => return Err(CwtError::UnexpectedType(value.clone())),
        None => return Err(CwtError::MissingType),
    }

    let x5chain = x5chain_from_header(protected).map_err(CwtError::Cose)?;
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
        Label::Int(COSE_CRITICAL_HEADER_LABEL) => !header.crit.is_empty(),
        _ => header.rest.iter().any(|(candidate, _)| candidate == label),
    }
}

fn parse_cose_sign1(bytes: &[u8]) -> Result<CoseSign1, CwtError> {
    let value = Value::from_slice(bytes).map_err(CwtError::CoseSerialization)?;
    let value = match value {
        Value::Tag(tag, value) if tag == iana::CborTag::Cwt as u64 => match *value {
            Value::Tag(tag, value) if tag == iana::CborTag::CoseSign1 as u64 => *value,
            Value::Tag(tag, _) => return Err(CwtError::UnexpectedCborTag(tag)),
            _ => {
                return Err(CwtError::InvalidCoseSign1(
                    "CWT tag must be followed by a COSE_Sign1 tag",
                ));
            }
        },
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

fn parse_header_map(value: Value, invalid_map_message: &'static str) -> Result<Header, CwtError> {
    if !matches!(&value, Value::Map(_)) {
        return Err(CwtError::InvalidCoseSign1(invalid_map_message));
    }

    Header::from_cbor_value(value).map_err(CwtError::CoseSerialization)
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use crypto::server_keys::generate::Ca;
    use error_category::Category;
    use error_category::ErrorCategory as _;
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
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
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
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
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
        let cwt_tagged = Value::Tag(iana::CborTag::Cwt as u64, Box::new(Value::from_slice(&tagged).unwrap()))
            .to_vec()
            .unwrap();
        UnverifiedCwt::<ToyMessage>::from_slice(&cwt_tagged).unwrap();
        let verified = parsed
            .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, None)
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
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
        let cose = sign_with_header(valid_protected_header(&key_pair, None), &key_pair).await;
        let cwt = UnverifiedCwt::<ToyMessage>::try_from(cose).unwrap();

        let verified = cwt
            .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, None)
            .unwrap();

        assert_eq!(verified.header().claims, None);
        assert_eq!(verified.payload(), &ToyMessage::default());

        let claims_without_iat = ClaimsSet::default().to_cbor_value().unwrap();
        let cose = sign_with_header(valid_protected_header(&key_pair, Some(claims_without_iat)), &key_pair).await;
        let cwt = UnverifiedCwt::try_from(cose).unwrap();
        assert_eq!(cwt.unverified_header.issued_at(), None);
    }

    #[tokio::test]
    async fn required_headers_are_rejected_when_missing_or_incorrect() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();

        let missing_algorithm = HeaderBuilder::new()
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(missing_algorithm, &key_pair).await),
            Err(CwtError::Cose(CoseError::MissingAlgorithm))
        ));

        let unsupported_algorithm = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES384)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(unsupported_algorithm, &key_pair).await),
            Err(CwtError::Cose(CoseError::UnsupportedAlgorithm(_)))
        ));

        let missing_type = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type("application/cwt".to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(missing_type, &key_pair).await),
            Err(CwtError::MissingType)
        ));

        let incorrect_type = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text("application/cwt".to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(incorrect_type, &key_pair).await),
            Err(CwtError::UnexpectedType(_))
        ));

        let missing_x5chain = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(missing_x5chain, &key_pair).await),
            Err(CwtError::Cose(CoseError::MissingLabel(Label::Int(
                COSE_X5CHAIN_HEADER_LABEL
            ))))
        ));

        let empty_x5chain = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Array(Vec::new()))
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(empty_x5chain, &key_pair).await),
            Err(CwtError::Cose(CoseError::EmptyCertificateChain))
        ));

        let single_certificate_array = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(
                COSE_X5CHAIN_HEADER_LABEL,
                Value::Array(vec![Value::Bytes(key_pair.certificate().to_vec())]),
            )
            .build();
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(single_certificate_array, &key_pair).await),
            Err(CwtError::Cose(CoseError::CertificateChainTooShort(1)))
        ));

        let protected = valid_protected_header(&key_pair, None);
        let unprotected = HeaderBuilder::new()
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .build();
        let cose =
            TypedCose::sign_with_protected_header(&ToyMessage::default(), protected, unprotected, &key_pair, true)
                .await
                .unwrap();
        assert!(matches!(
            UnverifiedCwt::try_from(cose),
            Err(CwtError::UnprotectedHeaderParameter(Label::Int(COSE_TYPE_HEADER_LABEL)))
        ));
    }

    #[tokio::test]
    async fn critical_headers_are_protected_and_understood() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();

        let mut supported = valid_protected_header(&key_pair, None);
        supported
            .crit
            .push(RegisteredLabel::Assigned(iana::HeaderParameter::Alg));
        UnverifiedCwt::try_from(sign_with_header(supported, &key_pair).await).unwrap();

        let mut unsupported = valid_protected_header(&key_pair, None);
        unsupported.key_id = vec![1];
        unsupported
            .crit
            .push(RegisteredLabel::Assigned(iana::HeaderParameter::Kid));
        assert!(matches!(
            UnverifiedCwt::try_from(sign_with_header(unsupported, &key_pair).await),
            Err(CwtError::UnsupportedCriticalHeader(RegisteredLabel::Assigned(
                iana::HeaderParameter::Kid
            )))
        ));

        let protected = valid_protected_header(&key_pair, None);
        let unprotected = HeaderBuilder::new().add_critical(iana::HeaderParameter::Alg).build();
        let cose =
            TypedCose::sign_with_protected_header(&ToyMessage::default(), protected, unprotected, &key_pair, true)
                .await
                .unwrap();
        assert!(matches!(
            UnverifiedCwt::try_from(cose),
            Err(CwtError::UnprotectedHeaderParameter(Label::Int(
                COSE_CRITICAL_HEADER_LABEL
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

        let cwt_tag_without_cose_tag = Value::Tag(
            iana::CborTag::Cwt as u64,
            Box::new(Value::Array(vec![
                Value::Bytes(Vec::new()),
                Value::Map(Vec::new()),
                Value::Bytes(Vec::new()),
                Value::Bytes(vec![0; 64]),
            ])),
        )
        .to_vec()
        .unwrap();
        assert!(matches!(
            UnverifiedCwt::<ToyMessage>::from_slice(&cwt_tag_without_cose_tag),
            Err(CwtError::InvalidCoseSign1(_))
        ));
    }

    #[tokio::test]
    async fn invalid_cwt_claims_are_rejected() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();

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
        let duplicate_types = Value::Map(vec![
            (
                Value::from(COSE_TYPE_HEADER_LABEL),
                Value::Text(WRPRC_CWT_TYPE.to_owned()),
            ),
            (
                Value::from(COSE_TYPE_HEADER_LABEL),
                Value::Text(WRPRC_CWT_TYPE.to_owned()),
            ),
        ]);
        let protected = cbor_serialize(&duplicate_types).unwrap();
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

        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
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
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
        let mut cose =
            SignedCwt::sign_with_certificate(&ToyMessage::default(), &key_pair, &MockTimeGenerator::default())
                .await
                .unwrap()
                .into_unverified()
                .cose
                .into_inner();
        cose.payload.as_mut().unwrap()[0] ^= 1;
        let cwt = UnverifiedCwt::<ToyMessage>::try_from(cose).unwrap();

        assert!(matches!(
            cwt.into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, None,),
            Err(CwtError::Cose(CoseError::EcdsaSignatureVerificationFailed(_)))
        ));
    }

    #[test]
    fn errors_with_untrusted_header_values_are_personal_data() {
        for error in [
            CwtError::UnexpectedType(Value::Text("personal data".to_owned())),
            CwtError::UnprotectedHeaderParameter(Label::Text("personal data".to_owned())),
            CwtError::UnsupportedCriticalHeader(RegisteredLabel::Text("personal data".to_owned())),
            CwtError::Cose(CoseError::UnsupportedAlgorithm(coset::Algorithm::Text(
                "personal data".to_owned(),
            ))),
        ] {
            assert_eq!(error.category(), Category::PersonalData);
        }
    }

    #[test]
    fn structural_errors_without_input_values_are_critical() {
        assert_eq!(
            CwtError::CoseSerialization(coset::CoseError::DuplicateMapKey).category(),
            Category::Critical
        );
        assert_eq!(
            CwtError::InvalidCoseSign1("invalid structure").category(),
            Category::Critical
        );
        assert_eq!(CwtError::UnexpectedCborTag(42).category(), Category::Critical);
    }
}
