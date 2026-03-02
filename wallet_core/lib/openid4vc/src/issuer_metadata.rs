use std::collections::HashMap;
use std::num::NonZeroU64;
use std::ops::Not;

use derive_more::Into;
use itertools::Itertools;
use josekit::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;
use serde_with::MapPreventDuplicates;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use url::Url;

use attestation_types::claim_path::ClaimPath;
use http_utils::data_uri::DataUri;
use sd_jwt_vc_metadata::ClaimMetadata;
use sd_jwt_vc_metadata::DisplayMetadata;
use sd_jwt_vc_metadata::LogoMetadata;
use sd_jwt_vc_metadata::RenderingMetadata;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::issuer_identifier::IssuerIdentifier;
use crate::issuer_identifier::IssuerUrl;
use crate::jwe::JweAlgorithm;
use crate::jwe::JweCompressionAlgorithm;
use crate::jwe::JweEncryptionAlgorithm;

/// Credential issuer metadata, as per
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0.html#section-12.2.4>.
#[serde_as]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerMetadata {
    /// The Credential Issuer's identifier, as defined in Section 12.2.1.
    pub credential_issuer: IssuerIdentifier,

    /// A non-empty array of strings, where each string is an identifier of the OAuth 2.0 Authorization Server (as
    /// defined in [RFC8414]) the Credential Issuer relies on for authorization. If this parameter is omitted, the
    /// entity providing the Credential Issuer is also acting as the Authorization Server, i.e., the Credential Issuer's
    /// identifier is used to obtain the Authorization Server metadata. The actual OAuth 2.0 Authorization Server
    /// metadata is obtained from the `oauth-authorization-server` well-known location as defined in Section 3 of
    /// [RFC8414]. When there are multiple entries in the array, the Wallet may be able to determine which Authorization
    /// Server to use by querying the metadata; for example, by examining the `grant_types_supported` values, the Wallet
    /// can filter the server to use based on the grant type it plans to use. When the Wallet is using
    /// `authorization_server` parameter in the Credential Offer as a hint to determine which Authorization Server to
    /// use out of multiple, the Wallet MUST NOT proceed with the flow if the `authorization_server` Credential Offer
    /// parameter value does not match any of the entries in the `authorization_servers` array.
    pub authorization_servers: Option<VecNonEmpty<IssuerIdentifier>>,

    /// URL of the Credential Issuer's Credential Endpoint, as defined in Section 8.2. This URL MUST use the https
    /// scheme and MAY contain port, path, and query parameter components.
    pub credential_endpoint: IssuerUrl,

    // TODO (PVW-5554): Remove this field when removing the batch credential endpoint.
    pub batch_credential_endpoint: Option<IssuerUrl>,

    /// URL of the Credential Issuer's Nonce Endpoint, as defined in Section 7. This URL MUST use the https scheme and
    /// MAY contain port, path, and query parameter components. If omitted, the Credential Issuer does not require the
    /// use of `c_nonce``.
    pub nonce_endpoint: Option<IssuerUrl>,

    /// URL of the Credential Issuer's Deferred Credential Endpoint, as defined in Section 9. This URL MUST use the
    /// https scheme and MAY contain port, path, and query parameter components. If omitted, the Credential Issuer does
    /// not support the Deferred Credential Endpoint.
    pub deferred_credential_endpoint: Option<IssuerUrl>,

    /// URL of the Credential Issuer's Notification Endpoint, as defined in Section 11. This URL MUST use the https
    /// scheme and MAY contain port, path, and query parameter components. If omitted, the Credential Issuer does not
    /// support the Notification Endpoint.
    pub notification_endpoint: Option<IssuerUrl>,

    /// Object containing information about whether the Credential Issuer supports encryption of the Credential Request
    /// on top of TLS.
    pub credential_request_encryption: Option<CredentialRequestEncryption>,

    /// Object containing information about whether the Credential Issuer supports encryption of the Credential Response
    /// on top of TLS.
    pub credential_response_encryption: Option<CredentialResponseEncryption>,

    /// Object containing information about the Credential Issuer's support for issuance of multiple Credentials in a
    /// batch in the Credential Endpoint. The presence of this parameter means that the issuer supports more than one
    /// key proof in the proofs parameter in the Credential Request so can issue more than one Verifiable Credential for
    /// the same Credential Dataset in a single request/response.
    pub batch_credential_issuance: Option<BatchCredentialIssuance>,

    ///  non-empty array of objects, where each object contains display properties of a Credential Issuer for a certain
    /// language.
    pub display: Option<VecNonEmpty<IssuerDisplay>>,

    /// Object that describes specifics of the Credential that the Credential Issuer supports issuance of. This object
    /// contains a list of name/value pairs, where each name is a unique identifier of the supported Credential being
    /// described. This identifier is used in the Credential Offer as defined in Section 4.1.1 to communicate to the
    /// Wallet which Credential is being offered. The value is an object that contains metadata about a specific
    /// Credential.
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    pub credential_configurations_supported: HashMap<String, CredentialConfiguration>,
}

#[derive(Debug, thiserror::Error)]
pub enum IssuerMetadataDiscoveryError {
    #[error("could not fetch or deserialize credential issuer metadata: {0}")]
    Http(#[from] reqwest::Error),

    #[error("credential issuer identifier in metadata does not match, expected: {expected}, received: {received}")]
    IssuerIdentifierMismatch {
        expected: Box<IssuerIdentifier>,
        received: Box<IssuerIdentifier>,
    },
}

impl IssuerMetadata {
    /// Discover the Credential Issuer metadata by GETting it from .well-known and parsing it.
    pub(crate) async fn discover(
        client: &reqwest::Client,
        issuer_identifier: &IssuerIdentifier,
    ) -> Result<Self, IssuerMetadataDiscoveryError> {
        // TODO (PVW-5527): Composing of the `.well-known` path below is not compliant
        //                  with the OpenID4VCI specification and should be fixed.
        let metadata = client
            .get(
                issuer_identifier
                    .as_base_url()
                    .join("/.well-known/openid-credential-issuer"),
            )
            .send()
            .await?
            .error_for_status()?
            .json::<Self>()
            .await?;

        // As per specification, "The [credential issuer] MUST be identical to the Credential Issuer's identifier value
        // into which the well-known URI string was inserted to create the URL used to retrieve the metadata. If these
        // values are not identical (when compared using a simple string comparison with no normalization), the data
        // contained in the response MUST NOT be used."
        if metadata.credential_issuer != *issuer_identifier {
            return Err(IssuerMetadataDiscoveryError::IssuerIdentifierMismatch {
                expected: Box::new(issuer_identifier.clone()),
                received: Box::new(metadata.credential_issuer),
            });
        }

        Ok(metadata)
    }

    /// Returns a non-empty slice of authorization servers.
    pub fn authorization_servers(&self) -> VecNonEmpty<&IssuerIdentifier> {
        self.authorization_servers
            .as_ref()
            .map(|servers| servers.nonempty_iter().collect())
            .unwrap_or_else(|| {
                // Per the spec, "If [the authorization_servers] parameter is omitted, the entity
                // providing the Credential Issuer is also acting as the Authorization Server".
                vec_nonempty![&self.credential_issuer]
            })
    }

    /// Returns the maximum batch size that issuer supports. If it does not support batch issuance, this returns 1.
    // TODO (PVW-5554): Use this value for determining the amount of proofs to include.
    pub fn batch_size(&self) -> NonZeroU64 {
        self.batch_credential_issuance
            .map(|batch_issuance| batch_issuance.batch_size.into())
            .unwrap_or(NonZeroU64::MIN)
    }
}

// Information about whether the Credential Issuer supports encryption of the Credential Request on top of TLS.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialRequestEncryption {
    /// A JSON Web Key Set, as defined in [RFC7591], that contains one or more public keys, to be used by the Wallet as
    /// an input to a key agreement for encryption of the Credential Request. Each JWK in the set MUST have a kid (Key
    /// ID) parameter that uniquely identifies the key.
    // TODO (PVW-5538): Wrap these in a type like `JwePublicKey` to perform validation when actually implementing
    //                  request encryption. Additionally this should check for the presence of `kid` parameters.
    pub jwks: VecNonEmpty<Jwk>,

    /// A non-empty array containing a list of the JWE [RFC7516] encryption algorithms (enc values) [RFC7518] supported
    /// by the Credential Endpoint to decode the Credential Request from a JWT.
    pub enc_values_supported: VecNonEmpty<JweEncryptionAlgorithm>,

    /// A non-empty array containing a list of the JWE [RFC7516] compression algorithms (zip values) [RFC7518] supported
    /// by the Credential Endpoint to uncompress the Credential Request after decryption. If absent then no compression
    /// algorithms are supported. The Wallet may use any of the supported compression algorithm to compress the
    /// Credential Request prior to encryption.
    pub zip_values_supported: Option<VecNonEmpty<JweCompressionAlgorithm>>,

    /// Boolean value specifying whether the Credential Issuer requires the additional encryption on top of TLS for the
    /// Credential Requests. If the value is true, the Credential Issuer requires encryption for every Credential
    /// Request. If the value is false, the Wallet MAY choose whether it encrypts the request or not.
    pub encryption_required: bool,
}

/// Information about whether the Credential Issuer supports encryption of the Credential Response on top of TLS.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialResponseEncryption {
    /// A non-empty array containing a list of the JWE [RFC7516] encryption algorithms (alg values) [RFC7518] supported
    /// by the Credential Endpoint to encode the Credential Response in a JWT.
    pub alg_values_supported: VecNonEmpty<JweAlgorithm>,

    /// A non-empty array containing a list of the JWE [RFC7516] encryption algorithms (enc values) [RFC7518] supported
    /// by the Credential Endpoint to encode the Credential Response in a JWT
    pub enc_values_supported: VecNonEmpty<JweEncryptionAlgorithm>,

    /// A non-empty array containing a list of the JWE [RFC7516] compression algorithms (zip values) [RFC7518] supported
    /// by the Credential Endpoint to compress the Credential Response prior to encryption. If absent then compression
    /// is not supported.
    pub zip_values_supported: Option<VecNonEmpty<JweCompressionAlgorithm>>,

    /// Boolean value specifying whether the Credential Issuer requires the additional encryption on top of TLS for the
    /// Credential Response. If the value is true, the Credential Issuer requires encryption for every Credential
    /// Response and therefore the Wallet MUST provide encryption keys in the Credential Request. If the value is false,
    /// the Wallet MAY choose whether it provides encryption keys or not.
    pub encryption_required: bool,
}

#[derive(Debug, thiserror::Error)]
#[error("batch size must be 2 or greater, received: {0}")]
pub struct NonZeroOrOneU64Error(NonZeroU64);

#[derive(Debug, Clone, Copy, Into, Serialize, Deserialize)]
#[serde(try_from = "NonZeroU64", into = "NonZeroU64")]
pub struct NonZeroOrOneU64(NonZeroU64);

impl NonZeroOrOneU64 {
    pub fn try_new(size: NonZeroU64) -> Result<Self, NonZeroOrOneU64Error> {
        if size.get() < 2 {
            return Err(NonZeroOrOneU64Error(size));
        }

        Ok(Self(size))
    }
}

impl TryFrom<NonZeroU64> for NonZeroOrOneU64 {
    type Error = NonZeroOrOneU64Error;

    fn try_from(value: NonZeroU64) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

/// Information about the Credential Issuer's support for issuance of multiple Credentials in a batch in the Credential
/// Endpoint. The presence of this parameter means that the issuer supports more than one key proof in the proofs
/// parameter in the Credential Request so can issue more than one Verifiable Credential for the same Credential Dataset
/// in a single request/response.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct BatchCredentialIssuance {
    // Integer value specifying the maximum array size for the proofs parameter in a Credential Request. It MUST be 2 or
    // greater.
    pub batch_size: NonZeroOrOneU64,
}

/// Display properties of a Credential Issuer for a certain language.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssuerDisplay {
    /// A language identifier, and a name in that language.
    #[serde(flatten)]
    pub name_locale: NameLocale,

    /// Object with information about the logo of the Credential Issuer.
    pub logo: Option<Logo>,
}

/// A language identifier, and a name in that language.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NameLocale {
    /// String value of a display name for the Credential Issuer or Credential.
    pub name: Option<String>,

    /// String value that identifies the language of this object represented as a language tag taken from values defined
    /// in BCP47 [RFC5646]. There MUST be only one object for each language identifier.
    pub locale: Option<String>,
}

/// Information about the logo of the Credential Issuer or Credential.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logo {
    /// String value that contains a URI where the Wallet can obtain the logo. The Wallet needs to determine the scheme,
    /// since the URI value could use the `https:` scheme, the `data:` scheme, etc.
    pub uri: Url,

    /// String value of the alternative text for the logo image.
    pub alt_text: Option<String>,
}

impl From<LogoMetadata> for Logo {
    fn from(value: LogoMetadata) -> Self {
        Self {
            uri: Url::from(&DataUri::from(value.image)),
            alt_text: Some(value.alt_text.into_inner()),
        }
    }
}

/// Metadata about a specific Credential.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialConfiguration {
    /// The format of this Credential, i.e., `jwt_vc_json` or `ldp_vc`. Depending on the format value, the object
    /// contains further elements defining the type and (optionally) particular claims the Credential MAY contain and
    /// information about how to display the Credential.
    #[serde(flatten)]
    pub format: CredentialFormat,

    /// A JSON string identifying the scope value that this Credential Issuer supports for this particular Credential.
    /// The value can be the same across multiple `credential_configurations_supported` objects. The Authorization
    /// Server MUST be able to uniquely identify the Credential Issuer based on the scope value. The Wallet can use this
    /// value in the Authorization Request as defined in Section 5.1.2. Scope values in this Credential Issuer metadata
    /// MAY duplicate those in the `scopes_supported` parameter of the Authorization Server. If scope is absent, the
    /// only way to request the Credential is using authorization_details [RFC9396] - in this case, the OAuth
    /// Authorization Server metadata for one of the Authorization Servers found from the Credential Issuer's Metadata
    /// must contain an `authorization_details_types_supported` that contains `openid_credential`.
    pub scope: Option<String>,

    /// Combines the presence or absence of the `cryptographic_binding_methods_supported` and `proof_types_supported`
    /// fields.
    #[serde(flatten)]
    pub cryptographic_binding: Option<CryptographicBinding>,

    /// Object containing information relevant to the usage and display of issued Credentials. Credential
    /// Format-specific mechanisms can overwrite the information in this object to convey Credential metadata.
    /// Format-specific mechanisms, such as SD-JWT VC display metadata are always preferred by the Wallet over the
    /// information in this object, which serves as the default fallback.
    pub credential_metadata: Option<CredentialMetadata>,
}

impl CredentialConfiguration {
    pub fn new_mdoc_ecdsa_p256_sha256(
        doctype: String,
        proof_types: Vec<ProofType>,
        vc_display: Vec<DisplayMetadata>,
        vc_claims: Vec<ClaimMetadata>,
    ) -> Self {
        Self::new_ecdsa_p256_sha256(
            CredentialFormat::new_mdoc_ecdsa_p256_sha256(doctype),
            CryptographicBinding::new_mdoc_ecdsa_p256_sha256(proof_types),
            vc_display,
            vc_claims,
        )
    }

    pub fn new_sd_jwt_ecdsa_p256_sha256(
        vct: String,
        proof_types: Vec<ProofType>,
        vc_display: Vec<DisplayMetadata>,
        vc_claims: Vec<ClaimMetadata>,
    ) -> Self {
        Self::new_ecdsa_p256_sha256(
            CredentialFormat::new_sd_jwt_ecdsa_p256_sha256(vct),
            CryptographicBinding::new_sd_jwt_ecdsa_p256_sha256(proof_types),
            vc_display,
            vc_claims,
        )
    }

    fn new_ecdsa_p256_sha256(
        format: CredentialFormat,
        cryptographic_binding: CryptographicBinding,
        vc_display: Vec<DisplayMetadata>,
        vc_claims: Vec<ClaimMetadata>,
    ) -> Self {
        Self {
            format,
            scope: None,
            cryptographic_binding: Some(cryptographic_binding),
            credential_metadata: Some(CredentialMetadata::new_from_sd_jwt_vc(vc_display, vc_claims)),
        }
    }
}

/// Format of a Credential, i.e., `jwt_vc_json` or `ldp_vc`. Depending on the format value, the object contains further
/// elements defining the type and (optionally) particular claims the Credential MAY contain and information about how
/// to display the Credential.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialFormat {
    /// Mobile Documents or mdocs (ISO/IEC 18013).
    MsoMdoc {
        /// String identifying the Credential type, as defined in [ISO.18013-5].
        doctype: String,

        /// A non-empty array of algorithm identifiers that identify the algorithms that the Issuer uses to sign the
        /// issued Credential. Cryptographic algorithm identifiers used in the `credential_signing_alg_values_supported`
        /// parameter correspond to the numeric COSE algorithm identifiers used to secure the IssuerAuth COSE structure,
        /// as defined in [ISO.18013-5].
        credential_signing_alg_values_supported: Option<VecNonEmpty<CoseAlgorithmIdentifier>>,
    },

    /// IETF SD-JWT VC.
    #[serde(rename = "dc+sd-jwt")]
    SdJwt {
        /// String designating the type of the Credential, as defined in [I-D.ietf-oauth-sd-jwt-vc].
        vct: String,

        /// A non-empty array of algorithm identifiers that identify the algorithms that the Issuer uses to sign the
        /// issued Credential. Cryptographic algorithm identifiers used in the `credential_signing_alg_values_supported`
        /// parameter are case sensitive strings and SHOULD be one of those JWS Algorithm Names defined in [IANA.JOSE].
        credential_signing_alg_values_supported: Option<VecNonEmpty<JwsAlgorithm>>,
    },

    // Allow the issuer to announce formats that the wallet doesn't support.
    #[serde(untagged)]
    // Unfortunately serde does not allow us to capture just the format.
    Other { format: String },
}

impl CredentialFormat {
    fn new_mdoc_ecdsa_p256_sha256(doctype: String) -> Self {
        Self::MsoMdoc {
            doctype,
            credential_signing_alg_values_supported: Some(vec_nonempty![CoseAlgorithmIdentifier::Known(
                KnownCoseAlgorithmIdentifier::Esp256
            )]),
        }
    }

    fn new_sd_jwt_ecdsa_p256_sha256(vct: String) -> Self {
        Self::SdJwt {
            vct,
            credential_signing_alg_values_supported: Some(vec_nonempty![JwsAlgorithm::ES256]),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(from = "i64", into = "i64")]
pub enum CoseAlgorithmIdentifier {
    Known(KnownCoseAlgorithmIdentifier),

    // Allow the issuer to COSE algorithm identifiers that the wallet doesn't support.
    Unknown(i64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(i64)]
pub enum KnownCoseAlgorithmIdentifier {
    Es256 = -7,
    Esp256 = -9,
}

impl From<CoseAlgorithmIdentifier> for i64 {
    fn from(value: CoseAlgorithmIdentifier) -> Self {
        match value {
            CoseAlgorithmIdentifier::Known(known_identifier) => known_identifier as i64,
            CoseAlgorithmIdentifier::Unknown(identifier) => identifier,
        }
    }
}

impl From<i64> for CoseAlgorithmIdentifier {
    fn from(value: i64) -> Self {
        match KnownCoseAlgorithmIdentifier::from_repr(value) {
            Some(known_identifier) => Self::Known(known_identifier),
            None => Self::Unknown(value),
        }
    }
}

impl PartialEq for CoseAlgorithmIdentifier {
    fn eq(&self, other: &Self) -> bool {
        i64::from(*self) == i64::from(*other)
    }
}

impl Eq for CoseAlgorithmIdentifier {}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptographicBinding {
    /// A non-empty array of case sensitive strings that identify the representation of the cryptographic key material
    /// that the issued Credential is bound to, as defined in Section 8.1. It MUST be present when Cryptographic Key
    /// Binding is required for a Credential, and omitted otherwise. If absent, Cryptographic Key Binding is not
    /// required for this credential.
    pub cryptographic_binding_methods_supported: VecNonEmpty<CryptographicBindingMethod>,

    /// Object that describes specifics of the key proof(s) that the Credential Issuer supports. It MUST be present if
    /// `cryptographic_binding_methods_supported` is present, and omitted otherwise. If absent, the Wallet is not
    /// required to supply proofs when requesting this credential. This object contains a list of name/value pairs,
    /// where each name is a unique identifier of the supported proof type(s).
    #[serde_as(as = "MapPreventDuplicates<_, _>")]
    pub proof_types_supported: HashMap<ProofType, ProofMetadata>,
}

impl CryptographicBinding {
    fn new_mdoc_ecdsa_p256_sha256(proof_types: Vec<ProofType>) -> Self {
        Self::new_ecdsa_p256_sha256(CryptographicBindingMethod::CoseKey, proof_types)
    }

    fn new_sd_jwt_ecdsa_p256_sha256(proof_types: Vec<ProofType>) -> Self {
        Self::new_ecdsa_p256_sha256(CryptographicBindingMethod::Jwk, proof_types)
    }

    fn new_ecdsa_p256_sha256(binding_method: CryptographicBindingMethod, proof_types: Vec<ProofType>) -> Self {
        Self {
            cryptographic_binding_methods_supported: vec_nonempty![binding_method],
            proof_types_supported: proof_types
                .into_iter()
                .map(|proof_type| (proof_type, ProofMetadata::new_ecdsa_p256_sha256()))
                .collect(),
        }
    }
}

/// Identifiers the representation of the cryptographic key material that the issued Credential is bound to, as defined
/// in Section 8.1. It MUST be present when Cryptographic Key Binding is required for a Credential, and omitted
/// otherwise. If absent, Cryptographic Key Binding is not required for this credential. Support for keys in JWK format
/// [RFC7517] is indicated by the value `jwk`. Support for keys expressed as a COSE Key object [RFC8152] (for example,
/// used in [ISO.18013-5]) is indicated by the value `cose_key`. When the Cryptographic Key Binding method is a DID,
/// valid values are a `did:` prefix followed by a method-name using a syntax as defined in Section 3.1 of [DID-Core],
/// but without a `:` and method-specific-id. For example, support for the DID method with a method-name "example" would
/// be represented by `did:example`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CryptographicBindingMethod {
    Jwk,
    CoseKey,

    // Allow the issuer to announce methods that the wallet doesn't support.
    #[serde(untagged)]
    Other(String),
}

/// A proof type communicates a proof of cryptographic key material used for binding a Credential in the Credential
/// Request.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    Jwt,
    Attestation,

    // Allow the issuer to announce types that the wallet doesn't support.
    #[serde(untagged)]
    Other(String),
}

/// Metadata of an individual key proof that the Credential Issuer supports.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofMetadata {
    /// A non-empty array of algorithm identifiers that the Issuer supports for this proof type. The Wallet uses one of
    /// them to sign the proof. For the `jwt` and `attestation` proof types cryptographic algorithm identifiers are case
    /// sensitive strings and SHOULD be one of those defined in [IANA.JOSE].
    pub proof_signing_alg_values_supported: VecNonEmpty<JwsAlgorithm>,

    /// Object that describes the requirement for key attestations as described in Appendix D, which the Credential
    /// Issuer expects the Wallet to send within the proof(s) of the Credential Request. If the Credential Issuer does
    /// not require a key attestation, this parameter MUST NOT be present in the metadata.
    pub key_attestations_required: Option<KeyAttestationsRequired>,
}

impl ProofMetadata {
    fn new_ecdsa_p256_sha256() -> Self {
        Self {
            proof_signing_alg_values_supported: vec_nonempty![JwsAlgorithm::ES256],
            key_attestations_required: None,
        }
    }
}

/// Algorithms that the Issuer supports for a proof, as defined in [IANA.JOSE]. The Wallet uses one of them to sign the
/// proof.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JwsAlgorithm {
    ES256,

    // Allow the issuer to announce algorithms that the wallet doesn't support.
    #[serde(untagged)]
    Other(String),
}

/// Requirement for key attestations as described in Appendix D, which the Credential Issuer expects the Wallet to send
/// within the proof(s) of the Credential Request. If the Credential Issuer does not require a key attestation, this
/// parameter MUST NOT be present in the metadata. If both `key_storage` and `user_authentication` parameters are
/// absent, the `key_attestations_required` parameter may be empty, indicating a key attestation is needed without
/// additional constraints.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyAttestationsRequired {
    /// A non-empty array defining values specified in Appendix D.2 accepted by the Credential Issuer.
    pub key_storage: Option<VecNonEmpty<AttackPotentialResistance>>,

    /// A non-empty array defining values specified in Appendix D.2 accepted by the Credential Issuer.
    pub user_authentication: Option<VecNonEmpty<AttackPotentialResistance>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttackPotentialResistance {
    Iso18045High,
    Iso18045Moderate,
    #[serde(rename = "iso_18045_enhanced-basic")]
    Iso18045EnhancedBasic,
    Iso18045Basic,

    // Allow the issuer to announce an attack potential resistance that the wallet doesn't support.
    #[serde(untagged)]
    Other(String),
}

/// Information relevant to the usage and display of issued Credentials. Credential Format-specific mechanisms can
/// overwrite the information in this object to convey Credential metadata. Format-specific mechanisms, such as
/// SD-JWT VC display metadata are always preferred by the Wallet over the information in this object, which serves
/// as the default fallback.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    /// A non-empty array of objects, where each object contains the display properties of the supported Credential
    /// for a certain language.
    pub display: Option<VecNonEmpty<CredentialDisplay>>,

    /// A non-empty array of claims description objects as defined in Appendix B.2.
    pub claims: Option<VecNonEmpty<CredentialClaim>>,
}

impl CredentialMetadata {
    fn new_from_sd_jwt_vc(display: Vec<DisplayMetadata>, claims: Vec<ClaimMetadata>) -> Self {
        Self {
            display: display
                .into_iter()
                .map(CredentialDisplay::from)
                .collect_vec()
                .try_into()
                .ok(),
            claims: claims
                .into_iter()
                .map(CredentialClaim::from)
                .collect_vec()
                .try_into()
                .ok(),
        }
    }
}

/// Display properties of a supported Credential for a certain language.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDisplay {
    /// A language identifier, and a name in that language.
    #[serde(flatten)]
    pub name_locale: NameLocale,

    /// Object with information about the logo of the Credential.
    pub logo: Option<Logo>,

    /// String value of a description of the Credential.
    pub description: Option<String>,

    /// String value of a background color of the Credential represented as numerical color values defined in CSS Color
    /// Module Level 3 [CSS-Color].
    pub background_color: Option<String>,

    /// Object with information about the background image of the Credential.
    pub background_image: Option<BackgroundImage>,

    /// String value of a text color of the Credential represented as numerical color values defined in CSS Color
    /// Module Level 3 [CSS-Color].
    pub text_color: Option<String>,
}

impl From<DisplayMetadata> for CredentialDisplay {
    fn from(value: DisplayMetadata) -> Self {
        let (logo, background_color, text_color) = match value.rendering {
            Some(RenderingMetadata::Simple {
                logo,
                background_color,
                text_color,
            }) => (logo, background_color, text_color),
            Some(RenderingMetadata::SvgTemplates) | None => (None, None, None),
        };

        Self {
            name_locale: NameLocale {
                name: Some(value.name),
                locale: Some(value.lang),
            },
            logo: logo.map(Logo::from),
            description: value.description,
            background_color,
            background_image: None,
            text_color,
        }
    }
}

/// Information about the background image of the Credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// String value that contains a URI where the Wallet can obtain the background image of the Credential from the
    /// Credential Issuer. The Wallet needs to determine the scheme, since the URI value could use the `https:` scheme,
    /// the `data:` scheme, etc.
    pub uri: Url,
}

const fn bool_value<const B: bool>() -> bool {
    B
}

/// A claims description object as used in the Credential Issuer metadata is an object used to describe how a certain
/// claim in the Credential is displayed to the End-User.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialClaim {
    /// The value MUST be a non-empty array representing a claims path pointer that specifies the path to a claim within
    /// the credential, as defined in Appendix C.
    pub path: VecNonEmpty<ClaimPath>,

    /// Boolean which, when set to `true`, indicates that the Credential Issuer will always include this claim in the
    /// issued Credential. If set to `false`, the claim is not included in the issued Credential if the wallet did not
    /// request the inclusion of the claim, and/or if the Credential Issuer chose to not include the claim. If the
    /// mandatory parameter is omitted, the default value is `false`.
    #[serde(default = "bool_value::<false>", skip_serializing_if = "<&bool>::not")]
    pub mandatory: bool,

    /// A non-empty array of objects, where each object contains display properties of a certain claim in the Credential
    /// for a certain language.
    pub display: Option<VecNonEmpty<NameLocale>>,
}

impl From<ClaimMetadata> for CredentialClaim {
    fn from(value: ClaimMetadata) -> Self {
        Self {
            path: value.path,
            mandatory: false,
            display: value
                .display
                .into_iter()
                .map(|display| NameLocale {
                    name: Some(display.label),
                    locale: Some(display.lang),
                })
                .collect_vec()
                .try_into()
                .ok(),
        }
    }
}
