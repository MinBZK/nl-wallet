use async_trait::async_trait;
use ecdsa::{elliptic_curve::rand_core::OsRng, SigningKey};
use indexmap::IndexMap;
use p256::NistP256;
use serde::{de::DeserializeOwned, Serialize};
use serde_bytes::ByteBuf;
use x509_parser::prelude::{FromDer, X509Certificate};

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, Entry, KeyGenerationResponseMessage, RequestKeyGenerationMessage, SparseIssuerSigned,
        StartIssuingMessage, UnsignedMdoc,
    },
    cose::ClonePayload,
    iso::*,
    serialization::{cbor_serialize, TaggedBytes},
    Error, Result,
};

use super::{Credential, CredentialCopies, Credentials, HolderError};

// TODO: support multiple certs per doctype, to allow key rollover.
// We might consider using https://docs.rs/owning_ref/latest/owning_ref/index.html to make the certificates owned.
/// Trusted CA certificates of issuers authorized to issue a doctype.
#[derive(Debug, Clone, Default)]
pub struct TrustedIssuerCerts<'a>(IndexMap<DocType, X509Certificate<'a>>);

impl<'a> From<IndexMap<DocType, X509Certificate<'a>>> for TrustedIssuerCerts<'a> {
    fn from(value: IndexMap<DocType, X509Certificate<'a>>) -> Self {
        Self(value)
    }
}

impl<'a, const N: usize> TryFrom<[(DocType, &'a [u8]); N]> for TrustedIssuerCerts<'a> {
    type Error = Error;

    fn try_from(value: [(DocType, &'a [u8]); N]) -> Result<Self> {
        Ok(value
            .iter()
            .map(|(doc_type, bts)| Ok((doc_type.clone(), Self::parse(bts)?)))
            .collect::<Result<IndexMap<_, _>>>()?
            .into())
    }
}

impl<'a> TrustedIssuerCerts<'a> {
    pub fn new() -> Self {
        IndexMap::new().into()
    }

    pub fn parse(cert_bts: &'a [u8]) -> Result<X509Certificate<'a>> {
        Ok(X509Certificate::from_der(cert_bts)
            .map_err(HolderError::CertificateParsingFailed)?
            .1)
    }

    pub fn get(&self, doc_type: &DocType) -> Result<&X509Certificate> {
        self.0
            .get(doc_type)
            .ok_or(Error::from(HolderError::UntrustedIssuer(doc_type.clone())))
    }
}

/// Used during a session to construct a HTTP client to interface with the server.
/// Can be used to pass information to the client that it needs during the session.
pub trait HttpClientBuilder {
    type Client: HttpClient;
    fn build(&self, service_engagement: ServiceEngagement) -> Self::Client;
}

#[async_trait]
pub trait HttpClient {
    async fn post<R, V>(&self, val: &V) -> Result<R>
    where
        V: Serialize + Sync,
        R: DeserializeOwned;
}

impl Credentials {
    pub async fn do_issuance<T: HttpClientBuilder>(
        &self,
        service_engagement: ServiceEngagement,
        client_builder: T,
        trusted_issuer_certs: &TrustedIssuerCerts<'_>,
    ) -> Result<()> {
        let client = client_builder.build(service_engagement);

        // Start issuance protocol
        let ready_msg: ReadyToProvisionMessage = client.post(&StartProvisioningMessage::default()).await?;
        let session_id = ready_msg.e_session_id;

        // Fetch the issuance details: challenge and the to-be-issued credentials
        let request: RequestKeyGenerationMessage = client
            .post(&StartIssuingMessage {
                e_session_id: session_id.clone(),
                version: 1, // TODO magic number
            })
            .await?;

        // Compute responses
        let state = IssuanceState::issuance_start(request)?;

        // Finish issuance protocol
        let issuer_response: DataToIssueMessage = client.post(&state.response).await?;

        // Process issuer response to obtain and save new credentials
        let creds = IssuanceState::issuance_finish(state, issuer_response, trusted_issuer_certs)?;
        self.add(creds.into_iter().flatten())
    }
}

#[derive(Debug)]
pub struct IssuanceState {
    pub request: RequestKeyGenerationMessage,
    pub response: KeyGenerationResponseMessage,

    /// Private keys grouped by distinct credentials, and then per copies of each distinct credential.
    pub private_keys: Vec<Vec<SigningKey<NistP256>>>,
}

impl IssuanceState {
    pub fn issuance_start(request: RequestKeyGenerationMessage) -> Result<IssuanceState> {
        let private_keys = request
            .unsigned_mdocs
            .iter()
            .map(|unsigned| Self::generate_keys(unsigned.count))
            .collect::<Vec<_>>();
        let response = KeyGenerationResponseMessage::new(&request, &private_keys)?;

        Ok(IssuanceState {
            request,
            private_keys,
            response,
        })
    }

    pub fn generate_keys(count: u64) -> Vec<SigningKey<p256::NistP256>> {
        (0..count)
            .map(|_| SigningKey::<p256::NistP256>::random(OsRng))
            .collect()
    }

    pub fn issuance_finish(
        state: IssuanceState,
        issuer_response: DataToIssueMessage,
        trusted_issuer_certs: &TrustedIssuerCerts,
    ) -> Result<Vec<CredentialCopies>> {
        issuer_response
            .mobile_id_documents
            .iter()
            .zip(&state.request.unsigned_mdocs)
            .zip(&state.private_keys)
            .map(|((doc, unsigned), keys)| {
                Ok(doc
                    .sparse_issuer_signed
                    .iter()
                    .zip(keys)
                    .map(|(iss_signature, key)| {
                        iss_signature.to_credential(key.clone(), unsigned, trusted_issuer_certs.get(&doc.doc_type)?)
                    })
                    .collect::<Result<Vec<_>>>()?
                    .into())
            })
            .collect()
    }
}

impl SparseIssuerSigned {
    pub(super) fn to_credential(
        &self,
        private_key: SigningKey<p256::NistP256>,
        unsigned: &UnsignedMdoc,
        iss_cert: &X509Certificate,
    ) -> Result<Credential> {
        let name_spaces: IssuerNameSpaces = unsigned
            .attributes
            .iter()
            .map(|(namespace, attrs)| {
                (
                    namespace.clone(),
                    attrs
                        .iter()
                        .enumerate()
                        .map(|(index, attr)| attr.to_issuer_signed_item(index, self.randoms[namespace][index].to_vec()))
                        .collect::<Vec<_>>()
                        .into(),
                )
            })
            .collect();

        let mso = MobileSecurityObject {
            version: self.sparse_issuer_auth.version.clone(),
            digest_algorithm: self.sparse_issuer_auth.digest_algorithm.clone(),
            value_digests: (&name_spaces).try_into()?,
            device_key_info: private_key.verifying_key().try_into()?,
            doc_type: unsigned.doc_type.clone(),
            validity_info: self.sparse_issuer_auth.validity_info.clone(),
        };
        let issuer_auth = self
            .sparse_issuer_auth
            .issuer_auth
            .clone_with_payload(cbor_serialize(&TaggedBytes::from(mso))?);

        let issuer_signed = IssuerSigned {
            name_spaces: Some(name_spaces),
            issuer_auth,
        };
        issuer_signed.verify(iss_cert)?;

        Ok(Credential {
            private_key,
            issuer_signed,
            doc_type: unsigned.doc_type.clone(),
        })
    }
}

impl Entry {
    fn to_issuer_signed_item(&self, index: usize, random: Vec<u8>) -> IssuerSignedItemBytes {
        IssuerSignedItem {
            digest_id: index as u64,
            random: ByteBuf::from(random),
            element_identifier: self.name.clone(),
            element_value: self.value.clone(),
        }
        .into()
    }
}
