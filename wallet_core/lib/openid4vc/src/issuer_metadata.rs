use std::collections::HashMap;
use std::num::NonZeroU64;

use derive_more::Into;
use josekit::jwk::Jwk;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use http_utils::data_uri::DataUri;
use http_utils::urls::BaseUrl;
use sd_jwt_vc_metadata::DisplayMetadata;
use sd_jwt_vc_metadata::NormalizedTypeMetadata;
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

    /// Boolean value specifying whether the Credential Issuer supports returning `credential_identifiers` parameter in
    /// the authorization_details Token Response parameter, with true indicating support. If omitted, the default value
    /// is `false`.
    pub credential_identifiers_supported: Option<bool>,

    /// Array of objects, where each object contains display properties of a Credential Issuer for a certain language.
    pub display: Option<Vec<IssuerDisplay>>,

    /// Object that describes specifics of the Credential that the Credential Issuer supports issuance of. This object
    /// contains a list of name/value pairs, where each name is a unique identifier of the supported Credential being
    /// described. This identifier is used in the Credential Offer as defined in Section 4.1.1 to communicate to the
    /// Wallet which Credential is being offered. The value is an object that contains metadata about a specific
    /// Credential.
    pub credential_configurations_supported: HashMap<String, CredentialMetadata>,
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
    pub name: Option<String>,
    pub locale: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logo {
    /// A URI where the Wallet can obtain the logo of the Credential Issuer. The Wallet needs
    /// to determine the scheme, since the URI value could use the `https:` scheme, the `data:` scheme, etc.
    pub uri: Url,

    /// String value of the alternative text for the logo image.
    pub alt_text: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialMetadata {
    /// Format of this Credential, i.e., `jwt_vc_json` or `ldp_vc`. Depending on the format value, may contain further
    /// elements defining the type and (optionally) particular claims the Credential MAY contain and information about
    /// how to display the Credential.
    #[serde(flatten)]
    pub format: CredentialFormat,

    /// A JSON string identifying the scope value that this Credential Issuer supports for this particular Credential.
    /// The value can be the same across multiple `credential_configurations_supported` objects. The Authorization
    /// Server MUST be able to uniquely identify the Credential Issuer based on the scope value. The Wallet can use
    /// this value in the Authorization Request as defined in Section 5.1.2. Scope values in this Credential Issuer
    /// metadata MAY duplicate those in the scopes_supported parameter of the Authorization Server.
    pub scope: Option<String>,

    /// Array of case sensitive strings that identify the representation of the cryptographic key material that the
    /// issued Credential is bound to, as defined in Section 7.1. Support for keys in JWK format [RFC7517] is indicated
    /// by the value `jwk`. Support for keys expressed as a COSE Key object [RFC8152] (for example, used in
    /// [ISO.18013-5]) is indicated by the value `cose_key`. When the Cryptographic Binding Method is a DID, valid
    /// values are a `did:` prefix followed by a method-name using a syntax as defined in Section 3.1 of [DID-Core],
    /// but without a `:` and method-specific-id. For example, support for the DID method with a method-name
    /// "example" would be represented by `did:example`.
    pub cryptographic_binding_methods_supported: Option<Vec<CryptographicBindingMethod>>,

    /// Array of case sensitive strings that identify the algorithms that the Issuer uses to sign the issued
    /// Credential.
    pub credential_signing_alg_values_supported: Option<Vec<CredentialSigningAlg>>,

    /// Object that describes specifics of the key proof(s) that the Credential Issuer supports. This object contains a
    /// list of name/value pairs, where each name is a unique identifier of the supported proof type(s). Valid values
    /// are defined in Section 7.2.1, other values MAY be used. This identifier is also used by the Wallet in the
    /// Credential Request as defined in Section 7.2. The value in the name/value pair is an object that contains
    /// metadata about the key proof.
    pub proof_types_supported: Option<HashMap<ProofType, ProofTypeData>>,

    /// Array of objects, where each object contains the display properties of the supported Credential for a certain
    /// language.
    pub display: Option<Vec<CredentialDisplay>>,
}

/// Format of a Credential, i.e., `jwt_vc_json` or `ldp_vc`. Depending on the format value, the object contains further
/// elements defining the type and (optionally) particular claims the Credential MAY contain and information about how
/// to display the Credential.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "format", rename_all = "snake_case")]
pub enum CredentialFormat {
    MsoMdoc {
        /// String identifying the Credential type, as defined in [ISO.18013-5].
        doctype: String,

        /// Object containing a list of name/value pairs, where the name is a certain namespace as defined in
        /// [ISO.18013-5] (or any profile of it), and the value is an object. This object also contains a list of
        /// name/value pairs, where the name is a claim name value that is defined in the respective namespace and is
        /// offered in the Credential. The value is an object detailing the specifics of the claim.
        claims: HashMap<String, HashMap<String, MsoMdocClaim>>,

        /// Array of namespaced claim name values that lists them in the order they should be displayed by the Wallet.
        /// The values MUST be two strings separated by a tilde ('~') character, where the first string is a namespace
        /// value and a second is a claim name value. For example, `org.iso.18013.5.1~given_name".
        order: Option<Vec<String>>,
    },

    // Allow the issuer to announce formats that the wallet doesn't support
    #[serde(untagged)]
    Other(serde_json::Value),
}

/// Metadata of an mdoc attribute.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MsoMdocClaim {
    /// Boolean which, when set to true, indicates that the Credential Issuer will always include this claim in the
    /// issued Credential. If set to `false`, the claim is not included in the issued Credential if the wallet did not
    /// request the inclusion of the claim, and/or if the Credential Issuer chose to not include the claim. If the
    /// mandatory parameter is omitted, the default value is `false`.
    pub mandatory: Option<bool>,

    ///  String value determining the type of value of the claim. Valid values defined by this specification are
    /// `string`, `number`, and `image` media types such as `image/jpeg` as defined in IANA media type registry for
    /// images (https://www.iana.org/assignments/media-types/media-types.xhtml#image). Other values MAY also be used.
    pub value_type: Option<String>,

    /// Array of objects, where each object contains display properties of a certain claim in the Credential for a
    /// certain language.
    pub display: Option<Vec<NameLocale>>,
}

/// Identifiers for the representation of the cryptographic key material that the issued Credential is bound to, as
/// defined in Section 7.1. Support for keys in JWK format [RFC7517] is indicated by the value `jwk`. Support for keys
/// expressed as a COSE Key object [RFC8152] (for example, used in [ISO.18013-5]) is indicated by the value `cose_key`.
/// When the Cryptographic Binding Method is a DID, valid values are a `did:` prefix followed by a method-name using a
/// syntax as defined in Section 3.1 of [DID-Core], but without a `:` and method-specific-id. For example, support for
/// the DID method with a method-name "example" would be represented by `did:example`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CryptographicBindingMethod {
    Jwk,
    CoseKey,

    // Allow the issuer to announce methods that the wallet doesn't support
    #[serde(untagged)]
    Other(String),
}

/// Algorithms that the Issuer uses to sign the issued Credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialSigningAlg {
    ES256,

    // Allow the issuer to announce algorithms that the wallet doesn't support
    #[serde(untagged)]
    Other(String),
}

/// Key proof type(s) that the Credential Issuer supports.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofType {
    Jwt,
    Cwt,
    LdpVp,

    // Allow the issuer to announce types that the wallet doesn't support
    #[serde(untagged)]
    Other(String),
}

/// Metadata of individual key proof type(s) that the Credential Issuer supports.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofTypeData {
    /// Array of case sensitive strings that identify the algorithms that the Issuer supports for this proof type.
    /// The Wallet uses one of them to sign the proof.
    pub proof_signing_alg_values_supported: Vec<ProofSigningAlg>,
}

/// Algorithms that the Issuer supports for a proof type. The Wallet uses one of them to sign the proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProofSigningAlg {
    ES256,

    // Allow the issuer to announce algorithms that the wallet doesn't support
    #[serde(untagged)]
    Other(String),
}

/// Display properties of a supported Credential for a certain language.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CredentialDisplay {
    /// String value of a display name for the Credential.
    pub name: String,

    /// String value that identifies the language of this object represented as a language tag taken from values
    /// defined in BCP47 [RFC5646].
    pub locale: Option<String>,

    /// Object with information about the logo of the Credential.
    pub logo: Option<Logo>,

    /// String value of a description of the Credential.
    pub description: Option<String>,

    /// String value of a background color of the Credential represented as numerical color values defined in CSS Color
    /// Module Level 37.
    pub background_color: Option<String>,

    /// Object with information about the background image of the Credential.
    pub background_image: Option<BackgroundImage>,

    /// String value of a text color of the Credential represented as numerical color values defined in CSS Color
    /// Module Level 37.
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
            _ => (None, None, None),
        };

        CredentialDisplay {
            name: value.name,
            locale: Some(value.lang),
            logo: logo.map(|logo| Logo {
                uri: Url::from(&DataUri::from(logo.image)),
                alt_text: Some(logo.alt_text.into_inner()),
            }),
            description: value.description,
            background_color,
            background_image: None,
            text_color,
        }
    }
}

impl CredentialMetadata {
    pub fn from_sd_jwt_vc_type_metadata(metadata: &NormalizedTypeMetadata) -> Self {
        Self {
            format: CredentialFormat::MsoMdoc {
                doctype: metadata.vct().to_string(),
                claims: HashMap::new(),
                order: None,
            },
            display: Some(
                metadata
                    .display()
                    .iter()
                    .map(|display| display.clone().into())
                    .collect(),
            ),
            scope: None,
            cryptographic_binding_methods_supported: Some(vec![CryptographicBindingMethod::CoseKey]),
            credential_signing_alg_values_supported: Some(vec![CredentialSigningAlg::ES256]),
            proof_types_supported: Some(HashMap::from([(
                ProofType::Jwt,
                ProofTypeData {
                    proof_signing_alg_values_supported: vec![ProofSigningAlg::ES256],
                },
            )])),
        }
    }
}

/// Object with information about the background image of the Credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundImage {
    /// String value that contains a URI where the Wallet can obtain the background image of the Credential from the
    /// Credential Issuer. The Wallet needs to determine the scheme, since the URI value could use the `https:` scheme,
    /// the `data:` scheme, etc.
    pub uri: BaseUrl,
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use crate::issuer_metadata::CredentialMetadata;
    use crate::issuer_metadata::CredentialSigningAlg;
    use crate::issuer_metadata::CryptographicBindingMethod;
    use crate::issuer_metadata::ProofSigningAlg;
    use crate::issuer_metadata::ProofType;

    use super::CredentialFormat;
    use super::IssuerMetadata;

    #[test]
    fn deserialize_example() {
        let example = r##"{
            "credential_issuer": "https://credential-issuer.example.com",
            "authorization_servers": [ "https://server.example.com" ],
            "credential_endpoint": "https://credential-issuer.example.com",
            "batch_credential_endpoint": "https://credential-issuer.example.com/batch_credential",
            "deferred_credential_endpoint": "https://credential-issuer.example.com/deferred_credential",
            "credential_response_encryption": {
                "alg_values_supported" : [
                    "ECDH-ES"
                ],
                "enc_values_supported" : [
                    "A128GCM"
                ],
                "encryption_required": false
            },
            "display": [
                {
                    "name": "Example University",
                    "locale": "en-US"
                },
                {
                    "name": "Example Université",
                    "locale": "fr-FR"
                }
            ],
            "credential_configurations_supported": {
                "UniversityDegreeCredential": {
                    "format": "jwt_vc_json",
                    "scope": "UniversityDegree",
                    "cryptographic_binding_methods_supported": [
                        "did:example"
                    ],
                    "credential_signing_alg_values_supported": [
                        "ES256"
                    ],
                    "credential_definition":{
                        "type": [
                            "VerifiableCredential",
                            "UniversityDegreeCredential"
                        ],
                        "credentialSubject": {
                            "given_name": {
                                "display": [
                                    {
                                        "name": "Given Name",
                                        "locale": "en-US"
                                    }
                                ]
                            },
                            "family_name": {
                                "display": [
                                    {
                                        "name": "Surname",
                                        "locale": "en-US"
                                    }
                                ]
                            },
                            "degree": {},
                            "gpa": {
                                "display": [
                                    {
                                        "name": "GPA"
                                    }
                                ]
                            }
                        }
                    },
                    "proof_types_supported": {
                        "jwt": {
                            "proof_signing_alg_values_supported": [
                                "ES256"
                            ]
                        }
                    },
                    "display": [
                        {
                            "name": "University Credential",
                            "locale": "en-US",
                            "logo": {
                                "uri": "https://university.example.edu/public/logo.png",
                                "alt_text": "a square logo of a university"
                            },
                            "background_color": "#12107c",
                            "text_color": "#FFFFFF"
                        }
                    ]
                }
            }
        }"##;

        let deserialized: IssuerMetadata = serde_json::from_str(example).unwrap();

        // Assert that some of the contents has the expected values
        assert_eq!(
            deserialized.credential_issuer.as_ref(),
            "https://credential-issuer.example.com"
        );
        let (cred_type, cred_metadata) = deserialized.credential_configurations_supported.iter().next().unwrap();
        assert_eq!(cred_type, "UniversityDegreeCredential");
        assert_matches!(cred_metadata.format, CredentialFormat::Other(..));
        assert_matches!(
            cred_metadata
                .credential_signing_alg_values_supported
                .as_ref()
                .unwrap()
                .first()
                .unwrap(),
            CredentialSigningAlg::ES256
        );
        assert_matches!(
            cred_metadata
                .cryptographic_binding_methods_supported
                .as_ref()
                .unwrap()
                .first()
                .unwrap(),
            CryptographicBindingMethod::Other(s) if s == "did:example"
        );
        let (proof_type, proof_type_data) = cred_metadata
            .proof_types_supported
            .as_ref()
            .unwrap()
            .iter()
            .next()
            .unwrap();
        assert_matches!(proof_type, ProofType::Jwt);
        assert_matches!(
            proof_type_data.proof_signing_alg_values_supported.first().unwrap(),
            ProofSigningAlg::ES256
        );
    }

    #[test]
    fn deserialize_mdoc_example() {
        let example = r##"{
            "format": "mso_mdoc",
            "doctype": "org.iso.18013.5.1.mDL",
            "cryptographic_binding_methods_supported": [
                "cose_key"
            ],
            "credential_signing_alg_values_supported": [
                "ES256", "ES384", "ES512"
            ],
            "display": [
                {
                    "name": "Mobile Driving License",
                    "locale": "en-US",
                    "logo": {
                        "uri": "https://state.example.org/public/mdl.png",
                        "alt_text": "state mobile driving license"
                    },
                    "background_color": "#12107c",
                    "text_color": "#FFFFFF"
                },
                {
                    "name": "モバイル運転免許証",
                    "locale": "ja-JP",
                    "logo": {
                        "uri": "https://state.example.org/public/mdl.png",
                        "alt_text": "米国州発行のモバイル運転免許証"
                    },
                    "background_color": "#12107c",
                    "text_color": "#FFFFFF"
                }
            ],
            "claims": {
                "org.iso.18013.5.1": {
                    "given_name": {
                        "display": [
                            {
                                "name": "Given Name",
                                "locale": "en-US"
                            },
                            {
                                "name": "名前",
                                "locale": "ja-JP"
                            }
                        ]
                    },
                    "family_name": {
                        "display": [
                            {
                                "name": "Surname",
                                "locale": "en-US"
                            }
                        ]
                    },
                    "birth_date": {
                        "mandatory": true
                    }
                },
                "org.iso.18013.5.1.aamva": {
                    "organ_donor": {}
                }
            }
        }"##;

        let deserialized: CredentialMetadata = serde_json::from_str(example).unwrap();

        // Assert that some of the contents has the expected values
        match deserialized.format {
            CredentialFormat::MsoMdoc { doctype, claims, order } => {
                assert_eq!(doctype, "org.iso.18013.5.1.mDL");
                let attrs = &claims["org.iso.18013.5.1"];
                let attr = &attrs["given_name"];
                assert_eq!(
                    attr.display.as_ref().unwrap().first().unwrap().name.as_ref().unwrap(),
                    "Given Name"
                );
                assert_matches!(order, None);
            }
            _ => panic!(),
        };
    }
}
