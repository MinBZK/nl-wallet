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
        let certs = value
            .iter()
            .map(|(doc_type, bts)| Ok((doc_type.clone(), Self::parse(bts)?)))
            .collect::<Result<IndexMap<_, _>>>()?
            .into();
        Ok(certs)
    }
}

impl<'a> TrustedIssuerCerts<'a> {
    pub fn new() -> Self {
        IndexMap::new().into()
    }

    pub fn parse(cert_bts: &'a [u8]) -> Result<X509Certificate<'a>> {
        let cert = X509Certificate::from_der(cert_bts)
            .map_err(HolderError::CertificateParsingFailed)?
            .1;
        Ok(cert)
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

#[async_trait]
pub trait UserConsentIssuance {
    async fn ask(&self, request: &RequestKeyGenerationMessage) -> bool;
}

impl Credentials {
    pub async fn do_issuance(
        &self,
        service_engagement: ServiceEngagement,
        user_consent: impl UserConsentIssuance,
        client_builder: impl HttpClientBuilder,
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

        if !user_consent.ask(&request).await {
            // Inform the server we want to abourt. We don't care if an error occurs here
            let _: Result<EndSessionMessage> = client
                .post(&RequestEndSessionMessage {
                    e_session_id: session_id.clone(),
                })
                .await;
            return Ok(());
        }

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

        let state = IssuanceState {
            request,
            private_keys,
            response,
        };
        Ok(state)
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
            .map(|((doc, unsigned), keys)| Self::create_cred_copies(doc, unsigned, keys, trusted_issuer_certs))
            .collect()
    }

    fn create_cred_copies(
        doc: &basic_sa_ext::MobileIDDocuments,
        unsigned: &UnsignedMdoc,
        keys: &Vec<SigningKey<NistP256>>,
        trusted_issuer_certs: &TrustedIssuerCerts,
    ) -> Result<CredentialCopies> {
        let cred_copies = doc
            .sparse_issuer_signed
            .iter()
            .zip(keys)
            .map(|(iss_signature, key)| {
                iss_signature.to_credential(key.clone(), unsigned, trusted_issuer_certs.get(&doc.doc_type)?)
            })
            .collect::<Result<Vec<_>>>()?
            .into();
        Ok(cred_copies)
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
            .map(|(namespace, entries)| (namespace.clone(), Self::create_attrs(namespace, entries, &self.randoms)))
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

        let cred = Credential {
            private_key,
            issuer_signed,
            doc_type: unsigned.doc_type.clone(),
        };
        Ok(cred)
    }

    fn create_attrs(namespace: &NameSpace, attrs: &[Entry], randoms: &IndexMap<NameSpace, Vec<ByteBuf>>) -> Attributes {
        attrs
            .iter()
            .enumerate()
            .map(|(index, attr)| attr.to_issuer_signed_item(index, randoms[namespace][index].to_vec()))
            .collect::<Vec<_>>()
            .into()
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
