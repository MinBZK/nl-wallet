use async_trait::async_trait;
use futures::future::try_join_all;
use indexmap::IndexMap;
use serde::{de::DeserializeOwned, Serialize};
use serde_bytes::ByteBuf;
use url::Url;
pub use webpki::TrustAnchor;
use x509_parser::nom::AsBytes;

use wallet_common::{generator::TimeGenerator, utils::random_string};

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
    },
    Result,
};

use super::{HolderError, Mdoc, MdocCopies, Storage, Wallet};

#[async_trait]
pub trait HttpClient {
    async fn post<R, V>(&self, url: &Url, val: &V) -> Result<R>
    where
        V: Serialize + Sync,
        R: DeserializeOwned;
}

/// Send and receive CBOR-encoded messages over HTTP using a [`reqwest::Client`].
pub struct CborHttpClient(pub reqwest::Client);

#[async_trait]
impl HttpClient for CborHttpClient {
    async fn post<R, V>(&self, url: &Url, val: &V) -> Result<R>
    where
        V: Serialize + Sync,
        R: DeserializeOwned,
    {
        let bytes = cbor_serialize(val)?;
        let response_bytes = self
            .0
            .post(url.clone())
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

#[derive(Debug)]
pub(crate) struct IssuanceSessionState<H> {
    client: H,
    url: Url,
    request: RequestKeyGenerationMessage,
}

impl<C: Storage, H: HttpClient> Wallet<C, H> {
    /// Do an ISO 23220-3 issuance session, using the SA-specific protocol from `basic_sa_ext.rs`.
    pub async fn start_issuance(
        &mut self,
        service_engagement: ServiceEngagement,
        client: H,
    ) -> Result<&[UnsignedMdoc]> {
        assert!(self.session_state.is_none());

        let url = service_engagement
            .url
            .as_ref()
            .ok_or(HolderError::MalformedServiceEngagement)?;

        // Start issuance protocol
        let start_prov_msg = StartProvisioningMessage {
            provisioning_code: service_engagement.pc.clone(),
        };
        let ready_msg: ReadyToProvisionMessage = client.post(url, &start_prov_msg).await?;
        let session_id = ready_msg.e_session_id;

        // Fetch the issuance details: challenge and the to-be-issued mdocs
        let start_issuing_msg = StartIssuingMessage {
            e_session_id: session_id,
            version: 1, // TODO magic number
        };
        let request: RequestKeyGenerationMessage = client.post(url, &start_issuing_msg).await?;

        self.session_state.replace(IssuanceSessionState {
            client,
            url: url.clone(),
            request,
        });

        Ok(&self.session_state.as_ref().unwrap().request.unsigned_mdocs)
    }

    pub async fn finish_issuance<K: MdocEcdsaKey>(&mut self, trust_anchors: &[TrustAnchor<'_>]) -> Result<()> {
        let state = self
            .session_state
            .as_ref()
            .ok_or(HolderError::MissingIssuanceSessionState)?;

        // Compute responses
        let (keys, responses) = state.keys_and_responses::<K>().await?;

        // Finish issuance protocol
        let issuer_response: DataToIssueMessage = state.client.post(&state.url, &responses).await?;

        // Process issuer response to obtain and save new mdocs
        let creds = state.construct_mdocs(keys, issuer_response, trust_anchors).await?;
        self.storage.add(creds.into_iter().flatten())?;

        // Clear session state now that all fallible operations have not failed
        self.session_state.take();

        Ok(())
    }

    pub async fn stop_issuance(&mut self) -> Result<()> {
        let IssuanceSessionState { request, client, url } = self
            .session_state
            .take()
            .ok_or(HolderError::MissingIssuanceSessionState)?;

        // Inform the server we want to abort. We don't care if an error occurs here
        let end_msg = RequestEndSessionMessage {
            e_session_id: request.e_session_id,
        };
        let _: Result<EndSessionMessage> = client.post(&url, &end_msg).await;

        Ok(())
    }
}

impl<H> IssuanceSessionState<H> {
    pub async fn keys_and_responses<K: MdocEcdsaKey>(&self) -> Result<(Vec<Vec<K>>, KeyGenerationResponseMessage)> {
        // Group the keys by distinct mdocs, and then per copies of each distinct mdoc
        let private_keys: Vec<Vec<K>> = self
            .request
            .unsigned_mdocs
            .iter()
            .map(|unsigned| Self::generate_keys(unsigned.copy_count))
            .collect::<Vec<_>>();
        let private_keys_refs = private_keys.iter().map(|f| f.as_slice()).collect::<Vec<_>>();
        let response = KeyGenerationResponseMessage::new(&self.request, private_keys_refs.as_slice()).await?;

        Ok((private_keys, response))
    }

    pub fn generate_keys<K: MdocEcdsaKey>(count: u64) -> Vec<K> {
        (0..count).map(|_| K::new(&random_string(32))).collect()
    }

    pub async fn construct_mdocs<K: MdocEcdsaKey>(
        &self,
        private_keys: Vec<Vec<K>>,
        issuer_response: DataToIssueMessage,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Vec<MdocCopies<K>>> {
        let mdoc_copies = try_join_all(
            issuer_response
                .mobile_eid_documents
                .iter()
                .zip(&self.request.unsigned_mdocs)
                .zip(&private_keys)
                .map(|((doc, unsigned), keys)| Self::create_cred_copies(doc, unsigned, keys, trust_anchors)),
        )
        .await?;

        Ok(mdoc_copies)
    }

    async fn create_cred_copies<K: MdocEcdsaKey>(
        doc: &basic_sa_ext::MobileeIDDocuments,
        unsigned: &UnsignedMdoc,
        keys: &[K],
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<MdocCopies<K>> {
        let cred_copies = try_join_all(
            doc.sparse_issuer_signed
                .iter()
                .zip(keys)
                .map(|(iss_signature, key)| iss_signature.to_mdoc(key, unsigned, trust_anchors)),
        )
        .await?;

        Ok(cred_copies.into())
    }
}

impl SparseIssuerSigned {
    pub(super) async fn to_mdoc<K: MdocEcdsaKey>(
        &self,
        private_key: &K,
        unsigned: &UnsignedMdoc,
        trust_anchors: &[TrustAnchor<'_>],
    ) -> Result<Mdoc<K>> {
        let name_spaces: IssuerNameSpaces = unsigned
            .attributes
            .iter()
            .map(|(namespace, entries)| {
                let attrs = (
                    namespace.clone(),
                    Self::create_attrs(namespace, entries, &self.randoms)?,
                );
                Ok(attrs)
            })
            .collect::<Result<_>>()?;

        let mso = MobileSecurityObject {
            version: self.sparse_issuer_auth.version.clone(),
            digest_algorithm: self.sparse_issuer_auth.digest_algorithm.clone(),
            value_digests: (&name_spaces).try_into()?,
            device_key_info: private_key
                .verifying_key()
                .await
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

        // Construct the mdoc, also verifying it (using `IssuerSigned::verify()`).
        let cred = Mdoc::new(
            private_key.identifier().to_string(),
            issuer_signed,
            &TimeGenerator,
            trust_anchors,
        )?;
        Ok(cred)
    }

    fn create_attrs(
        namespace: &NameSpace,
        entries: &[Entry],
        randoms: &IndexMap<NameSpace, Vec<ByteBuf>>,
    ) -> Result<Attributes> {
        entries
            .iter()
            .enumerate()
            .map(|(index, entry)| entry.to_issuer_signed_item(index, randoms[namespace][index].to_vec()))
            .collect::<Result<Vec<_>>>()
            .map(Attributes::from)
    }
}

impl Entry {
    fn to_issuer_signed_item(&self, index: usize, random: Vec<u8>) -> Result<IssuerSignedItemBytes> {
        if random.len() < ATTR_RANDOM_LENGTH {
            return Err(HolderError::AttributeRandomLength(random.len(), ATTR_RANDOM_LENGTH).into());
        }
        let item = IssuerSignedItem {
            digest_id: index as u64,
            random: ByteBuf::from(random),
            element_identifier: self.name.clone(),
            element_value: self.value.clone(),
        }
        .into();
        Ok(item)
    }
}
