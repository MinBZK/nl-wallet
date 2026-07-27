//! Wallet Relying Party Registration Certificates (WRPRCs) encoded as CWTs.
//!
//! The types in this module implement the WRPRC CWT profile from ETSI TS 119 475; they are not generic CWT
//! containers.

use chrono::DateTime;
use chrono::Utc;
use ciborium::value::Value;
use coset::AsCborValue;
use coset::CborSerializable;
use coset::CoseSign1;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::RegisteredLabel;
use coset::cwt::ClaimsSet;
use coset::cwt::Timestamp;
use coset::iana;
use coset::iana::CborTag;
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
pub struct WrprcCwtHeader {
    pub claims: Option<ClaimsSet>,
    pub x5chain: VecNonEmpty<BorrowingCertificate>,
}

impl WrprcCwtHeader {
    pub fn issued_at(&self) -> Option<&Timestamp> {
        self.claims.as_ref().and_then(|claims| claims.issued_at.as_ref())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum WrprcCwtError {
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
    #[error("missing protected WRPRC CWT type")]
    #[category(critical)]
    MissingType,
    #[error("unexpected protected WRPRC CWT type: {0:?}")]
    #[category(pd)]
    UnexpectedType(Value),
    #[error("WRPRC CWT header parameter {0:?} must be protected")]
    #[category(pd)]
    UnprotectedHeaderParameter(Label),
    #[error("unsupported critical COSE header parameter: {0:?}")]
    #[category(pd)]
    UnsupportedCriticalHeader(RegisteredLabel<iana::HeaderParameter>),
    #[error("WRPRC CWT iat claim is not a finite NumericDate")]
    #[category(critical)]
    InvalidIssuedAt,
}

/// A parsed, but not yet authenticated, WRPRC encoded as a CWT.
///
/// Parsing validates the COSE structure and WRPRC CWT profile, but does not establish the authenticity of the header
/// or payload. Use [`UnverifiedWrprcCwt::into_verified_against_trust_anchors`] before consuming either as trusted
/// input.
#[derive(Debug, PartialEq)]
pub struct UnverifiedWrprcCwt<T> {
    cose: TypedCose<CoseSign1, T>,
    unverified_header: WrprcCwtHeader,
}

impl<T> Clone for UnverifiedWrprcCwt<T> {
    fn clone(&self) -> Self {
        Self {
            cose: self.cose.clone(),
            unverified_header: self.unverified_header.clone(),
        }
    }
}

impl<T> UnverifiedWrprcCwt<T> {
    /// Parse a COSE_Sign1 object that is untagged, tagged as COSE_Sign1, or tagged as CWT around a tagged COSE_Sign1.
    pub fn from_slice(bytes: &[u8]) -> Result<Self, WrprcCwtError> {
        Self::try_from(parse_cose_sign1(bytes)?)
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, WrprcCwtError> {
        self.cose
            .as_ref()
            .clone()
            .to_vec()
            .map_err(WrprcCwtError::CoseSerialization)
    }
}

impl<T> UnverifiedWrprcCwt<T>
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
    ) -> Result<VerifiedWrprcCwt<T>, WrprcCwtError> {
        let payload = self
            .cose
            .verify_against_trust_anchors_with_chain(
                self.unverified_header.x5chain.clone(),
                trust_anchors,
                time,
                certificate_usage,
            )
            .map_err(WrprcCwtError::Cose)?;

        Ok(VerifiedWrprcCwt {
            unverified: self,
            payload,
        })
    }
}

/// A freshly signed WRPRC encoded as a CWT.
///
/// This type cannot be deserialized from untrusted input. Convert it into an [`UnverifiedWrprcCwt`] for transport or
/// trust anchor verification.
#[derive(Debug, PartialEq)]
pub struct SignedWrprcCwt<T>(UnverifiedWrprcCwt<T>);

impl<T> Clone for SignedWrprcCwt<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> SignedWrprcCwt<T> {
    pub fn to_vec(&self) -> Result<Vec<u8>, WrprcCwtError> {
        self.0.to_vec()
    }

    pub fn as_unverified(&self) -> &UnverifiedWrprcCwt<T> {
        &self.0
    }

    pub fn into_unverified(self) -> UnverifiedWrprcCwt<T> {
        self.0
    }
}

impl<T> From<SignedWrprcCwt<T>> for UnverifiedWrprcCwt<T> {
    fn from(value: SignedWrprcCwt<T>) -> Self {
        value.into_unverified()
    }
}

/// An authenticated WRPRC CWT together with its parsed header and payload.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifiedWrprcCwt<T> {
    unverified: UnverifiedWrprcCwt<T>,
    payload: T,
}

impl<T> VerifiedWrprcCwt<T> {
    pub fn header(&self) -> &WrprcCwtHeader {
        &self.unverified.unverified_header
    }

    pub fn payload(&self) -> &T {
        &self.payload
    }

    pub fn into_payload(self) -> T {
        self.payload
    }

    pub fn as_unverified(&self) -> &UnverifiedWrprcCwt<T> {
        &self.unverified
    }

    pub fn to_vec(&self) -> Result<Vec<u8>, WrprcCwtError> {
        self.unverified.to_vec()
    }
}

impl<T> From<VerifiedWrprcCwt<T>> for UnverifiedWrprcCwt<T> {
    fn from(value: VerifiedWrprcCwt<T>) -> Self {
        value.unverified
    }
}

impl<T> SignedWrprcCwt<T>
where
    T: Serialize,
{
    /// Sign a WRPRC CWT using ES256, including the signing certificate and an `iat` claimed signing time in the
    /// protected CWT Claims header parameter.
    pub async fn sign_with_certificate<K: EcdsaKey>(
        payload: &T,
        key_pair: &KeyPair<K>,
        time: &impl Generator<DateTime<Utc>>,
    ) -> Result<Self, WrprcCwtError> {
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
                claims.to_cbor_value().map_err(WrprcCwtError::CoseSerialization)?,
            )
            .build();

        let cose = TypedCose::sign_with_protected_header(payload, protected_header, Header::default(), key_pair, true)
            .await
            .map_err(WrprcCwtError::Cose)?;

        Ok(Self(UnverifiedWrprcCwt::try_from(cose)?))
    }
}

impl<T> TryFrom<CoseSign1> for UnverifiedWrprcCwt<T> {
    type Error = WrprcCwtError;

    fn try_from(value: CoseSign1) -> Result<Self, Self::Error> {
        Self::try_from(TypedCose::from(value))
    }
}

impl<T> TryFrom<TypedCose<CoseSign1, T>> for UnverifiedWrprcCwt<T> {
    type Error = WrprcCwtError;

    fn try_from(value: TypedCose<CoseSign1, T>) -> Result<Self, Self::Error> {
        let unverified_header = validate_wrprc_header(&value)?;
        Ok(Self {
            cose: value,
            unverified_header,
        })
    }
}

fn validate_wrprc_header<T>(cose: &TypedCose<CoseSign1, T>) -> Result<WrprcCwtHeader, WrprcCwtError> {
    if cose.as_ref().payload.is_none() {
        return Err(WrprcCwtError::Cose(CoseError::MissingPayload));
    }

    let protected = cose.protected_header();
    let unprotected = &cose.as_ref().unprotected;

    for label in [
        Label::Int(COSE_ALGORITHM_HEADER_LABEL),
        Label::Int(COSE_CRITICAL_HEADER_LABEL),
        Label::Int(COSE_TYPE_HEADER_LABEL),
        Label::Int(CWT_CLAIMS_HEADER_LABEL),
        Label::Int(COSE_X5CHAIN_HEADER_LABEL),
    ] {
        if header_contains_label(unprotected, &label) {
            return Err(WrprcCwtError::UnprotectedHeaderParameter(label));
        }
    }

    for critical_header in &protected.crit {
        if critical_header != &RegisteredLabel::Assigned(iana::HeaderParameter::Alg) {
            return Err(WrprcCwtError::UnsupportedCriticalHeader(critical_header.clone()));
        }
    }

    cose.validate_algorithm().map_err(WrprcCwtError::Cose)?;

    match protected
        .rest
        .iter()
        .find(|(label, _)| label == &Label::Int(COSE_TYPE_HEADER_LABEL))
        .map(|(_, value)| value)
    {
        Some(Value::Text(cwt_type)) if cwt_type == WRPRC_CWT_TYPE => {}
        Some(value) => return Err(WrprcCwtError::UnexpectedType(value.clone())),
        None => return Err(WrprcCwtError::MissingType),
    }

    let x5chain = x5chain_from_header(protected).map_err(WrprcCwtError::Cose)?;
    let claims = protected
        .rest
        .iter()
        .find(|(label, _)| label == &Label::Int(CWT_CLAIMS_HEADER_LABEL))
        .map(|(_, value)| ClaimsSet::from_cbor_value(value.clone()).map_err(WrprcCwtError::CoseSerialization))
        .transpose()?;

    if claims
        .as_ref()
        .and_then(|claims| claims.issued_at.as_ref())
        .is_some_and(|issued_at| matches!(issued_at, Timestamp::FractionalSeconds(value) if !value.is_finite()))
    {
        return Err(WrprcCwtError::InvalidIssuedAt);
    }

    Ok(WrprcCwtHeader { claims, x5chain })
}

fn header_contains_label(header: &Header, label: &Label) -> bool {
    match label {
        Label::Int(COSE_ALGORITHM_HEADER_LABEL) => header.alg.is_some(),
        Label::Int(COSE_CRITICAL_HEADER_LABEL) => !header.crit.is_empty(),
        _ => header.rest.iter().any(|(candidate, _)| candidate == label),
    }
}

fn parse_cose_sign1(bytes: &[u8]) -> Result<CoseSign1, WrprcCwtError> {
    let value = Value::from_slice(bytes).map_err(WrprcCwtError::CoseSerialization)?;
    let value = match value {
        Value::Tag(tag, value) if tag == CborTag::Cwt as u64 => match *value {
            Value::Tag(tag, value) if tag == CborTag::CoseSign1 as u64 => *value,
            Value::Tag(tag, _) => return Err(WrprcCwtError::UnexpectedCborTag(tag)),
            _ => {
                return Err(WrprcCwtError::InvalidCoseSign1(
                    "CWT tag must be followed by a COSE_Sign1 tag",
                ));
            }
        },
        Value::Tag(tag, value) if tag == CborTag::CoseSign1 as u64 => *value,
        Value::Tag(tag, _) => return Err(WrprcCwtError::UnexpectedCborTag(tag)),
        value => value,
    };

    CoseSign1::from_cbor_value(value).map_err(WrprcCwtError::CoseSerialization)
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

    fn valid_wrprc_protected_header(key_pair: &KeyPair, claims: Option<Value>) -> Header {
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
        let signed = SignedWrprcCwt::sign_with_certificate(
            &ToyMessage::default(),
            &key_pair,
            &MockTimeGenerator::new(issued_at),
        )
        .await
        .unwrap();

        let encoded = signed.to_vec().unwrap();
        let parsed = UnverifiedWrprcCwt::<ToyMessage>::from_slice(&encoded).unwrap();
        let tagged = Value::Tag(
            CborTag::CoseSign1 as u64,
            Box::new(Value::from_slice(&encoded).unwrap()),
        )
        .to_vec()
        .unwrap();
        UnverifiedWrprcCwt::<ToyMessage>::from_slice(&tagged).unwrap();
        let cwt_tagged = Value::Tag(CborTag::Cwt as u64, Box::new(Value::from_slice(&tagged).unwrap()))
            .to_vec()
            .unwrap();
        UnverifiedWrprcCwt::<ToyMessage>::from_slice(&cwt_tagged).unwrap();
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
    async fn wrprc_cwt_claims_and_iat_are_optional_when_parsing() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
        let cose = sign_with_header(valid_wrprc_protected_header(&key_pair, None), &key_pair).await;
        let cwt = UnverifiedWrprcCwt::<ToyMessage>::try_from(cose).unwrap();

        let verified = cwt
            .into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, None)
            .unwrap();

        assert_eq!(verified.header().claims, None);
        assert_eq!(verified.payload(), &ToyMessage::default());

        let claims_without_iat = ClaimsSet::default().to_cbor_value().unwrap();
        let cose = sign_with_header(
            valid_wrprc_protected_header(&key_pair, Some(claims_without_iat)),
            &key_pair,
        )
        .await;
        let cwt = UnverifiedWrprcCwt::try_from(cose).unwrap();
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
            UnverifiedWrprcCwt::try_from(sign_with_header(missing_algorithm, &key_pair).await),
            Err(WrprcCwtError::Cose(CoseError::MissingAlgorithm))
        ));

        let unsupported_algorithm = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES384)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(unsupported_algorithm, &key_pair).await),
            Err(WrprcCwtError::Cose(CoseError::UnsupportedAlgorithm(_)))
        ));

        let missing_type = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .content_type("application/cwt".to_owned())
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(missing_type, &key_pair).await),
            Err(WrprcCwtError::MissingType)
        ));

        let incorrect_type = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text("application/cwt".to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Bytes(key_pair.certificate().to_vec()))
            .build();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(incorrect_type, &key_pair).await),
            Err(WrprcCwtError::UnexpectedType(_))
        ));

        let missing_x5chain = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .build();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(missing_x5chain, &key_pair).await),
            Err(WrprcCwtError::Cose(CoseError::MissingLabel(Label::Int(
                COSE_X5CHAIN_HEADER_LABEL
            ))))
        ));

        let empty_x5chain = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(COSE_X5CHAIN_HEADER_LABEL, Value::Array(Vec::new()))
            .build();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(empty_x5chain, &key_pair).await),
            Err(WrprcCwtError::Cose(CoseError::EmptyCertificateChain))
        ));

        let single_certificate_array = HeaderBuilder::new()
            .algorithm(iana::Algorithm::ES256)
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .value(
                COSE_X5CHAIN_HEADER_LABEL,
                Value::Array(vec![Value::Bytes(key_pair.certificate().to_vec())]),
            )
            .build();
        let cwt = UnverifiedWrprcCwt::try_from(sign_with_header(single_certificate_array, &key_pair).await).unwrap();
        assert_eq!(cwt.unverified_header.x5chain.first(), key_pair.certificate());

        let protected = valid_wrprc_protected_header(&key_pair, None);
        let unprotected = HeaderBuilder::new()
            .value(COSE_TYPE_HEADER_LABEL, Value::Text(WRPRC_CWT_TYPE.to_owned()))
            .build();
        let cose =
            TypedCose::sign_with_protected_header(&ToyMessage::default(), protected, unprotected, &key_pair, true)
                .await
                .unwrap();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(cose),
            Err(WrprcCwtError::UnprotectedHeaderParameter(Label::Int(
                COSE_TYPE_HEADER_LABEL
            )))
        ));
    }

    #[tokio::test]
    async fn critical_headers_are_protected_and_understood() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();

        let mut supported = valid_wrprc_protected_header(&key_pair, None);
        supported
            .crit
            .push(RegisteredLabel::Assigned(iana::HeaderParameter::Alg));
        UnverifiedWrprcCwt::try_from(sign_with_header(supported, &key_pair).await).unwrap();

        let mut unsupported = valid_wrprc_protected_header(&key_pair, None);
        unsupported.key_id = vec![1];
        unsupported
            .crit
            .push(RegisteredLabel::Assigned(iana::HeaderParameter::Kid));
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(unsupported, &key_pair).await),
            Err(WrprcCwtError::UnsupportedCriticalHeader(RegisteredLabel::Assigned(
                iana::HeaderParameter::Kid
            )))
        ));

        let protected = valid_wrprc_protected_header(&key_pair, None);
        let unprotected = HeaderBuilder::new().add_critical(iana::HeaderParameter::Alg).build();
        let cose =
            TypedCose::sign_with_protected_header(&ToyMessage::default(), protected, unprotected, &key_pair, true)
                .await
                .unwrap();
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(cose),
            Err(WrprcCwtError::UnprotectedHeaderParameter(Label::Int(
                COSE_CRITICAL_HEADER_LABEL
            )))
        ));
    }

    #[test]
    fn malformed_cose_sign1_is_rejected() {
        let not_an_array = cbor_serialize(&Value::Map(Vec::new())).unwrap();
        assert!(matches!(
            UnverifiedWrprcCwt::<ToyMessage>::from_slice(&not_an_array),
            Err(WrprcCwtError::CoseSerialization(_))
        ));

        let protected_header_not_bytes = cbor_serialize(&Value::Array(vec![
            Value::Map(Vec::new()),
            Value::Map(Vec::new()),
            Value::Bytes(Vec::new()),
            Value::Bytes(vec![0; 64]),
        ]))
        .unwrap();
        assert!(matches!(
            UnverifiedWrprcCwt::<ToyMessage>::from_slice(&protected_header_not_bytes),
            Err(WrprcCwtError::CoseSerialization(_))
        ));

        let cwt_tag_without_cose_tag = Value::Tag(
            CborTag::Cwt as u64,
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
            UnverifiedWrprcCwt::<ToyMessage>::from_slice(&cwt_tag_without_cose_tag),
            Err(WrprcCwtError::InvalidCoseSign1(_))
        ));
    }

    #[tokio::test]
    async fn invalid_cwt_claims_are_rejected() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();

        let claims_not_a_map = valid_wrprc_protected_header(&key_pair, Some(Value::Array(Vec::new())));
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(sign_with_header(claims_not_a_map, &key_pair).await),
            Err(WrprcCwtError::CoseSerialization(_))
        ));

        let invalid_iat = Value::Map(vec![(Value::from(6), Value::Text("not a date".to_owned()))]);
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(
                sign_with_header(valid_wrprc_protected_header(&key_pair, Some(invalid_iat)), &key_pair,).await
            ),
            Err(WrprcCwtError::CoseSerialization(_))
        ));

        let non_finite_iat = Value::Map(vec![(Value::from(6), Value::Float(f64::NAN))]);
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(
                sign_with_header(valid_wrprc_protected_header(&key_pair, Some(non_finite_iat)), &key_pair,).await
            ),
            Err(WrprcCwtError::InvalidIssuedAt)
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
            UnverifiedWrprcCwt::<ToyMessage>::from_slice(&encoded),
            Err(WrprcCwtError::CoseSerialization(coset::CoseError::DuplicateMapKey))
        ));

        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
        let duplicate_iat = Value::Map(vec![
            (Value::from(6), Value::from(1_700_000_000)),
            (Value::from(6), Value::from(1_700_000_001)),
        ]);
        assert!(matches!(
            UnverifiedWrprcCwt::try_from(
                sign_with_header(valid_wrprc_protected_header(&key_pair, Some(duplicate_iat)), &key_pair,).await
            ),
            Err(WrprcCwtError::CoseSerialization(coset::CoseError::DuplicateMapKey))
        ));
    }

    #[tokio::test]
    async fn tampered_payload_is_not_returned() {
        let ca = Ca::generate_wrpac_mock_ca().unwrap();
        let key_pair = ca.generate_wrpac_verifier_mock().unwrap();
        let mut cose =
            SignedWrprcCwt::sign_with_certificate(&ToyMessage::default(), &key_pair, &MockTimeGenerator::default())
                .await
                .unwrap()
                .into_unverified()
                .cose
                .into_inner();
        cose.payload.as_mut().unwrap()[0] ^= 1;
        let cwt = UnverifiedWrprcCwt::<ToyMessage>::try_from(cose).unwrap();

        assert!(matches!(
            cwt.into_verified_against_trust_anchors(&TrustAnchors::from(&ca), &TimeGenerator, None,),
            Err(WrprcCwtError::Cose(CoseError::EcdsaSignatureVerificationFailed(_)))
        ));
    }

    #[test]
    fn errors_with_untrusted_header_values_are_personal_data() {
        for error in [
            WrprcCwtError::UnexpectedType(Value::Text("personal data".to_owned())),
            WrprcCwtError::UnprotectedHeaderParameter(Label::Text("personal data".to_owned())),
            WrprcCwtError::UnsupportedCriticalHeader(RegisteredLabel::Text("personal data".to_owned())),
            WrprcCwtError::Cose(CoseError::UnsupportedAlgorithm(coset::Algorithm::Text(
                "personal data".to_owned(),
            ))),
        ] {
            assert_eq!(error.category(), Category::PersonalData);
        }
    }

    #[test]
    fn structural_errors_without_input_values_are_critical() {
        assert_eq!(
            WrprcCwtError::CoseSerialization(coset::CoseError::DuplicateMapKey).category(),
            Category::Critical
        );
        assert_eq!(
            WrprcCwtError::InvalidCoseSign1("invalid structure").category(),
            Category::Critical
        );
        assert_eq!(WrprcCwtError::UnexpectedCborTag(42).category(), Category::Critical);
    }
}
