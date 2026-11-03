use std::num::NonZeroUsize;

use chrono::DateTime;
use chrono::Utc;
use ciborium::tag::Required;
use ciborium::value::Value;
use coset::CoseSign1;
use coset::Header;
use coset::HeaderBuilder;
use coset::Label;
use coset::ProtectedHeader;
use coset::SignatureContext;
use coset::iana;
use coset::sig_structure_data;
use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateExtendedKeyUsage;
use crypto::x509::crl::CertificateCrlVerificationError;
use crypto::x509::crl::CertificateCrlVerifier;
use crypto::x509::crl::CrlFetcher;
use serde::Serialize;
use serde::de::DeserializeOwned;
use utils::generator::Generator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::CborIntMap;
use crate::error::CoseError;
use crate::message::TypedCose;
use crate::serialization::cbor_serialize;

/// COSE header label for `x5chain`, defined in RFC 9360.
pub const COSE_X5CHAIN_HEADER_LABEL: i64 = 33;

/// COSE header label for `typ`, defined in RFC 9596.
pub const COSE_TYP_HEADER_LABEL: i64 = 16;

pub trait CoseTyp {
    const TYP: &'static str;

    fn is_valid_typ(header_typ: Option<&Value>) -> Result<(), CoseError> {
        // The `typ` header contains a media type, which is case-insensitive and consists of ASCII characters only.
        match header_typ {
            Some(Value::Text(typ)) if typ.eq_ignore_ascii_case(Self::TYP) => Ok(()),
            header_typ => Err(CoseError::UnexpectedTyp(Self::TYP, header_typ.cloned())),
        }
    }
}

impl<T: CoseTyp> CoseTyp for CborIntMap<T> {
    const TYP: &'static str = T::TYP;
}

/// A tagged COSE_Sign1 structure is identified by the CBOR tag 18.
///
/// ```cddl
/// COSE_Sign1_Tagged = #6.18(COSE_Sign1)
/// ```
///
/// <https://www.rfc-editor.org/rfc/rfc9052.html#section-4.2>
pub type TaggedCoseSign1<T> = Required<TypedCose<CoseSign1, T>, { iana::CborTag::CoseSign1 as u64 }>;

pub fn header_with_x5chain(chain: &VecNonEmpty<&BorrowingCertificate>) -> Header {
    let encode_certificate = |certificate: &&BorrowingCertificate| Value::Bytes(certificate.to_vec());

    if chain.len() == NonZeroUsize::MIN {
        HeaderBuilder::new()
            .value(COSE_X5CHAIN_HEADER_LABEL, encode_certificate(chain.first()))
            .build()
    } else {
        HeaderBuilder::new()
            .value(
                COSE_X5CHAIN_HEADER_LABEL,
                Value::Array(chain.iter().map(encode_certificate).collect()),
            )
            .build()
    }
}

pub(crate) fn x5chain_from_header(header: &Header) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError> {
    let value = header
        .rest
        .iter()
        .find(|(label, _)| label == &Label::Int(COSE_X5CHAIN_HEADER_LABEL))
        .map(|(_, value)| value)
        .ok_or(CoseError::MissingLabel(Label::Int(COSE_X5CHAIN_HEADER_LABEL)))?;

    match value {
        Value::Bytes(bytes) => Ok(vec_nonempty![
            BorrowingCertificate::from_der(bytes.clone()).map_err(CoseError::CertificateParse)?
        ]),
        Value::Array(items) => {
            if items.is_empty() {
                return Err(CoseError::EmptyCertificateChain);
            }

            let certificates = items
                .iter()
                .map(|item| {
                    item.as_bytes()
                        .ok_or(CoseError::CertificateUnexpectedHeaderType)
                        .and_then(|bytes| {
                            BorrowingCertificate::from_der(bytes.clone()).map_err(CoseError::CertificateParse)
                        })
                })
                .collect::<Result<Vec<_>, CoseError>>()?;
            certificates.try_into().map_err(|_| CoseError::EmptyCertificateChain)
        }
        _ => Err(CoseError::CertificateUnexpectedHeaderType),
    }
}

impl<T> TypedCose<CoseSign1, T> {
    pub async fn sign(payload: &T, unprotected_header: Header, private_key: &impl EcdsaKey) -> Result<Self, CoseError>
    where
        T: Serialize,
    {
        let payload = cbor_serialize(payload).map_err(CoseError::Cbor)?;
        Ok(sign_cose(payload, unprotected_header, private_key).await?.into())
    }

    pub(crate) async fn sign_with_protected_header(
        payload: &T,
        protected_header: Header,
        unprotected_header: Header,
        private_key: &impl EcdsaKey,
    ) -> Result<Self, CoseError>
    where
        T: Serialize,
    {
        let payload = cbor_serialize(payload).map_err(CoseError::Cbor)?;
        Ok(
            sign_cose_with_headers(payload, protected_header, unprotected_header, private_key)
                .await?
                .into(),
        )
    }

    /// Sign a COSE payload and include the key pair's certificate in the unprotected `x5chain` header.
    pub async fn sign_with_certificate<K: EcdsaKey>(payload: &T, key_pair: &KeyPair<K>) -> Result<Self, CoseError>
    where
        T: Serialize,
    {
        let header = header_with_x5chain(&vec_nonempty![key_pair.certificate()]);
        Self::sign(payload, header, key_pair).await
    }

    /// Sign a COSE payload and include the algorithm, the type from [`CoseTyp::TYP`] and the key pair's certificate
    /// (`x5chain`) in the protected header.
    pub async fn sign_with_typ_and_certificate<K: EcdsaKey>(
        payload: &T,
        key_pair: &KeyPair<K>,
    ) -> Result<Self, CoseError>
    where
        T: Serialize + CoseTyp,
    {
        let mut protected_header = header_with_x5chain(&vec_nonempty![key_pair.certificate()]);
        protected_header.alg = Some(coset::Algorithm::Assigned(iana::Algorithm::ES256));
        protected_header
            .rest
            .push((Label::Int(COSE_TYP_HEADER_LABEL), Value::Text(T::TYP.to_owned())));

        Self::sign_with_protected_header(payload, protected_header, Header::default(), key_pair).await
    }

    /// Get the certificate chain from the unprotected `x5chain` header parameter.
    pub fn unprotected_x5chain(&self) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError> {
        x5chain_from_header(&self.as_ref().unprotected)
    }

    /// Get the certificate chain from the protected `x5chain` header parameter.
    pub fn protected_x5chain(&self) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError> {
        x5chain_from_header(self.protected_header())
    }

    /// Verify the certificate path and COSE signature, then deserialize the authenticated payload.
    ///
    /// A `None` certificate usage still validates the certificate path, but does not require a profile-specific
    /// extended key usage.
    ///
    /// Configured trust anchors present among the intermediate certificates are ignored. The leaf certificate
    /// must not itself be a trust anchor.
    pub fn verify_against_trust_anchors(
        &self,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateExtendedKeyUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let x5chain = self.unprotected_x5chain()?;

        self.verify_with_x5chain(&x5chain, trust_anchors, time, certificate_usage)
    }

    /// Verify the certificate path from the protected `x5chain` header and the COSE signature, enforce
    /// [`CoseTyp::TYP`] on the protected `typ` header, then deserialize the authenticated payload. This is the
    /// counterpart of [`Self::sign_with_typ_and_certificate`].
    ///
    /// Requiring a protected `x5chain` integrity protects the end-entity certificate, as RFC 9360 requires and as ISO
    /// 18013-5 (second edition, 12.3.6.3) requires for the MSO revocation list. Note that this differs from mdoc
    /// `IssuerAuth`, which ISO 18013-5 puts in the unprotected header.
    ///
    /// Configured trust anchors present among the intermediate certificates are ignored. The leaf certificate
    /// must not itself be a trust anchor.
    pub fn verify_with_typ_against_trust_anchors(
        &self,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateExtendedKeyUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned + CoseTyp,
    {
        let typ = self
            .protected_header()
            .rest
            .iter()
            .find(|(label, _)| label == &Label::Int(COSE_TYP_HEADER_LABEL))
            .map(|(_, value)| value);
        T::is_valid_typ(typ)?;

        let x5chain = self.protected_x5chain()?;

        self.verify_with_x5chain(&x5chain, trust_anchors, time, certificate_usage)
    }

    /// Verify the certificate path with fail-closed CRL checking and the COSE signature, then deserialize the
    /// authenticated payload.
    ///
    /// Configured trust anchors present among the intermediate certificates are ignored. The leaf certificate
    /// must not itself be a trust anchor.
    pub async fn verify_against_trust_anchors_with_crl(
        &self,
        trust_anchors: &TrustAnchors,
        crl_verifier: &CertificateCrlVerifier<impl CrlFetcher>,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateExtendedKeyUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let x5chain = self.unprotected_x5chain()?;

        self.verify_against_trust_anchors_with_chain_and_crl(
            x5chain,
            trust_anchors,
            crl_verifier,
            time,
            certificate_usage,
        )
        .await
    }

    /// Verify the certificate path of `x5chain` and the COSE signature, then deserialize the authenticated payload.
    ///
    /// Configured trust anchors present among the intermediate certificates are ignored. The leaf certificate
    /// must not itself be a trust anchor.
    pub(crate) fn verify_with_x5chain(
        &self,
        x5chain: &VecNonEmpty<BorrowingCertificate>,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateExtendedKeyUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let (certificate, intermediate_certs) = x5chain.split_first();
        certificate
            .verify(certificate_usage, intermediate_certs, time, trust_anchors)
            .map_err(CoseError::CertificateVerify)?;
        self.verify_and_parse(certificate.public_key())
    }

    async fn verify_against_trust_anchors_with_chain_and_crl(
        &self,
        x5chain: VecNonEmpty<BorrowingCertificate>,
        trust_anchors: &TrustAnchors,
        crl_verifier: &CertificateCrlVerifier<impl CrlFetcher>,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateExtendedKeyUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        crl_verifier
            .verify_chain(x5chain.as_slice(), trust_anchors, certificate_usage, time)
            .await
            .map_err(|error| match error {
                CertificateCrlVerificationError::CertificateVerify(source) => CoseError::CertificateVerify(source),
                error => CoseError::CertificateCrl(error),
            })?;
        self.verify_and_parse(x5chain.first().public_key())
    }
}

fn signature_data_with_header(payload: &[u8], header: Header) -> (Vec<u8>, ProtectedHeader) {
    let protected_header = ProtectedHeader {
        original_data: None,
        header,
    };
    let signature_data = sig_structure_data(
        SignatureContext::CoseSign1,
        protected_header.clone(),
        None,
        &[],
        payload,
    );
    (signature_data, protected_header)
}

async fn sign_cose(
    payload: Vec<u8>,
    unprotected_header: Header,
    private_key: &impl EcdsaKey,
) -> Result<CoseSign1, CoseError> {
    sign_cose_with_headers(
        payload,
        HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build(),
        unprotected_header,
        private_key,
    )
    .await
}

async fn sign_cose_with_headers(
    payload: Vec<u8>,
    protected_header: Header,
    unprotected_header: Header,
    private_key: &impl EcdsaKey,
) -> Result<CoseSign1, CoseError> {
    let (signature_data, protected_header) = signature_data_with_header(&payload, protected_header);
    let signature = private_key
        .try_sign(&signature_data)
        .await
        .map_err(|error| CoseError::Signing(error.into()))?
        .to_vec();

    Ok(CoseSign1 {
        signature,
        payload: Some(payload),
        protected: protected_header,
        unprotected: unprotected_header,
    })
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use crypto::server_keys::generate::Ca;
    use serde::Deserialize;
    use utils::generator::TimeGenerator;

    use super::*;

    const TOY_TYP: &str = "application/toy+cwt";

    #[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
    struct ToyMessage {
        string: String,
    }

    impl CoseTyp for ToyMessage {
        const TYP: &'static str = TOY_TYP;
    }

    fn toy_message() -> ToyMessage {
        ToyMessage {
            string: "Hello, World!".to_owned(),
        }
    }

    #[tokio::test]
    async fn test_protected_and_unprotected_x5chain() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_issuer_status_list_mock().unwrap();

        let cose = TypedCose::sign_with_typ_and_certificate(&toy_message(), &key_pair)
            .await
            .unwrap();

        assert_eq!(cose.protected_x5chain().unwrap().first(), key_pair.certificate());
        assert_matches!(
            cose.unprotected_x5chain(),
            Err(CoseError::MissingLabel(Label::Int(COSE_X5CHAIN_HEADER_LABEL)))
        );
    }

    #[tokio::test]
    async fn test_verify_with_typ_against_trust_anchors_verifies_sign_with_typ_and_certificate() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_issuer_status_list_mock().unwrap();

        let cose = TypedCose::sign_with_typ_and_certificate(&toy_message(), &key_pair)
            .await
            .unwrap();
        let payload = cose
            .verify_with_typ_against_trust_anchors(&ca.to_trust_anchors(), &TimeGenerator, None)
            .unwrap();

        assert_eq!(payload, toy_message());
    }

    #[tokio::test]
    async fn test_verify_with_typ_against_trust_anchors_rejects_unexpected_typ() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_issuer_status_list_mock().unwrap();

        let mut protected_header = header_with_x5chain(&vec_nonempty![key_pair.certificate()]);
        protected_header.rest.push((
            Label::Int(COSE_TYP_HEADER_LABEL),
            Value::Text("application/other+cwt".to_owned()),
        ));
        protected_header.alg = Some(coset::Algorithm::Assigned(iana::Algorithm::ES256));

        let cose =
            TypedCose::sign_with_protected_header(&toy_message(), protected_header, Header::default(), &key_pair)
                .await
                .unwrap();

        let error = cose
            .verify_with_typ_against_trust_anchors(&ca.to_trust_anchors(), &TimeGenerator, None)
            .unwrap_err();

        assert_matches!(error, CoseError::UnexpectedTyp(TOY_TYP, Some(Value::Text(typ))) if typ == "application/other+cwt");
    }

    #[tokio::test]
    async fn test_verify_with_typ_against_trust_anchors_requires_protected_x5chain() {
        let ca = Ca::generate_mock();
        let key_pair = ca.generate_issuer_status_list_mock().unwrap();

        let mut protected_header = Header::default();
        protected_header
            .rest
            .push((Label::Int(COSE_TYP_HEADER_LABEL), Value::Text(TOY_TYP.to_owned())));
        protected_header.alg = Some(coset::Algorithm::Assigned(iana::Algorithm::ES256));
        let unprotected_header = header_with_x5chain(&vec_nonempty![key_pair.certificate()]);
        let cose =
            TypedCose::sign_with_protected_header(&toy_message(), protected_header, unprotected_header, &key_pair)
                .await
                .unwrap();

        let error = cose
            .verify_with_typ_against_trust_anchors(&ca.to_trust_anchors(), &TimeGenerator, None)
            .unwrap_err();

        assert_matches!(error, CoseError::MissingLabel(Label::Int(COSE_X5CHAIN_HEADER_LABEL)));
    }
}
