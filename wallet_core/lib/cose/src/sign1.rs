use std::num::NonZeroUsize;

use chrono::DateTime;
use chrono::Utc;
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
use crypto::x509::CertificateUsage;
use serde::Serialize;
use serde::de::DeserializeOwned;
use utils::generator::Generator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::error::CoseError;
use crate::message::TypedCose;
use crate::serialization::cbor_serialize;

/// COSE header label for `x5chain`, defined in RFC 9360.
pub const COSE_X5CHAIN_HEADER_LABEL: i64 = 33;

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
        Value::Bytes(bytes) => Ok(vec_nonempty![BorrowingCertificate::from_der(bytes.clone())?]),
        Value::Array(items) => {
            let certificates = items
                .iter()
                .map(|item| {
                    item.as_bytes()
                        .ok_or(CoseError::CertificateUnexpectedHeaderType)
                        .and_then(|bytes| Ok(BorrowingCertificate::from_der(bytes.clone())?))
                })
                .collect::<Result<Vec<_>, CoseError>>()?;
            certificates.try_into().map_err(|_| CoseError::EmptyCertificateChain)
        }
        _ => Err(CoseError::CertificateUnexpectedHeaderType),
    }
}

impl<T> TypedCose<CoseSign1, T> {
    pub async fn sign(
        payload: &T,
        unprotected_header: Header,
        private_key: &impl EcdsaKey,
        include_payload: bool,
    ) -> Result<Self, CoseError>
    where
        T: Serialize,
    {
        let payload = cbor_serialize(payload)?;
        Ok(sign_cose(&payload, unprotected_header, private_key, include_payload)
            .await?
            .into())
    }

    pub(crate) async fn sign_with_protected_header(
        payload: &T,
        protected_header: Header,
        unprotected_header: Header,
        private_key: &impl EcdsaKey,
        include_payload: bool,
    ) -> Result<Self, CoseError>
    where
        T: Serialize,
    {
        let payload = cbor_serialize(payload)?;
        Ok(sign_cose_with_headers(
            &payload,
            protected_header,
            unprotected_header,
            private_key,
            include_payload,
        )
        .await?
        .into())
    }

    /// Sign a COSE payload and include the key pair's certificate in the unprotected `x5chain` header.
    pub async fn sign_with_certificate<K: EcdsaKey>(
        payload: &T,
        key_pair: &KeyPair<K>,
        include_payload: bool,
    ) -> Result<Self, CoseError>
    where
        T: Serialize,
    {
        let header = header_with_x5chain(&vec_nonempty![key_pair.certificate()]);
        Self::sign(payload, header, key_pair, include_payload).await
    }

    /// Get the certificate chain from the unprotected `x5chain` header parameter.
    pub fn x5chain(&self) -> Result<VecNonEmpty<BorrowingCertificate>, CoseError> {
        x5chain_from_header(&self.as_inner().unprotected)
    }

    /// Verify the certificate path and COSE signature, then deserialize the authenticated payload.
    pub fn verify_against_trust_anchors(
        &self,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        self.verify_against_trust_anchors_with_chain(self.x5chain()?, trust_anchors, time, certificate_usage)
    }

    pub(crate) fn verify_against_trust_anchors_with_chain(
        &self,
        x5chain: VecNonEmpty<BorrowingCertificate>,
        trust_anchors: &TrustAnchors,
        time: &impl Generator<DateTime<Utc>>,
        certificate_usage: Option<CertificateUsage>,
    ) -> Result<T, CoseError>
    where
        T: DeserializeOwned,
    {
        let (certificate, chain) = x5chain.into_nonempty_iter().next();
        certificate.verify(
            certificate_usage,
            &chain.into_iter().collect::<Vec<_>>(),
            time,
            trust_anchors,
        )?;
        self.verify_and_parse(certificate.public_key())
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

pub async fn sign_cose(
    payload: &[u8],
    unprotected_header: Header,
    private_key: &impl EcdsaKey,
    include_payload: bool,
) -> Result<CoseSign1, CoseError> {
    sign_cose_with_headers(
        payload,
        HeaderBuilder::new().algorithm(iana::Algorithm::ES256).build(),
        unprotected_header,
        private_key,
        include_payload,
    )
    .await
}

async fn sign_cose_with_headers(
    payload: &[u8],
    protected_header: Header,
    unprotected_header: Header,
    private_key: &impl EcdsaKey,
    include_payload: bool,
) -> Result<CoseSign1, CoseError> {
    let (signature_data, protected_header) = signature_data_with_header(payload, protected_header);
    let signature = private_key
        .try_sign(&signature_data)
        .await
        .map_err(|error| CoseError::Signing(error.into()))?
        .to_vec();

    Ok(CoseSign1 {
        signature,
        payload: include_payload.then(|| payload.to_vec()),
        protected: protected_header,
        unprotected: unprotected_header,
    })
}
