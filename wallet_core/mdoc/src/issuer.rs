use chrono::Utc;
use ciborium::value::Value;
use coset::{CoseSign1, HeaderBuilder};

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, KeyGenerationResponseMessage, MdocResponses, MobileIDDocuments,
        RequestKeyGenerationMessage, Response, SparseIssuerAuth, SparseIssuerSigned, UnsignedMdoc,
    },
    cose::{ClonePayload, MdocCose},
    iso::*,
    serialization::TaggedBytes,
    Result,
};

pub struct Issuer {
    private_key: ecdsa::SigningKey<p256::NistP256>,
    cert_bts: Vec<u8>,
    request: RequestKeyGenerationMessage,
}

impl Issuer {
    pub fn new(
        request: RequestKeyGenerationMessage,
        private_key: ecdsa::SigningKey<p256::NistP256>,
        cert_bts: Vec<u8>,
    ) -> Issuer {
        Issuer {
            request,
            cert_bts,
            private_key,
        }
    }

    fn issue_cred(&self, unsigned_mdoc: &UnsignedMdoc, response: &Response) -> Result<SparseIssuerSigned> {
        let attrs = unsigned_mdoc
            .attributes
            .clone()
            .into_iter()
            .map(|(namespace, attrs)| Ok((namespace, Attributes::try_from(attrs)?)))
            .collect::<Result<IssuerNameSpaces>>()?;

        let now = Utc::now();
        let validity = ValidityInfo {
            signed: now.into(),
            valid_from: now.into(),
            valid_until: unsigned_mdoc.valid_until.clone(),
            expected_update: None,
        };
        let mso = MobileSecurityObject {
            version: "1.0".to_string(),
            digest_algorithm: "SHA-256".to_string(),
            doc_type: unsigned_mdoc.doc_type.clone(),
            value_digests: (&attrs).try_into()?,
            device_key_info: response.public_key.clone().into(),
            validity_info: validity.clone(),
        };

        let headers = HeaderBuilder::new()
            .value(33, Value::Bytes(self.cert_bts.clone()))
            .build();
        let cose: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(&mso.into(), headers, &self.private_key)?;

        Ok(SparseIssuerSigned {
            randoms: attrs
                .into_iter()
                .map(|(namespace, attrs)| (namespace, attrs.0.into_iter().map(|attr| attr.0.random).collect()))
                .collect(),
            sparse_issuer_auth: SparseIssuerAuth {
                version: "1.0".to_string(),
                digest_algorithm: "SHA-256".to_string(),
                validity_info: validity,
                issuer_auth: cose.clone_without_payload(),
            },
        })
    }

    fn issue_creds(
        &self,
        doctype_responses: &MdocResponses,
        unsigned: &UnsignedMdoc,
    ) -> Result<Vec<SparseIssuerSigned>> {
        doctype_responses
            .responses
            .iter()
            .map(|response| self.issue_cred(unsigned, response))
            .collect()
    }

    pub fn issue(self, device_response: &KeyGenerationResponseMessage) -> Result<DataToIssueMessage> {
        device_response.verify(&self.request)?;

        let docs = device_response
            .mdoc_responses
            .iter()
            .zip(&self.request.unsigned_mdocs)
            .map(|(responses, unsigned)| {
                Ok(MobileIDDocuments {
                    doc_type: unsigned.doc_type.clone(),
                    sparse_issuer_signed: self.issue_creds(responses, unsigned)?,
                })
            })
            .collect::<Result<Vec<MobileIDDocuments>>>()?;

        Ok(DataToIssueMessage {
            e_session_id: self.request.e_session_id,
            mobile_id_documents: docs,
        })
    }
}
