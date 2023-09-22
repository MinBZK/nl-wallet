use chrono::{DateTime, Utc};
use indexmap::IndexMap;
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use webpki::TrustAnchor;

use wallet_common::{generator::Generator, utils::sha256};

use crate::{
    basic_sa_ext::Entry,
    iso::*,
    utils::{
        keys::{MdocEcdsaKey, MdocKeyType},
        serialization::cbor_serialize,
    },
    verifier::ValidityRequirement,
    Result,
};

use super::{CborHttpClient, HttpClient, IssuanceSessionState};

pub trait Storage {
    fn add(&self, creds: impl Iterator<Item = Mdoc>) -> Result<()>;
    fn list(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>>;

    // TODO returning all copies of all mdocs is very crude and should be refined.
    fn get(&self, doctype: &DocType) -> Option<Vec<MdocCopies>>;
}

pub struct Wallet<C, H = CborHttpClient> {
    pub(crate) storage: C,
    pub(crate) session_state: Option<IssuanceSessionState>,
    pub(crate) client: H,
}

impl<C: Storage, H: HttpClient> Wallet<C, H> {
    pub fn new(storage: C, client: H) -> Self {
        Self {
            storage,
            session_state: None,
            client,
        }
    }

    pub fn list_mdocs(&self) -> IndexMap<DocType, Vec<IndexMap<NameSpace, Vec<Entry>>>> {
        self.storage.list()
    }
}

/// Stores multiple copies of mdocs that have identical attributes.
///
/// TODO: support marking an mdoc has having been used, so that it can be avoided in future disclosures,
/// for unlinkability.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MdocCopies {
    pub cred_copies: Vec<Mdoc>,
}

impl IntoIterator for MdocCopies {
    type Item = Mdoc;
    type IntoIter = std::vec::IntoIter<Mdoc>;
    fn into_iter(self) -> Self::IntoIter {
        self.cred_copies.into_iter()
    }
}
impl From<Vec<Mdoc>> for MdocCopies {
    fn from(creds: Vec<Mdoc>) -> Self {
        Self { cred_copies: creds }
    }
}
impl MdocCopies {
    pub fn new() -> Self {
        MdocCopies { cred_copies: vec![] }
    }
}

/// A full mdoc: everything needed to disclose attributes from the mdoc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Mdoc {
    /// Doctype of the mdoc. This is also present inside the `issuer_signed`; we include it here for
    /// convenience (fetching it from the `issuer_signed` would involve parsing the COSE inside it).
    pub doc_type: String,

    /// Identifier of the mdoc's private key. Obtain a reference to it with [`Keyfactory::generate(private_key_id)`].
    // Note that even though these fields are not `pub`, to users of this package their data is still accessible
    // by serializing the mdoc and examining the serialized bytes. This is not a problem because it is essentially
    // unavoidable: when stored (i.e. serialized), we need to include all of this data to be able to recover a usable
    // mdoc after deserialization.
    pub(crate) private_key_id: String,
    pub(crate) issuer_signed: IssuerSigned,
    pub(crate) key_type: MdocKeyType,
}

impl Mdoc {
    /// Construct a new `Mdoc`, verifying it against the specified thrust anchors before returning it.
    pub fn new<K: MdocEcdsaKey>(
        private_key: String,
        issuer_signed: IssuerSigned,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<Mdoc> {
        let (_, mso) = issuer_signed.verify(ValidityRequirement::AllowNotYetValid, time, trust_anchors)?;
        let mdoc = Mdoc {
            doc_type: mso.doc_type,
            private_key_id: private_key,
            issuer_signed,
            key_type: K::KEY_TYPE,
        };
        Ok(mdoc)
    }

    /// Get a list of attributes ([`Entry`] instances) contained in the mdoc, mapped per [`NameSpace`].
    pub fn attributes(&self) -> IndexMap<NameSpace, Vec<Entry>> {
        self.issuer_signed
            .name_spaces
            .as_ref()
            .unwrap_or(&IndexMap::new())
            .iter()
            .map(|(namespace, attrs)| (namespace.clone(), Vec::<Entry>::from(attrs)))
            .collect::<IndexMap<_, _>>()
    }

    pub fn public_key(&self) -> Result<VerifyingKey> {
        self.issuer_signed
            .issuer_auth
            .dangerous_parse_unverified()?
            .0
            .device_key_info
            .try_into()
    }

    /// Hash of the mdoc, acting as an identifier for the mdoc. Takes into account its doctype
    /// and all of its attributes, using [`Self::attributes()`].
    /// Computed schematically as `SHA256(CBOR(doctype, attributes))`.
    ///
    /// Credentials having the exact same attributes
    /// with the exact same values have the same hash, regardless of the randoms of the attributes; the issuer
    /// signature; or the validity of the mdoc.
    pub fn hash(&self) -> Result<Vec<u8>> {
        let digest = sha256(&cbor_serialize(&(&self.doc_type, &self.attributes()))?);
        Ok(digest)
    }
}
