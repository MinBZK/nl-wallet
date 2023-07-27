use crate::cose::{ClonePayload, CoseKey};
use crate::iso::*;
use crate::serialization::cbor_serialize;
use crate::{cose::MdocCose, crypto::random_bytes, serialization::TaggedBytes};

use anyhow::{anyhow, Result};
use chrono::Utc;
use ciborium::value::Value;
use coset::{AsCborValue, CoseSign1, CoseSign1Builder, HeaderBuilder, Label};
use ecdsa::signature::Signer;
use serde::{Deserialize, Serialize};
use std::ops::Add;

pub struct Issuer {
    private_key: ecdsa::SigningKey<p256::NistP256>,
    cert_bts: Vec<u8>,
    doc_type: DocType,
    attributes: IssuerNameSpaces,
    pub challenge: Vec<u8>,
}

impl Issuer {
    pub fn new(
        private_key: ecdsa::SigningKey<p256::NistP256>,
        cert_bts: Vec<u8>,
        doc_type: DocType,
        attributes: IssuerNameSpaces,
    ) -> Result<Issuer> {
        return Ok(Issuer {
            cert_bts,
            private_key,
            doc_type,
            attributes,
            challenge: random_bytes(32)?,
        });
    }
}

impl Issuer {
    pub fn issue(self, device_response: &IssuanceDeviceResponse) -> Result<IssuerSigned> {
        let public_key = device_response.verify(self.challenge.as_slice())?;

        let now = Utc::now();
        let mso = MobileSecurityObject {
            version: "1.0".to_string(),
            digest_algorithm: "SHA-256".to_string(),
            doc_type: self.doc_type,
            value_digests: (&self.attributes).try_into()?,
            device_key_info: DeviceKeyInfo {
                device_key: public_key,
                key_authorizations: None,
                key_info: None,
            },
            validity_info: ValidityInfo {
                signed: now.into(),
                valid_from: now.into(),
                valid_until: now.add(chrono::Duration::days(365)).into(),
                expected_update: None,
            },
        };

        let headers = HeaderBuilder::new()
            .value(33, Value::Bytes(self.cert_bts))
            .build();
        let cose: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>> =
            MdocCose::sign(mso.into(), headers, &self.private_key)?;

        Ok(IssuerSigned {
            name_spaces: Some(self.attributes),
            issuer_auth: cose,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IssuanceDeviceResponse {
    cose: MdocCose<CoseSign1, IssuanceDeviceResponseContents>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct IssuanceDeviceResponseContents {
    challenge: Vec<u8>,
}

impl IssuanceDeviceResponse {
    pub(crate) fn sign(
        challenge: &[u8],
        device_key: &ecdsa::SigningKey<p256::NistP256>,
    ) -> Result<Self> {
        let public_key_cbor = CoseKey::try_from(&ecdsa::VerifyingKey::from(device_key))?
            .0
            .to_cbor_value()
            .map_err(|e| anyhow!("{e}"))?;
        let cose = CoseSign1Builder::new()
            .unprotected(
                HeaderBuilder::new()
                    .text_value("public_key".to_string(), public_key_cbor)
                    .build(),
            )
            .payload(cbor_serialize(&IssuanceDeviceResponseContents {
                challenge: challenge.into(),
            })?)
            .create_signature(&[], |data| device_key.sign(data).to_vec())
            .build()
            .clone_without_payload();

        Ok(IssuanceDeviceResponse { cose: cose.into() })
    }

    pub(crate) fn verify(&self, challenge: &[u8]) -> Result<CoseKey> {
        let public_key = CoseKey::from_cbor_value(
            self.cose
                .unprotected_header_item(&Label::Text("public_key".to_string()))?
                .clone(),
        )
        .map_err(|e| anyhow!(e))?;

        self.cose
            .clone_with_payload(cbor_serialize(&IssuanceDeviceResponseContents {
                challenge: challenge.into(),
            })?)
            .verify(&(&public_key).try_into()?)?;

        Ok(public_key)
    }
}
