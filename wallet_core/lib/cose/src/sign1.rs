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
use crypto::PublicKey;
use crypto::keys::EcdsaKey;
use crypto::server_keys::KeyPair;
use crypto::trust_anchor::TrustAnchors;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateUsage;
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
    pub fn x5chain(&self) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError> {
        x5chain_from_header(&self.as_ref().unprotected)
    }

    /// Get the certificate chain from the unprotected `x5chain` header parameter, rejecting it if it contains the trust
    /// anchor, as mandated by HAIP 1.0.
    fn x5chain_without_trust_anchor(
        &self,
        trust_anchors: &TrustAnchors,
    ) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError> {
        let x5chain = self.x5chain()?;

        BorrowingCertificate::ensure_trust_anchor_not_in_chain(&x5chain, trust_anchors)
            .map_err(CoseError::CertificateVerify)?;

        Ok(x5chain)
    }

    /// Verify the certificate path and COSE signature, then deserialize the authenticated payload.
    ///
    /// A `None` certificate usage still validates the certificate path, but does not require a profile-specific
    /// extended key usage.
    ///
    /// As mandated by HAIP 1.0, the `x5chain` header must not contain the trust anchor.
    pub fn verify_against_trust_anchors(
        &self,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let x5chain = self.x5chain_without_trust_anchor(trust_anchors)?;

        self.verify_with_x5chain(&x5chain, trust_anchors, time, certificate_usage)
    }

    /// Verify the certificate path with fail-closed CRL checking and the COSE signature, then deserialize the
    /// authenticated payload.
    ///
    /// As mandated by HAIP 1.0, the `x5chain` header must not contain the trust anchor.
    pub async fn verify_against_trust_anchors_with_crl(
        &self,
        trust_anchors: &TrustAnchors,
        crl_verifier: &CertificateCrlVerifier<impl CrlFetcher>,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let x5chain = self.x5chain_without_trust_anchor(trust_anchors)?;

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
    /// This does not apply any profile-specific rules to the contents of `x5chain`, such as checking that it does not
    /// contain the trust anchor.
    pub(crate) fn verify_with_x5chain(
        &self,
        x5chain: &VecNonEmpty<BorrowingCertificate>,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let (certificate, intermediate_certs) = x5chain.split_first();
        certificate
            .verify(certificate_usage, intermediate_certs, time, trust_anchors)
            .map_err(CoseError::CertificateVerify)?;

        let PublicKey::ESP256(verifying_key) = certificate.public_key() else {
            return Err(CoseError::UnsupportedCertificateAlgorithm);
        };

        self.verify_and_parse(verifying_key)
    }

    async fn verify_against_trust_anchors_with_chain_and_crl(
        &self,
        x5chain: VecNonEmpty<BorrowingCertificate>,
        trust_anchors: &TrustAnchors,
        crl_verifier: &CertificateCrlVerifier<impl CrlFetcher>,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
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

        let PublicKey::ESP256(verifying_key) = x5chain.first().public_key() else {
            return Err(CoseError::UnsupportedCertificateAlgorithm);
        };

        self.verify_and_parse(verifying_key)
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
