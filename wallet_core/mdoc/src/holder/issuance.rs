use async_trait::async_trait;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};
use serde_bytes::ByteBuf;
pub use webpki::TrustAnchor;

use wallet_common::utils::random_string;
use x509_parser::nom::AsBytes;

use crate::{
    basic_sa_ext::{
        DataToIssueMessage, Entry, KeyGenerationResponseMessage, RequestKeyGenerationMessage, SparseIssuerSigned,
        StartIssuingMessage, UnsignedMdoc,
    },
    iso::*,
    issuer_shared::IssuanceError,
    utils::{
        cose::ClonePayload,
        keys::MdocEcdsaKey,
        serialization::{cbor_deserialize, cbor_serialize, TaggedBytes},
        TimeGenerator,
    },
    verifier::ValidityRequirement,
    Result,
};

use super::{HolderError, Mdoc, MdocCopies, Storage, Wallet};

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

/// Send and receive CBOR-encoded messages over HTTP for a session using a [`reqwest::Client`]
/// and a [`ServiceEngagement`].
pub struct CborHttpClient {
    pub service_engagement: ServiceEngagement,
    pub client: reqwest::Client,
}

#[async_trait]
impl HttpClient for CborHttpClient {
    async fn post<R, V>(&self, val: &V) -> Result<R>
    where
        V: Serialize + Sync,
        R: DeserializeOwned,
    {
        let bytes = cbor_serialize(val)?;
        let response_bytes = self
            .client
            .post(self.service_engagement.url.as_ref().unwrap())
            .body(bytes)
            .send()
            .await
            .map_err(HolderError::RequestError)?
            .bytes()
            .await
            .map_err(HolderError::RequestError)?;
        let response = cbor_deserialize(response_bytes.as_bytes())?;
        Ok(response)
    }
}

/// Ask the user for consent during an issuance session, presentimg them with the [`RequestKeyGenerationMessage`]
/// containing the to-be-received mdocs.
#[async_trait]
pub trait IssuanceUserConsent {
    async fn ask(&self, request: &RequestKeyGenerationMessage) -> bool;
}

impl<C: Storage> Wallet<C> {
    /// Do an ISO 23220-3 issuance session, using the SA-specific protocol from `basic_sa_ext.rs`.
    pub async fn do_issuance<K: MdocEcdsaKey>(
        &self,
        service_engagement: ServiceEngagement,
        user_consent: &impl IssuanceUserConsent,
        client_builder: &impl HttpClientBuilder,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<()> {
        let client = client_builder.build(service_engagement);

        // Start issuance protocol
        let ready_msg: ReadyToProvisionMessage = client.post(&StartProvisioningMessage::default()).await?;
        let session_id = ready_msg.e_session_id;

        // Fetch the issuance details: challenge and the to-be-issued mdocs
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
        let state = IssuanceState::<K>::issuance_start(request)?;

        // Finish issuance protocol
        let issuer_response: DataToIssueMessage = client.post(&state.response).await?;

        // Process issuer response to obtain and save new mdocs
        let creds = state.issuance_finish(issuer_response, trust_anchors)?;
        self.storage.add(creds.into_iter().flatten())
    }
}

#[derive(Debug)]
pub struct IssuanceState<K> {
    pub request: RequestKeyGenerationMessage,
    pub response: KeyGenerationResponseMessage,

    /// Private keys grouped by distinct mdocs, and then per copies of each distinct mdoc.
    pub private_keys: Vec<Vec<K>>,
}

impl<K: MdocEcdsaKey> IssuanceState<K> {
    pub fn issuance_start(request: RequestKeyGenerationMessage) -> Result<IssuanceState<K>> {
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

    pub fn generate_keys(count: u64) -> Vec<K> {
        (0..count).map(|_| K::new(&random_string(32))).collect()
    }

    pub fn issuance_finish(
        self,
        issuer_response: DataToIssueMessage,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Vec<MdocCopies<K>>> {
        issuer_response
            .mobile_id_documents
            .iter()
            .zip(&self.request.unsigned_mdocs)
            .zip(&self.private_keys)
            .map(|((doc, unsigned), keys)| Self::create_cred_copies(doc, unsigned, keys, trust_anchors))
            .collect()
    }

    fn create_cred_copies(
        doc: &basic_sa_ext::MobileeIDDocuments,
        unsigned: &UnsignedMdoc,
        keys: &Vec<K>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<MdocCopies<K>> {
        let cred_copies = doc
            .sparse_issuer_signed
            .iter()
            .zip(keys)
            .map(|(iss_signature, key)| iss_signature.to_mdoc(key, unsigned, trust_anchors))
            .collect::<Result<Vec<_>>>()?
            .into();
        Ok(cred_copies)
    }
}

impl SparseIssuerSigned {
    pub(super) fn to_mdoc<K: MdocEcdsaKey>(
        &self,
        private_key: &K,
        unsigned: &UnsignedMdoc,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Mdoc<K>> {
        let name_spaces: IssuerNameSpaces = unsigned
            .attributes
            .iter()
            .map(|(namespace, entries)| (namespace.clone(), Self::create_attrs(namespace, entries, &self.randoms)))
            .collect();

        let mso = MobileSecurityObject {
            version: self.sparse_issuer_auth.version.clone(),
            digest_algorithm: self.sparse_issuer_auth.digest_algorithm.clone(),
            value_digests: (&name_spaces).try_into()?,
            device_key_info: private_key
                .verifying_key()
                .map_err(|e| IssuanceError::PrivatePublicKeyConversion(e.into()))?
                .try_into()?,
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
        issuer_signed.verify(ValidityRequirement::AllowNotYetValid, &TimeGenerator, trust_anchors)?;

        let cred = Mdoc::_new(
            unsigned.doc_type.clone(),
            private_key.identifier().to_string(),
            issuer_signed,
        );
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
