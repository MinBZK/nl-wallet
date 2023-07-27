//! This module contains data structures defined in the ISO 18013-5 spec, divided into four submodules.
//!
//! Some conventions used here:
//! - Type aliases are used where possible, in particular when no methods need to be attached to the data structure.
//! - For some structs, the spec demands that they should be serialized as a sequence (i.e., without the field names)
//!   or as a map with integer values (also without the field names). In this case, we define an associated struct
//!   whose name ends on `Keyed` which does use field names. This allows us to refer to the contents of such data
//!   structures by name instead of by numbers. We transform them into the form required by the spec using the
//!   [`CborSeq`](crate::serialization::CborSeq) and [`CborIntMap`](crate::serialization::CborIntMap) wrappers.
//! - Some CBOR data structures contain other data structures not directly, but instead their CBOR-serialized bytes.
//!   For this the [`TaggedBytes`](crate::TaggedBytes) wrapper is used.

/// Data structures containing mdoc credentials.
///
/// The main citizen of this module is [`MobileSecurityObject`], which is the object that is signed by the issuer.
/// This data structure does not directly contain the attributes ([`IssuerSignedItem`]) but instead only their digests,
/// to enable selective disclosure.
pub mod credentials {
    use crate::{
        cose::CoseKey,
        crypto::{cbor_digest, random_bytes},
        serialization::TaggedBytes,
    };

    use anyhow::Result;
    use chrono::Utc;
    use ciborium::{tag, value::Value};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use serde_bytes::ByteBuf;
    use std::fmt::Debug;

    pub type NameSpace = String;

    pub type Digest = ByteBuf;
    pub type DigestID = u32;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DigestIDs(pub IndexMap<DigestID, Digest>);
    impl TryFrom<&Attributes> for DigestIDs {
        type Error = anyhow::Error;
        fn try_from(val: &Attributes) -> Result<Self> {
            Ok(DigestIDs(
                val.0
                    .iter()
                    .enumerate()
                    .map(|(i, attr)| Ok((i as u32, ByteBuf::from(cbor_digest(attr)?))))
                    .collect::<Result<IndexMap<_, _>, anyhow::Error>>()?,
            ))
        }
    }

    /// Digests of the attributes, grouped per [`NameSpace`].
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ValueDigests(pub IndexMap<NameSpace, DigestIDs>);
    impl TryFrom<&IssuerNameSpaces> for ValueDigests {
        type Error = anyhow::Error;
        fn try_from(val: &IssuerNameSpaces) -> Result<Self> {
            Ok(ValueDigests(
                val.iter()
                    .map(|(namespace, attrs)| Ok((namespace.clone(), DigestIDs::try_from(attrs)?)))
                    .collect::<Result<IndexMap<_, _>, anyhow::Error>>()?,
            ))
        }
    }

    pub type KeyInfo = IndexMap<i32, Value>;

    pub type AuthorizedNameSpaces = Vec<NameSpace>;
    pub type AuthorizedDataElements = IndexMap<NameSpace, DataElementsArray>;
    pub type DataElementsArray = Vec<DataElementIdentifier>;
    pub type DataElementIdentifier = String;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct KeyAuthorizations {
        #[serde(rename = "nameSpaces")]
        pub(crate) name_spaces: Option<AuthorizedNameSpaces>,
        #[serde(rename = "dataElements")]
        pub(crate) data_elements: Option<AuthorizedDataElements>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DeviceKeyInfo {
        #[serde(rename = "deviceKey")]
        pub(crate) device_key: DeviceKey,
        #[serde(rename = "keyAuthorizations")]
        pub(crate) key_authorizations: Option<KeyAuthorizations>,
        #[serde(rename = "KeyInfo")]
        pub(crate) key_info: Option<KeyInfo>,
    }

    pub type DeviceKey = CoseKey;

    /// Data signed by the issuer, containing among others the digests of the attributes ([`ValueDigests`]).
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct MobileSecurityObject {
        pub(crate) version: String,
        #[serde(rename = "digestAlgorithm")]
        pub(crate) digest_algorithm: String,
        #[serde(rename = "valueDigests")]
        pub(crate) value_digests: ValueDigests,
        #[serde(rename = "deviceKeyInfo")]
        pub(crate) device_key_info: DeviceKeyInfo,
        #[serde(rename = "docType")]
        pub(crate) doc_type: String,
        #[serde(rename = "validityInfo")]
        pub(crate) validity_info: ValidityInfo,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ValidityInfo {
        pub(crate) signed: Tdate,
        #[serde(rename = "validFrom")]
        pub(crate) valid_from: Tdate,
        #[serde(rename = "validUntil")]
        pub(crate) valid_until: Tdate,
        #[serde(rename = "expectedUpdate")]
        pub(crate) expected_update: Option<Tdate>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Tdate(pub tag::Required<String, 0>);
    impl From<chrono::DateTime<Utc>> for Tdate {
        fn from(t: chrono::DateTime<Utc>) -> Self {
            Tdate(tag::Required(
                t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ))
        }
    }

    pub type DocType = String;

    /// [`Attributes`], which contains [`IssuerSignedItem`]s, grouped per [`NameSpace`].
    pub type IssuerNameSpaces = IndexMap<NameSpace, Attributes>;

    /// A `Vec` of [`IssuerSignedItemBytes`], i.e., attributes. In the [`IssuerNameSpaces`] map,
    /// this is used as the type of the keys. (This datastructure is itself not named in the spec.)
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Attributes(pub(crate) Vec<IssuerSignedItemBytes>);
    impl TryFrom<IndexMap<String, Value>> for Attributes {
        type Error = anyhow::Error;
        fn try_from(val: IndexMap<String, Value>) -> Result<Self> {
            Ok(Attributes(
                val.into_iter()
                    .enumerate()
                    .map(|(i, (key, val))| Ok(IssuerSignedItem::new(i as u32, key, val)?.into()))
                    .collect::<Result<Vec<_>, anyhow::Error>>()?,
            ))
        }
    }

    pub type IssuerSignedItemBytes = TaggedBytes<IssuerSignedItem>;

    /// An attribute.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct IssuerSignedItem {
        #[serde(rename = "digestID")]
        pub(crate) digest_id: u32,
        #[serde(with = "serde_bytes")]
        pub(crate) random: Vec<u8>,
        #[serde(rename = "elementIdentifier")]
        pub(crate) element_identifier: String,
        #[serde(rename = "elementValue")]
        pub(crate) element_value: Value,
    }

    impl IssuerSignedItem {
        pub fn new(
            digest_id: u32,
            element_identifier: String,
            element_value: Value,
        ) -> Result<IssuerSignedItem> {
            let random = random_bytes(32)?;
            Ok(IssuerSignedItem {
                digest_id,
                random,
                element_identifier,
                element_value,
            })
        }
    }
}

/// Data structures used in disclosure, created by the holder and sent to the verifier.
///
/// The main citizens of this module are [`DeviceResponse`], which is what the holder sends to the verifier during
/// verification, and [`IssuerSigned`], which contains the entire issuer-signed credential and the disclosed attributes.
pub mod disclosure {
    use crate::{
        cose::MdocCose,
        iso::credentials::*,
        serialization::{NullCborValue, RequiredValue, TaggedBytes},
    };

    use ciborium::value::Value;
    use coset::{CoseMac0, CoseSign1};
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DeviceResponse {
        pub(crate) version: String,
        pub(crate) documents: Option<Vec<Document>>,
        #[serde(rename = "documentErrors")]
        pub(crate) document_errors: Option<Vec<DocumentError>>,
        pub(crate) status: u32,
    }

    pub type DocumentError = IndexMap<DocType, ErrorCode>;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Document {
        #[serde(rename = "docType")]
        pub(crate) doc_type: DocType,
        #[serde(rename = "issuerSigned")]
        pub(crate) issuer_signed: IssuerSigned,
        #[serde(rename = "deviceSigned")]
        pub(crate) device_signed: DeviceSigned,
        pub(crate) errors: Option<Errors>,
    }

    /// The issuer-signed MSO in Cose format, as well as some or all of the attributes
    /// (i.e. [`IssuerSignedItem`]s) contained in the credential.
    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct IssuerSigned {
        #[serde(rename = "nameSpaces")]
        pub(crate) name_spaces: Option<IssuerNameSpaces>,
        #[serde(rename = "issuerAuth")]
        pub(crate) issuer_auth: MdocCose<CoseSign1, TaggedBytes<MobileSecurityObject>>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DeviceSigned {
        #[serde(rename = "nameSpaces")]
        pub(crate) name_spaces: DeviceNameSpacesBytes,
        #[serde(rename = "deviceAuth")]
        pub(crate) device_auth: DeviceAuth,
    }

    pub type DeviceNameSpacesBytes = TaggedBytes<DeviceNameSpaces>;
    pub type DeviceNameSpaces = IndexMap<NameSpace, DeviceSignedItems>;
    pub type DeviceSignedItems = IndexMap<DataElementIdentifier, Value>;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum DeviceAuth {
        #[serde(rename = "deviceSignature")]
        DeviceSignature(MdocCose<CoseSign1, RequiredValue<NullCborValue>>),
        #[serde(rename = "deviceMac")]
        DeviceMac(MdocCose<CoseMac0, RequiredValue<NullCborValue>>),
    }

    pub type Errors = IndexMap<NameSpace, ErrorItems>;
    pub type ErrorItems = IndexMap<DataElementIdentifier, ErrorCode>;
    pub type ErrorCode = i32;
}

/// Data structures used in disclosure for everything that has to be signed with the credential's private key.
/// Mainly [`DeviceAuthentication`] and all data structures inside it, which includes a transcript
/// of the session so far.
///
/// NB. "Device authentication" is not to be confused with the [`DeviceAuth`] data structure in the
/// ['disclosure`] module (which contains the holder's signature over [`DeviceAuthentication`] defined here).
pub mod device_authentication {
    use crate::{
        cose::CoseKey,
        iso::{credentials::*, disclosure::*},
        serialization::{
            CborIntMap, CborSeq, DeviceAuthenticationString, RequiredValue, TaggedBytes,
        },
    };

    use ciborium::value::Value;
    use fieldnames_derive::FieldNames;
    use serde::{Deserialize, Serialize};
    use serde_bytes::ByteBuf;
    use std::fmt::Debug;

    pub type DeviceAuthentication = CborSeq<DeviceAuthenticationKeyed>;

    pub type DeviceAuthenticationBytes = TaggedBytes<DeviceAuthentication>;

    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct DeviceAuthenticationKeyed {
        pub(crate) device_authentication: RequiredValue<DeviceAuthenticationString>,
        pub(crate) session_transcript: SessionTranscript,
        pub(crate) doc_type: DocType,
        pub(crate) device_name_spaces_bytes: DeviceNameSpacesBytes,
    }

    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct SessionTranscriptKeyed {
        pub(crate) device_engagement_bytes: DeviceEngagementBytes,
        pub(crate) ereader_key_bytes: EReaderKeyBytes,
        pub(crate) handover: Handover,
    }

    pub type SessionTranscript = CborSeq<SessionTranscriptKeyed>;

    pub type DeviceEngagementBytes = TaggedBytes<DeviceEngagement>;

    #[derive(Debug, Clone)]
    pub enum Handover {
        QRHandover,
        NFCHandover(NFCHandover),
    }

    #[derive(Debug, Clone)]
    pub struct NFCHandover {
        pub(crate) handover_select_message: ByteBuf,
        pub(crate) handover_request_message: Option<ByteBuf>,
    }

    pub type DeviceEngagement = CborIntMap<DeviceEngagementKeyed>;

    // TODO: support remaining fields
    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct DeviceEngagementKeyed {
        pub(crate) version: String,
        pub(crate) security: Security,
        pub(crate) device_retrieval_methods: Option<DeviceRetrievalMethods>,
        pub(crate) server_retrieval_methods: Option<ServerRetrievalMethods>,
        pub(crate) protocol_info: Option<ProtocolInfo>,
    }

    pub type Security = CborSeq<SecurityKeyed>;

    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct SecurityKeyed {
        pub(crate) cipher_suite_identifier: u32,
        pub(crate) e_device_key_bytes: EDeviceKeyBytes,
    }

    pub type DeviceRetrievalMethods = Vec<DeviceRetrievalMethod>;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ServerRetrievalMethods {
        #[serde(rename = "webApi")]
        pub(crate) web_api: WebApi,
        #[serde(rename = "oidc")]
        pub(crate) oidc: Oidc,
    }

    pub type Oidc = CborSeq<WebSessionInfo>;

    pub type WebApi = CborSeq<WebSessionInfo>;

    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct WebSessionInfo {
        pub(crate) version: u32,
        pub(crate) issuer_url: String,
        pub(crate) server_retrieval_token: String,
    }

    pub type ProtocolInfo = Value;

    pub type DeviceRetrievalMethod = CborSeq<DeviceRetrievalMethodKeyed>;

    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct DeviceRetrievalMethodKeyed {
        pub(crate) typ: u32,
        pub(crate) version: u32,
        pub(crate) retrieval_options: RetrievalOptions,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum RetrievalOptions {
        WifiOptions, // TODO
        BleOptions,
        NfcOptions,
    }

    pub type EReaderKeyBytes = TaggedBytes<EReaderKey>;
    pub type EReaderKey = CoseKey;
    pub type EDeviceKeyBytes = TaggedBytes<EDeviceKey>;
    pub type EDeviceKey = CoseKey;
}

/// Data structures with which a verifier requests attributes from a holder.
pub mod device_retrieval {
    use crate::{
        cose::MdocCose,
        iso::{credentials::*, device_authentication::*},
        serialization::{CborSeq, ReaderAuthenticationString, RequiredValue, TaggedBytes},
    };
    use fieldnames_derive::FieldNames;

    use ciborium::value::Value;
    use coset::CoseSign1;
    use indexmap::IndexMap;
    use serde::{Deserialize, Serialize};
    use std::fmt::Debug;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DeviceRequest {
        pub(crate) version: String,
        #[serde(rename = "docRequests")]
        pub(crate) doc_requests: Vec<DocRequest>,
    }
    impl DeviceRequest {
        pub fn new(items_requests: Vec<ItemsRequest>) -> DeviceRequest {
            DeviceRequest {
                version: "1.0".to_string(),
                doc_requests: items_requests
                    .into_iter()
                    .map(|items_request| DocRequest {
                        items_request: items_request.into(),
                        reader_auth: None,
                    })
                    .collect(),
            }
        }
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct DocRequest {
        #[serde(rename = "itemsRequest")]
        pub(crate) items_request: ItemsRequestBytes,
        #[serde(rename = "readerAuth")]
        pub(crate) reader_auth: Option<ReaderAuth>,
    }

    pub type ReaderAuth = MdocCose<CoseSign1, Value>;
    pub type ReaderAuthenticationBytes = TaggedBytes<ReaderAuthentication>;
    pub type ReaderAuthentication = CborSeq<ReaderAuthenticationKeyed>;

    #[derive(Serialize, Deserialize, FieldNames, Debug, Clone)]
    pub struct ReaderAuthenticationKeyed {
        pub(crate) reader_auth_string: RequiredValue<ReaderAuthenticationString>,
        pub(crate) session_transcript: SessionTranscript,
        pub(crate) items_request_bytes: ItemsRequestBytes,
    }

    pub type ItemsRequestBytes = TaggedBytes<ItemsRequest>;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct ItemsRequest {
        #[serde(rename = "docType")]
        pub(crate) doc_type: DocType,
        #[serde(rename = "nameSpaces")]
        pub(crate) name_spaces: NameSpaces,
        #[serde(rename = "requestInfo")]
        pub(crate) request_info: Option<IndexMap<String, Value>>,
    }

    pub type NameSpaces = IndexMap<NameSpace, DataElements>;
    pub type DataElements = IndexMap<DataElementIdentifier, IndentToRetain>;
    pub type IndentToRetain = bool;
}

pub use credentials::*;
pub use device_authentication::*;
pub use device_retrieval::*;
pub use disclosure::*;
