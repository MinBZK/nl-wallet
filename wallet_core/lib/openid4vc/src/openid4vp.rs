use std::collections::HashMap;
use std::fmt;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::sync::LazyLock;
use std::time::Duration;

use base64::prelude::*;
use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use futures::future::try_join_all;
use indexmap::IndexSet;
use itertools::Itertools;
use josekit::JoseError;
use josekit::jwe::JweHeader;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use josekit::jwe::alg::ecdh_es::EcdhEsJweEncrypter;
use josekit::jwk::Jwk;
use josekit::jwk::alg::ec::EcKeyPair;
use josekit::jwt::JwtPayload;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::DeserializeOwned;
use serde::de::value::StringDeserializer;
use serde_with::DeserializeAs;
use serde_with::DeserializeFromStr;
use serde_with::SerializeAs;
use serde_with::SerializeDisplay;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use attestation_data::disclosure::DisclosedAttestation;
use attestation_data::disclosure::DisclosedAttestationError;
use attestation_data::disclosure::DisclosedAttestations;
use crypto::utils::sha256;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use dcql::CredentialQueryIdentifier;
use dcql::Query;
use dcql::disclosure::CredentialValidationError;
use dcql::disclosure::ExtendingVctRetriever;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::normalized::UnsupportedDcqlFeatures;
use dcql::unique_id_vec::UniqueIdVec;
use error_category::ErrorCategory;
use http_utils::urls::BaseUrl;
use jwt::Algorithm;
use jwt::JwtTyp;
use jwt::UnverifiedJwt;
use jwt::Validation;
use jwt::error::JwtX5cError;
use jwt::headers::HeaderWithX5c;
use mdoc::DeviceResponse;
use mdoc::SessionTranscript;
use mdoc::utils::serialization::CborBase64;
use sd_jwt::key_binding_jwt::KbVerificationOptions;
use sd_jwt::sd_jwt::UnverifiedSdJwtPresentation;
use token_status_list::verification::client::StatusListClient;
use token_status_list::verification::verifier::RevocationStatus;
use token_status_list::verification::verifier::RevocationVerifier;
use utils::generator::Generator;
use utils::generator::TimeGenerator;
use utils::vec_at_least::IntoNonEmptyIterator;
use utils::vec_at_least::NonEmptyIterator;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;
use wscd::Poa;
use wscd::PoaVerificationError;

use crate::authorization::AuthorizationRequest;
use crate::authorization::ResponseMode;
use crate::authorization::ResponseType;
use crate::jwe::JweEncryptionAlgorithm;

/// Leeway used in the lower end of the `iat` verification, used to account for clock skew.
const SD_JWT_IAT_LEEWAY: Duration = Duration::from_secs(5);
const SD_JWT_IAT_WINDOW: Duration = Duration::from_secs(15 * 60);

/// OpenID4VP request uri.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpRequestUri {
    /// MUST equal the client_id from the full Authorization Request.
    pub client_id: ClientId,

    #[serde(flatten)]
    pub object: VpRequestUriObject,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Keeping these names as is might make more sense, but the spec says lowercase
pub enum VpRequestUriMethod {
    #[default]
    GET,
    POST,
}

/// The three ways an OpenID4VP request URI object can be conveyed, as described here:
/// https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-5.4
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum VpRequestUriObject {
    /// A request object by reference, i.e., contains a `request_uri` parameter
    /// pointing to where to fetch the actual authorization request.
    /// Note that this is the only supported variant of VpRequestUriObject.
    AsReference {
        request_uri: BaseUrl,
        request_uri_method: Option<VpRequestUriMethod>,
    },

    /// A request object by value, i.e., a `request` parameter with an inline JWT.
    /// Note that technically, instead of String, the request value could be an
    /// UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>, but since we don't
    /// actually support AsValue, we (currently) don't bother.
    AsValue { request: String },

    /// A direct authorization request with required fields as query parameters.
    /// Note that, as with AsValue, we do not support the AsQueryParameters variant.
    AsQueryParameters { response_type: String, nonce: String },
}

/// An OpenID4VP Authorization Request, allowing an RP to request a set of credentials/attributes from a wallet.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpAuthorizationRequest {
    pub aud: VpAuthorizationRequestAudience,

    #[serde(flatten)]
    pub oauth_request: AuthorizationRequest,

    /// Contains requirements on the credentials and/or attributes to be disclosed.
    pub dcql_query: Query,

    /// Metadata about the verifier such as their encryption key(s).
    pub client_metadata: Option<VpClientMetadata>,

    /// REQUIRED if the ResponseMode `direct_post` or `direct_post.jwt` is used.
    /// In that case, the Authorization Response is to be posted to this URL by the wallet.
    pub response_uri: Option<BaseUrl>,

    pub wallet_nonce: Option<String>,

    #[serde_as(as = "Option<Vec<JsonBase64>>")]
    pub transaction_data: Option<VecNonEmpty<serde_json::Map<String, serde_json::Value>>>,
}

impl JwtTyp for VpAuthorizationRequest {}

/// JsonBase64, a base64url-encoded JSON object. Used currently for transaction_data.
/// See: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-5.1
pub struct JsonBase64;

impl<T> SerializeAs<T> for JsonBase64
where
    T: Serialize,
{
    fn serialize_as<S>(source: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let bytes = serde_json::to_vec(source).map_err(serde::ser::Error::custom)?;
        let base64 = BASE64_URL_SAFE_NO_PAD.encode(bytes).serialize(serializer)?;

        Ok(base64)
    }
}

impl<'de, T> DeserializeAs<'de, T> for JsonBase64
where
    T: DeserializeOwned,
{
    fn deserialize_as<D>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base64 = String::deserialize(deserializer)?;
        let bytes = BASE64_URL_SAFE_NO_PAD
            .decode(base64)
            .map_err(|error| serde::de::Error::custom(format!("error decoding entry as base64: {error}")))?;
        let value = serde_json::from_slice(bytes.as_slice())
            .map_err(|error| serde::de::Error::custom(format!("error parsing entry as JSON: {error}")))?;

        Ok(value)
    }
}

#[derive(Debug, Clone, Default, SerializeDisplay, DeserializeFromStr, strum::EnumString, strum::Display)]
pub enum VpAuthorizationRequestAudience {
    #[default]
    #[strum(to_string = "https://self-issued.me/v2")]
    SelfIssued,
}

/// Metadata of the verifier (which acts as the "client" in OAuth).
/// https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-11
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpClientMetadata {
    pub jwks: VpJwks,
    pub vp_formats: VpFormat,

    /// Non-empty array of strings, where each string is a JWE [RFC7516] enc algorithm that can be used as the content
    /// encryption algorithm for encrypting the Response. When a response_mode requiring encryption of the Response
    /// (such as `dc_api.jwt`` or `direct_post.jwt``) is specified, this MUST be present for anything other than the
    /// default single value of `A128GCM``. Otherwise, this SHOULD be absent.
    pub encrypted_response_enc_values_supported: Option<VecNonEmpty<JweEncryptionAlgorithm>>,
}

/// `client_id` prefix values as defined by OpenID4VP 1.0 section 5.9.3.
/// <https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-5.9.3>
#[derive(Debug, Clone, PartialEq, Eq, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum ClientIdScheme {
    RedirectUri,
    OpenidFederation,
    DecentralizedIdentifier,
    VerifierAttestation,
    X509SanDns,
    X509Hash,
    Origin,
    #[strum(default)]
    Other(String),
}

#[derive(Debug, Clone, SerializeDisplay, DeserializeFromStr, PartialEq, Eq)]
pub struct ClientId {
    id: String,
    scheme: Option<ClientIdScheme>,
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.scheme {
            Some(scheme) => write!(f, "{scheme}:{}", self.id),
            // https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#name-fallback
            None => write!(f, "{}", self.id),
        }
    }
}

impl From<&str> for ClientId {
    fn from(s: &str) -> Self {
        if let Some((scheme_str, id)) = s.split_once(':') {
            Self {
                scheme: Some(scheme_str.into()),
                id: id.to_string(),
            }
        } else {
            Self {
                // https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#name-fallback
                scheme: None,
                id: s.to_string(),
            }
        }
    }
}

impl FromStr for ClientId {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(ClientId::from(s))
    }
}

impl ClientId {
    pub fn x509_san_dns(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            scheme: Some(ClientIdScheme::X509SanDns),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpJwks {
    pub keys: VecNonEmpty<Jwk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VpFormat {
    MsoMdoc { alg: IndexSet<FormatAlg> },
    SdJwt { alg: IndexSet<FormatAlg> },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FormatAlg {
    #[default]
    ES256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletRequest {
    pub wallet_nonce: Option<String>,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)] // might leak sensitive data
pub enum JwePublicKeyError {
    #[error("unsupported JWK: expected {expected}, found {found:?} in {field}")]
    UnsupportedJwk {
        field: &'static str,
        expected: &'static str,
        found: Option<Box<serde_json::Value>>,
    },

    #[error("error parsing JWK: {0}")]
    JwkParsing(#[source] JoseError),
}

/// Wraps a [`Jwk`], performs validation on it and ensures that this contains an EC public key on the P256 curve,
/// with a `kid` parameter. The underlying encrypter construction performs the remaining JWE-specific validation.
/// Unfortunately, `josekit` actually does little validation when deserializing its `Jwk` type, opting instead to
/// perform validations when this type is used. Since this goes counter to our principle of validating on input and
/// failing early, we eagerly create a [`EcdhEsJweEncrypter`] here, which is where `josekit` actually performs the
/// requisite validations on the contents of the JWK.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "Jwk")]
pub struct JwePublicKey {
    #[serde(flatten)]
    jwk: Jwk,
    #[serde(skip)]
    encrypter: EcdhEsJweEncrypter,
}

impl JwePublicKey {
    pub fn try_new(jwk: Jwk) -> Result<Self, JwePublicKeyError> {
        // Avoid jwk.key_type() which panics if `kty` is not set.
        if jwk_parameter_as_str(&jwk, "kty") != Some("EC") {
            return Err(JwePublicKeyError::UnsupportedJwk {
                field: "kty",
                expected: "EC",
                found: jwk_parameter_value(&jwk, "kty"),
            });
        }

        if jwk_parameter_as_str(&jwk, "crv") != Some("P-256") {
            return Err(JwePublicKeyError::UnsupportedJwk {
                field: "crv",
                expected: "P-256",
                found: jwk_parameter_value(&jwk, "crv"),
            });
        }

        if jwk_parameter_as_str(&jwk, "kid").is_none() {
            return Err(JwePublicKeyError::UnsupportedJwk {
                field: "kid",
                expected: "a string",
                found: jwk_parameter_value(&jwk, "kid"),
            });
        }

        let encrypter = EcdhEsJweAlgorithm::EcdhEs
            .encrypter_from_jwk(&jwk)
            .map_err(JwePublicKeyError::JwkParsing)?;

        let jwe_pubkey = Self { jwk, encrypter };

        Ok(jwe_pubkey)
    }

    pub fn jwk(&self) -> &Jwk {
        &self.jwk
    }

    pub fn encrypter(&self) -> &EcdhEsJweEncrypter {
        &self.encrypter
    }

    pub fn kid(&self) -> &str {
        jwk_parameter_as_str(&self.jwk, "kid").expect("JwePublicKey guarantees presence of kid parameter")
    }

    /// Generate the bytes of a RFC 7638 compliant SHA-256 thumbprint, without URL-safe Base64 encoding.
    /// Unfortunately, `josekit` does not include this future, so we have to generate it ourselves.
    pub fn sha256_thumbprint_bytes(&self) -> Vec<u8> {
        // These values are guaranteed to exist by this type's validation.
        let (x, y) = ["x", "y"]
            .iter()
            .map(|field| self.jwk.parameter(field).unwrap().as_str().unwrap())
            .collect_tuple()
            .unwrap();

        sha256(format!(r#"{{"crv":"P-256","kty":"EC","x":"{x}","y":"{y}"}}"#).as_bytes())
    }
}

impl TryFrom<Jwk> for JwePublicKey {
    type Error = JwePublicKeyError;

    fn try_from(value: Jwk) -> Result<Self, Self::Error> {
        Self::try_new(value)
    }
}

fn jwk_parameter_as_str<'a>(jwk: &'a Jwk, field: &str) -> Option<&'a str> {
    jwk.parameter(field).and_then(serde_json::Value::as_str)
}

fn jwk_parameter_value(jwk: &Jwk, field: &str) -> Option<Box<serde_json::Value>> {
    jwk.parameter(field).cloned().map(Box::new)
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum AuthRequestValidationError {
    #[error("unexpected field: {0}")]
    #[category(critical)]
    UnexpectedField(&'static str),
    #[error("unsupported field: {0}")]
    #[category(critical)]
    UnsupportedField(&'static str),
    #[error("missing required field: {0}")]
    #[category(critical)]
    ExpectedFieldMissing(&'static str),
    #[error("unsupported value for field {field}: expected {expected}, found {found}")]
    #[category(pd)]
    UnsupportedFieldValue {
        field: &'static str,
        expected: &'static str,
        found: String,
    },
    #[error("missing kid in JWK at client_metadata.jwks[{0}]")]
    #[category(critical)]
    MissingJwkKid(usize),
    #[error(
        "no supported JWK found in client_metadata.jwks: expected at least one EC/P-256 JWK with alg ECDH-ES: {errors}",
        errors = .0.iter().join(", ")
    )]
    #[category(pd)]
    NoSupportedJwk(Vec<JwePublicKeyError>),
    #[error(
        "no supported value found in client_metadata.encrypted_response_enc_values_supported: expected at least one \
         of A128GCM, A192GCM or A256GCM"
    )]
    #[category(critical)]
    NoSupportedEncryptedResponseEnc,
    #[error("{0}")]
    #[category(pd)]
    Jwk(#[from] JwePublicKeyError),
    #[error("unsupported client_id scheme: {scheme}. Only x509_san_dns is currently supported")]
    #[category(critical)]
    UnsupportedClientIdScheme { scheme: ClientIdScheme },
    #[error("unsupported client_id without a scheme (pre-registered). Only x509_san_dns is currently supported")]
    #[category(critical)]
    UnsupportedClientIdWithoutScheme,
    #[error("response_uri fqdn {fqdn} does not match client_id {id}.")]
    #[category(critical)]
    UnmatchedResponseFqdn { fqdn: String, id: String },
    #[error("unsupported DCQL query: {0}")]
    UnsupportedDcqlQuery(#[from] UnsupportedDcqlFeatures),
    #[error(
        "client_id from Authorization Request was {client_id}, should have been x509_san_dns:{dns_san} to match the \
         SAN DNSName from the X.509 certificate"
    )]
    #[category(critical)]
    UnauthorizedClientId { client_id: String, dns_san: String },
    #[error("Subject Alternative Name missing from X.509 certificate")]
    #[category(critical)]
    MissingSAN,
    #[error("error parsing X.509 certificate: {0}")]
    CertificateParsing(#[from] CertificateError),
    #[error("failed to verify Authorization Request JWT: {0}")]
    JwtVerification(#[from] JwtX5cError),
    #[error("mismatch in wallet nonce: did not receive nonce when one was expected, or vice versa")]
    #[category(critical)]
    WalletNonceMismatch,
}

static AUD_VALIDATIONS: LazyLock<Validation> = LazyLock::new(|| {
    let mut validation = Validation::new(Algorithm::ES256);
    validation.set_required_spec_claims(&["aud"]);
    validation
});

impl VpAuthorizationRequest {
    /// Construct a new Authorization Request by verifying an Authorization Request JWT against
    /// the specified trust anchors.
    pub fn try_new(
        jws: &UnverifiedJwt<VpAuthorizationRequest, HeaderWithX5c>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(VpAuthorizationRequest, BorrowingCertificate), AuthRequestValidationError> {
        let mut validation_options = AUD_VALIDATIONS.to_owned();
        validation_options.set_audience(&[VpAuthorizationRequestAudience::SelfIssued.to_string()]);

        let (header, auth_request) = jws.parse_and_verify_against_trust_anchors(
            trust_anchors,
            &TimeGenerator,
            CertificateUsage::ReaderAuth,
            &validation_options,
        )?;

        Ok((auth_request, header.x5c.into_first()))
    }

    /// Validate that an Authorization Request satisfies the following:
    /// - the request contents are compliant with the OpenID4VP specification.
    /// - the `client_id` uses the `x509_san_dns` scheme and equals one of the DNS SAN names in the X.509
    ///   certificate.
    ///
    /// This method consumes `self` and turns it into an [`NormalizedVpAuthorizationRequest`], which
    /// contains only the fields we need and use.
    pub fn validate(
        self,
        rp_cert: &BorrowingCertificate,
        wallet_nonce: Option<&str>,
    ) -> Result<(NormalizedVpAuthorizationRequest, JweEncryptionAlgorithm), AuthRequestValidationError> {
        let dns_sans = rp_cert.san_dns_names()?;
        if dns_sans.is_empty() {
            return Err(AuthRequestValidationError::MissingSAN);
        }
        let (validated_auth_request, selected_encryption_algorithm) =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(self)?;
        let client_id = &validated_auth_request.client_id;

        match &client_id.scheme {
            Some(ClientIdScheme::X509SanDns) => {}
            Some(scheme) => {
                return Err(AuthRequestValidationError::UnsupportedClientIdScheme { scheme: scheme.clone() });
            }
            None => return Err(AuthRequestValidationError::UnsupportedClientIdWithoutScheme),
        }

        if !dns_sans.contains(&client_id.id.as_str()) {
            return Err(AuthRequestValidationError::UnauthorizedClientId {
                client_id: client_id.to_string(),
                dns_san: dns_sans.join(", "),
            });
        }

        if wallet_nonce != validated_auth_request.wallet_nonce.as_deref() {
            return Err(AuthRequestValidationError::WalletNonceMismatch);
        }

        // https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-8.2
        // This checks the fqdn of the response uri against the x509_san_dns client id
        if let Some(response_uri_fqdn) = validated_auth_request.response_uri.fqdn() {
            if response_uri_fqdn != client_id.id {
                return Err(AuthRequestValidationError::UnmatchedResponseFqdn {
                    fqdn: response_uri_fqdn.to_string(),
                    id: client_id.id.clone(),
                });
            }
        } else {
            return Err(AuthRequestValidationError::UnmatchedResponseFqdn {
                fqdn: validated_auth_request.response_uri.to_string(),
                id: client_id.id.clone(),
            });
        }

        Ok((validated_auth_request, selected_encryption_algorithm))
    }
}

/// An OpenID4VP Authorization Request that has been validated to conform to the OpenID4VP specification:
/// a subset of [`VpAuthorizationRequest`] that always contains fields we require, and no fields we don't.
///
/// Note that this data type is internal to both the wallet and verifier, and not part of the OpenID4VP protocol,
/// so it is never sent over the wire. It implements (De)serialize so that the verifier can persist it to
/// the session store.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedVpAuthorizationRequest {
    pub client_id: ClientId,
    pub nonce: String,
    pub encryption_pubkey: JwePublicKey,
    pub response_uri: BaseUrl,
    pub credential_requests: NormalizedCredentialRequests,
    pub client_metadata: VpClientMetadata,
    pub state: Option<String>,
    pub wallet_nonce: Option<String>,
}

impl NormalizedVpAuthorizationRequest {
    /// Construct an Authorization Request to be sent by this verifier.
    pub fn new_for_verifier(
        credential_requests: NormalizedCredentialRequests,
        client_id: ClientId,
        nonce: String,
        encryption_pubkey: JwePublicKey,
        response_uri: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Self {
        let jwk = encryption_pubkey.jwk().clone();

        Self {
            client_id,
            nonce,
            encryption_pubkey,
            response_uri,
            credential_requests,
            client_metadata: VpClientMetadata {
                jwks: VpJwks {
                    keys: vec_nonempty![jwk],
                },
                vp_formats: VpFormat::MsoMdoc {
                    alg: IndexSet::from([FormatAlg::ES256]),
                },
                // HAIP requires verifiers to list both A128GCM and A256GCM in
                // `encrypted_response_enc_values_supported`:
                // https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-5
                // The JWE enc (encryption algorithm) header parameter (see Section 4.1.2 of [RFC7516]) values A128GCM and A256GCM (as defined in Section 5.3 of [RFC7518]) MUST be supported by Verifiers.
                encrypted_response_enc_values_supported: Some(vec_nonempty![
                    JweEncryptionAlgorithm::A128Gcm,
                    JweEncryptionAlgorithm::A256Gcm
                ]),
            },
            state: None,
            wallet_nonce,
        }
    }

    fn select_encryption_algorithm(
        client_metadata: &VpClientMetadata,
    ) -> Result<JweEncryptionAlgorithm, AuthRequestValidationError> {
        let encryption_algorithm = client_metadata
            .encrypted_response_enc_values_supported
            .as_ref()
            .map(|enc_values| {
                enc_values
                    .iter()
                    .filter_map(|alg| alg.preference_rank().map(|rank| (rank, alg)))
                    .max_by_key(|(rank, _)| *rank)
                    .map(|(_, alg)| alg.clone())
                    .ok_or(AuthRequestValidationError::NoSupportedEncryptedResponseEnc)
            })
            .transpose()?
            .unwrap_or_default();

        Ok(encryption_algorithm)
    }

    fn try_from_with_selected_encryption_algorithm(
        vp_auth_request: VpAuthorizationRequest,
    ) -> Result<(Self, JweEncryptionAlgorithm), AuthRequestValidationError> {
        // Check absence of fields that must not be present in an OpenID4VP Authorization Request
        if vp_auth_request.oauth_request.authorization_details.is_some() {
            return Err(AuthRequestValidationError::UnexpectedField("authorization_details"));
        }
        if vp_auth_request.oauth_request.code_challenge.is_some() {
            return Err(AuthRequestValidationError::UnexpectedField("code_challenge"));
        }
        if vp_auth_request.oauth_request.redirect_uri.is_some() {
            return Err(AuthRequestValidationError::UnexpectedField("redirect_uri"));
        }
        if vp_auth_request.oauth_request.scope.is_some() {
            return Err(AuthRequestValidationError::UnexpectedField("scope"));
        }
        if vp_auth_request.oauth_request.request_uri.is_some() {
            return Err(AuthRequestValidationError::UnexpectedField("request_uri"));
        }
        if vp_auth_request.transaction_data.is_some() {
            return Err(AuthRequestValidationError::UnsupportedField("transaction_data"));
        }

        // Check presence of fields that must be present in an OpenID4VP Authorization Request
        if vp_auth_request.oauth_request.nonce.is_none() {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("nonce"));
        }
        if vp_auth_request.oauth_request.response_mode.is_none() {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("response_mode"));
        }
        if vp_auth_request.response_uri.is_none() {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("response_uri"));
        }
        let Some(client_metadata) = vp_auth_request.client_metadata else {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("client_metadata"));
        };

        // Check that various enums have the expected values
        if vp_auth_request.oauth_request.response_type != ResponseType::VpToken.into() {
            return Err(AuthRequestValidationError::UnsupportedFieldValue {
                field: "response_type",
                expected: "vp_token",
                found: serde_json::to_string(&vp_auth_request.oauth_request.response_type).unwrap(),
            });
        }
        if vp_auth_request.oauth_request.response_mode.unwrap() != ResponseMode::DirectPostJwt {
            return Err(AuthRequestValidationError::UnsupportedFieldValue {
                field: "response_mode",
                expected: "direct_post.jwt",
                found: serde_json::to_string(&vp_auth_request.oauth_request.response_mode).unwrap(),
            });
        }
        let jwks = &client_metadata.jwks.keys;

        // OpenID4VP 1.0 mandates that every JWK in verifier metadata has a `kid`.
        if let Some((index, _)) = jwks
            .iter()
            .enumerate()
            .find(|(_, jwk)| jwk_parameter_as_str(jwk, "kid").is_none())
        {
            return Err(AuthRequestValidationError::MissingJwkKid(index));
        }

        // Choose the first key the wallet supports (currently ECDH-ES on P-256).
        let mut jwk_errors = Vec::new();
        let mut encryption_pubkey = None;

        for jwk in jwks.iter().cloned() {
            match JwePublicKey::try_new(jwk) {
                Ok(supported_jwk) => {
                    encryption_pubkey = Some(supported_jwk);
                    break;
                }
                Err(error) => jwk_errors.push(error),
            }
        }

        let encryption_pubkey = encryption_pubkey.ok_or(AuthRequestValidationError::NoSupportedJwk(jwk_errors))?;

        // Validate that the verifier advertised at least one encryption algorithm that the wallet supports,
        // and keep the selected value for later use on the wallet encryption path.
        // See: <https://openid.net/specs/openid4vc-high-assurance-interoperability-profile-1_0.html#section-5-2.5>.
        //
        // If the verifier did not send any supported algorithms, default to AES128GCM.
        // See: <https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#section-5.1-2.4.2.2>
        let selected_encryption_algorithm = Self::select_encryption_algorithm(&client_metadata)?;

        let client_id = vp_auth_request.oauth_request.client_id.as_str().into();

        Ok((
            NormalizedVpAuthorizationRequest {
                client_id,
                nonce: vp_auth_request.oauth_request.nonce.unwrap(),
                encryption_pubkey,
                credential_requests: vp_auth_request.dcql_query.try_into()?,
                response_uri: vp_auth_request.response_uri.unwrap(),
                client_metadata,
                state: vp_auth_request.oauth_request.state,
                wallet_nonce: vp_auth_request.wallet_nonce,
            },
            selected_encryption_algorithm,
        ))
    }

    pub fn session_transcript(&self) -> SessionTranscript {
        SessionTranscript::new_oid4vp(
            &self.client_id.to_string(),
            &self.nonce,
            Some(&self.encryption_pubkey.sha256_thumbprint_bytes()),
            &self.response_uri,
        )
    }
}

impl From<NormalizedVpAuthorizationRequest> for VpAuthorizationRequest {
    fn from(value: NormalizedVpAuthorizationRequest) -> Self {
        Self {
            aud: VpAuthorizationRequestAudience::SelfIssued,
            oauth_request: AuthorizationRequest {
                response_type: ResponseType::VpToken.into(),
                client_id: value.client_id.to_string(),
                nonce: Some(value.nonce),
                response_mode: Some(ResponseMode::DirectPostJwt),
                redirect_uri: None,
                state: None,
                authorization_details: None,
                request_uri: None,
                code_challenge: None,
                scope: None,
            },
            dcql_query: value.credential_requests.into(),
            client_metadata: Some(value.client_metadata),
            response_uri: Some(value.response_uri),
            wallet_nonce: value.wallet_nonce,
            transaction_data: None,
        }
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(unexpected)]
pub enum AuthResponseError {
    #[error("error (de)serializing JWE payload: {0}")]
    Json(#[from] serde_json::Error),

    #[error("error encrypting/decrypting JWE: {0}")]
    Jwe(#[source] JoseError),

    #[error("state had incorrect value: expected {expected:?}, found {found:?}")]
    StateIncorrect {
        expected: Option<String>,
        found: Option<String>,
    },

    #[error("failed to decode apu field from JWE")]
    Utf8(#[from] FromUtf8Error),

    #[error("no Document received in any DeviceResponse for credential query identifier: {0}")]
    #[category(pd)]
    NoMdocDocuments(CredentialQueryIdentifier),

    #[error("error verifying disclosed mdoc(s): {0}")]
    MdocVerification(#[from] mdoc::Error),

    #[error("error verifying disclosed SD-JWT: {0}")]
    SdJwtVerification(#[from] sd_jwt::error::DecoderError),

    #[error("error converting SD-JWT JWK: {0}")]
    SdJwtJwkConversion(#[source] jwt::error::JwkConversionError),

    #[error("response does not satisfy credential request(s): {0}")]
    UnsatisfiedCredentialRequest(#[source] CredentialValidationError),

    #[error("missing PoA")]
    MissingPoa,

    #[error("error verifying PoA: {0}")]
    PoaVerification(#[from] PoaVerificationError),

    #[error("error converting disclosed attestations: {0}")]
    #[category(pd)]
    DisclosedAttestation(#[from] DisclosedAttestationError),

    #[error("not all revocation statuses are valid")]
    #[category(expected)]
    RevocationStatusNotAllValid,
}

/// Disclosure of a credential, generally containing the issuer-signed credential itself, the disclosed attributes,
/// and a holder signature over some nonce provided by the verifier.
#[serde_as]
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum VerifiablePresentation {
    // NB: a `DeviceResponse` can contain disclosures of multiple mdocs. In case of other (not yet supported) formats,
    //     each credential is expected to result in a separate Verifiable Presentation.
    MsoMdoc(#[serde_as(as = "Vec<CborBase64>")] VecNonEmpty<DeviceResponse>),
    SdJwt(VecNonEmpty<UnverifiedSdJwtPresentation>),
}

/// Manual implementation of [`Deserialize`] for [`VerifiablePresentation`] is necessary, in order to help `serde`
/// discern between the two enum variants without attempting to do a full base64 / CBOR decode.
impl<'de> Deserialize<'de> for VerifiablePresentation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let payloads = VecNonEmpty::<String>::deserialize(deserializer)?;

        // Assume the payloads are SD-JWT if any of them contains a
        // tilde character, which does not occur in URL-safe Base64.
        let verifiable_presentation = if payloads.iter().any(|payload| payload.contains('~')) {
            let presentations = payloads
                .into_nonempty_iter()
                .map(|sd_jwt| UnverifiedSdJwtPresentation::deserialize(StringDeserializer::new(sd_jwt)))
                .collect::<Result<_, _>>()?;
            Self::SdJwt(presentations)
        } else {
            let device_responses = payloads
                .into_nonempty_iter()
                .map(|base64| CborBase64::deserialize_as(StringDeserializer::new(base64)))
                .collect::<Result<_, _>>()?;

            Self::MsoMdoc(device_responses)
        };

        Ok(verifiable_presentation)
    }
}

// We do not reuse or embed the `AuthorizationResponse` struct from `authorization.rs`, because in no variant
// of OpenID4VP that we (plan to) support do we need the `code` field from that struct, which is its primary citizen.
/// An OpenID4VP Authorization Response, with the wallet's disclosed credentials/attributes in the `vp_token`.
#[derive(Debug, Clone, Serialize, Deserialize, Constructor)]
pub struct VpAuthorizationResponse {
    /// A map of Verifiable Presentations, keyed by the credential query identifier of the original DCQL request.
    pub vp_token: HashMap<CredentialQueryIdentifier, VerifiablePresentation>,

    /// MUST equal the `state` from the Authorization Request.
    /// May be used by the RP to link incoming Authorization Responses to its corresponding Authorization Request,
    /// for example in case the `response_uri` contains no session token or other identifier.
    pub state: Option<String>,

    pub poa: Option<Poa>,
}

impl VpAuthorizationResponse {
    /// Create a JWE containing a new encrypted Authorization Request.
    pub fn new_encrypted(
        vp_token: HashMap<CredentialQueryIdentifier, VerifiablePresentation>,
        auth_request: &NormalizedVpAuthorizationRequest,
        encryption_algorithm: &JweEncryptionAlgorithm,
        encryption_nonce: &str,
        poa: Option<Poa>,
    ) -> Result<String, AuthResponseError> {
        Self::new(vp_token, auth_request.state.clone(), poa).encrypt(
            auth_request,
            encryption_algorithm,
            encryption_nonce,
        )
    }

    fn encrypt(
        &self,
        auth_request: &NormalizedVpAuthorizationRequest,
        encryption_algorithm: &JweEncryptionAlgorithm,
        encryption_nonce: &str,
    ) -> Result<String, AuthResponseError> {
        let mut header = JweHeader::new();
        header.set_token_type("JWT");

        // Set the `apv` to the nonce contained in the auth request and specified by the verifier,
        // while setting the `apu` to a random nonce generated by the holder.
        header.set_agreement_partyuinfo(encryption_nonce);
        header.set_agreement_partyvinfo(auth_request.nonce.clone());

        // Use the first content encryption algorithm that both parties support.
        header.set_content_encryption(encryption_algorithm.to_string());
        header.set_key_id(auth_request.encryption_pubkey.kid().to_string());

        // VpAuthorizationRequest always serializes to a JSON object useable as a JWT payload.
        let serde_json::Value::Object(payload) = serde_json::to_value(self)? else {
            panic!("VpAuthorizationResponse did not serialize to object")
        };
        let payload = JwtPayload::from_map(payload).unwrap();

        // Encrypt the response with the encrypter that was already created from the JWK.
        let jwe = josekit::jwt::encode_with_encrypter(&payload, &header, auth_request.encryption_pubkey.encrypter())
            .map_err(AuthResponseError::Jwe)?;

        Ok(jwe)
    }

    #[expect(clippy::too_many_arguments)]
    pub async fn decrypt_and_verify<C>(
        jwe: &str,
        private_key: &EcKeyPair,
        auth_request: &NormalizedVpAuthorizationRequest,
        accepted_wallet_client_ids: &[String],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        extending_vct_values: &impl ExtendingVctRetriever,
        revocation_verifier: &RevocationVerifier<C>,
        accept_undetermined_revocation_status: bool,
    ) -> Result<UniqueIdVec<DisclosedAttestations>, AuthResponseError>
    where
        C: StatusListClient,
    {
        let response = Self::decrypt(jwe, private_key)?;

        response
            .verify(
                auth_request,
                accepted_wallet_client_ids,
                time,
                trust_anchors,
                extending_vct_values,
                revocation_verifier,
                accept_undetermined_revocation_status,
            )
            .await
    }

    fn decrypt(jwe: &str, private_key: &EcKeyPair) -> Result<VpAuthorizationResponse, AuthResponseError> {
        let decrypter = EcdhEsJweAlgorithm::EcdhEs
            .decrypter_from_jwk(&private_key.to_jwk_key_pair())
            .expect("should be able to create EcdhEsJweDecrypter from EcKeyPair");
        let (payload, _header) =
            josekit::jwt::decode_with_decrypter(jwe, &decrypter).map_err(AuthResponseError::Jwe)?;

        let payload = serde_json::from_value(serde_json::Value::Object(payload.into()))?;

        Ok(payload)
    }

    #[expect(clippy::too_many_arguments)]
    async fn verify<C>(
        self,
        auth_request: &NormalizedVpAuthorizationRequest,
        accepted_wallet_client_ids: &[String],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        extending_vct_values: &impl ExtendingVctRetriever,
        revocation_verifier: &RevocationVerifier<C>,
        accept_undetermined_revocation_status: bool,
    ) -> Result<UniqueIdVec<DisclosedAttestations>, AuthResponseError>
    where
        C: StatusListClient,
    {
        // Step 1: Verify the cryptographic integrity of the verifiable presentations
        //         and extract the disclosed attestations from them.

        // Only one mdoc `SessionTranscript` is needed for the entire response.
        // However, it may not be required, so initialize it lazily.
        let session_transcript = LazyLock::new(|| auth_request.session_transcript());

        let mut holder_public_keys: Vec<VerifyingKey> = Vec::new();
        let mut disclosed_attestations = HashMap::new();

        for (id, presentation) in self.vp_token {
            let (attestations, public_keys) = match presentation {
                VerifiablePresentation::MsoMdoc(device_responses) => {
                    let (keys, attestations): (Vec<_>, Vec<_>) =
                        try_join_all(device_responses.iter().map(|device_response| async {
                            Self::mdoc_to_disclosed_attestation(
                                device_response,
                                &session_transcript,
                                time,
                                trust_anchors,
                                revocation_verifier,
                            )
                            .await
                        }))
                        .await?
                        .into_iter()
                        .flatten()
                        .unzip();

                    (
                        attestations
                            .try_into()
                            .map_err(|_| AuthResponseError::NoMdocDocuments(id.clone()))?,
                        keys,
                    )
                }
                VerifiablePresentation::SdJwt(sdw_jwt_payloads) => {
                    let (keys, attestations): (Vec<_>, Vec<_>) =
                        try_join_all(sdw_jwt_payloads.into_iter().map(|unverified_presentation| async {
                            Self::sd_jwt_to_disclosed_attestation(
                                unverified_presentation,
                                auth_request,
                                time,
                                trust_anchors,
                                revocation_verifier,
                            )
                            .await
                        }))
                        .await?
                        .into_iter()
                        .unzip();

                    (attestations.try_into().unwrap(), keys)
                }
            };

            // Retain the used holder public keys for checking the PoA in step 2.
            holder_public_keys.extend(public_keys);
            disclosed_attestations.insert(id.clone(), attestations);
        }

        // Step 2: Check the revocation statuses of the disclosed credentials.
        Self::evaluate_revocation_policy(
            disclosed_attestations
                .values()
                .flat_map(|attestations: &VecNonEmpty<_>| {
                    attestations.iter().map(|attestation| &attestation.revocation_status)
                }),
            accept_undetermined_revocation_status,
        )?;

        // Step 3: Verify the PoA, if present. Unfortunately `VerifyingKey` does not
        //         implement `Hash`, so we have to sort and deduplicate manually.
        holder_public_keys.sort();
        holder_public_keys.dedup();
        if holder_public_keys.len() >= 2 {
            self.poa.ok_or(AuthResponseError::MissingPoa)?.verify(
                &holder_public_keys,
                auth_request.client_id.to_string().as_str(),
                accepted_wallet_client_ids,
                &auth_request.nonce,
            )?
        }

        // Step 4: Verify the `state` field, against that of the Authorization Request.
        if self.state != auth_request.state {
            return Err(AuthResponseError::StateIncorrect {
                expected: auth_request.state.clone(),
                found: self.state.clone(),
            });
        }

        // Step 5: Check that we received all the attributes that we requested.
        auth_request
            .credential_requests
            .is_satisfied_by_disclosed_credentials(&disclosed_attestations, extending_vct_values)
            .map_err(AuthResponseError::UnsatisfiedCredentialRequest)?;

        // Step 6: Sort the disclosed attestations into the same order as that of the Credential Requests in the DCQL
        //         request and remove any attributes that were not present in the respective Credential Request. This
        //         removal is not a privacy feature, as at this point the attributes have already left the holder.
        //         However, we discard them here in order to provide a predictable API to the RP by publishing exactly
        //         those attributes it requested at the `disclosed_attributes` endpoint.
        //
        //         Note that the order of the attributes within a `DisclosedAttestation` is undefined.
        let disclosed_attestations = auth_request
            .credential_requests
            .as_ref()
            .iter()
            .map(|credential_request| {
                // Safety: in step 4 we checked that for each `credential_request`
                //         there is a matching disclosed attestation.
                let (id, mut attestations) = disclosed_attestations.remove_entry(credential_request.id()).unwrap();

                for attestation in attestations.iter_mut() {
                    attestation.attributes.prune(credential_request.claim_paths());
                }

                DisclosedAttestations { id, attestations }
            })
            .collect_vec();

        // Safety: this comes from mapping over auth_request.credential_requests, which is
        // a newtype around a `UniqueIdVec`.
        let disclosed_attestations = UniqueIdVec::try_from(disclosed_attestations).unwrap();

        Ok(disclosed_attestations)
    }

    async fn mdoc_to_disclosed_attestation<C>(
        device_response: &DeviceResponse,
        session_transcript: &SessionTranscript,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        revocation_verifier: &RevocationVerifier<C>,
    ) -> Result<Vec<(VerifyingKey, DisclosedAttestation)>, AuthResponseError>
    where
        C: StatusListClient,
    {
        // Verify the cryptographic integrity of each mdoc `DeviceResponse`
        // and obtain a `DisclosedDocuments` for each.
        let disclosed_documents = device_response
            .verify(None, session_transcript, time, trust_anchors, revocation_verifier)
            .await?;

        // Then attempt to convert the disclosed documents to `DisclosedAttestation`s.
        let disclosed_attestations = disclosed_documents
            .into_iter()
            .map(|disclosed_document| {
                Ok::<_, AuthResponseError>((
                    disclosed_document.device_key,
                    DisclosedAttestation::try_from(disclosed_document)
                        .map_err(AuthResponseError::DisclosedAttestation)?,
                ))
            })
            .try_collect()?;

        Ok(disclosed_attestations)
    }

    async fn sd_jwt_to_disclosed_attestation<C>(
        unverified_presentation: UnverifiedSdJwtPresentation,
        auth_request: &NormalizedVpAuthorizationRequest,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        revocation_verifier: &RevocationVerifier<C>,
    ) -> Result<(VerifyingKey, DisclosedAttestation), AuthResponseError>
    where
        C: StatusListClient,
    {
        let kb_verification_options = KbVerificationOptions {
            expected_aud: &auth_request.client_id.to_string(),
            expected_nonce: &auth_request.nonce,
            iat_leeway: SD_JWT_IAT_LEEWAY,
            iat_acceptance_window: SD_JWT_IAT_WINDOW,
        };

        let presentation = unverified_presentation
            .into_verified_against_trust_anchors(trust_anchors, &kb_verification_options, time, revocation_verifier)
            .await?;

        let holder_public_key = presentation
            .sd_jwt()
            .holder_pubkey()
            .map_err(AuthResponseError::SdJwtJwkConversion)?;

        let disclosed_attestation = DisclosedAttestation::try_from(presentation)?;

        Ok((holder_public_key, disclosed_attestation))
    }

    fn evaluate_revocation_policy<'a>(
        statuses: impl Iterator<Item = &'a Option<RevocationStatus>>,
        accept_undetermined_revocation_status: bool,
    ) -> Result<(), AuthResponseError> {
        if !statuses.into_iter().all(|status| {
            status.is_some_and(|status| {
                status == RevocationStatus::Valid
                    || (status == RevocationStatus::Undetermined && accept_undetermined_revocation_status)
            })
        }) {
            return Err(AuthResponseError::RevocationStatusNotAllValid);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpResponse {
    pub redirect_uri: Option<BaseUrl>,
}

#[cfg(any(test, feature = "test"))]
pub mod test {
    use super::*;

    impl NormalizedVpAuthorizationRequest {
        pub fn new_from_certificate(
            credential_requests: NormalizedCredentialRequests,
            rp_certificate: &BorrowingCertificate,
            nonce: String,
            encryption_pubkey: JwePublicKey,
            response_uri: BaseUrl,
            wallet_nonce: Option<String>,
        ) -> Self {
            let client_id = ClientId::x509_san_dns(
                rp_certificate
                    .san_dns_name()
                    .expect("certificate SAN DNSName should be parseable")
                    .expect("certificate should contain SAN DNSName"),
            );

            Self::new_for_verifier(
                credential_requests,
                client_id,
                nonce,
                encryption_pubkey,
                response_uri,
                wallet_nonce,
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use base64::prelude::*;
    use futures::FutureExt;
    use itertools::Itertools;
    use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
    use josekit::jwk::Jwk;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use josekit::jwk::alg::ed::EdCurve;
    use rstest::rstest;
    use rustls_pki_types::TrustAnchor;
    use serde::Deserialize;
    use serde_json::json;
    use serde_with::serde_as;

    use attestation_data::attributes::AttributesTraversalBehaviour;
    use attestation_data::disclosure::DisclosedAttributes;
    use attestation_data::test_credential::nl_pid_address_minimal_address;
    use attestation_data::test_credential::nl_pid_credentials_full_name;

    use attestation_types::claim_path::ClaimPath;
    use attestation_types::pid_constants::PID_ATTESTATION_TYPE;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use crypto::server_keys::generate::mock::ISSUANCE_CERT_CN;
    use crypto::server_keys::generate::mock::PID_ISSUER_CERT_CN;
    use crypto::x509::CertificateUsage;
    use dcql::CredentialFormat;
    use dcql::CredentialQueryIdentifier;
    use dcql::normalized::NormalizedCredentialRequest;
    use dcql::normalized::NormalizedCredentialRequests;
    use http_utils::urls::BaseUrl;
    use jwt::SignedJwt;
    use jwt::pop::JwtPopClaims;
    use mdoc::DeviceResponse;
    use mdoc::examples::Example;
    use mdoc::holder::Mdoc;
    use mdoc::holder::disclosure::PartialMdoc;
    use sd_jwt::builder::SignedSdJwt;
    use sd_jwt::examples::WITH_KB_SD_JWT;
    use sd_jwt::key_binding_jwt::KeyBindingJwtBuilder;
    use sd_jwt::sd_jwt::UnsignedSdJwtPresentation;
    use token_status_list::verification::client::mock::StatusListClientStub;
    use token_status_list::verification::verifier::RevocationStatus;
    use token_status_list::verification::verifier::RevocationVerifier;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_at_least::VecNonEmpty;
    use utils::vec_nonempty;
    use wscd::Poa;
    use wscd::mock_remote::MockRemoteWscd;
    use wscd::wscd::JwtPoaInput;

    use crate::AuthorizationErrorCode;
    use crate::VpAuthorizationErrorCode;
    use crate::jwe::JweEncryptionAlgorithm;
    use crate::mock::ExtendingVctRetrieverStub;
    use crate::mock::MOCK_WALLET_CLIENT_ID;

    use super::AuthRequestValidationError;
    use super::AuthResponseError;
    use super::ClientId;
    use super::ClientIdScheme;
    use super::JsonBase64;
    use super::JwePublicKey;
    use super::JwePublicKeyError;
    use super::NormalizedVpAuthorizationRequest;
    use super::VerifiablePresentation;
    use super::VpAuthorizationRequest;
    use super::VpAuthorizationResponse;
    use super::VpRequestUri;
    use super::VpRequestUriObject;

    #[serde_as]
    #[derive(Debug, Deserialize)]
    #[expect(dead_code)]
    struct TransactionDataEntries(
        #[serde_as(as = "Vec<JsonBase64>")] VecNonEmpty<serde_json::Map<String, serde_json::Value>>,
    );

    enum ExpectedJwePublicKeyError {
        UnsupportedJwk { field: &'static str },
        JwkParsing,
    }

    #[rstest]
    #[case::ed(
        Jwk::generate_ed_key(EdCurve::Ed25519).unwrap(),
        ExpectedJwePublicKeyError::UnsupportedJwk { field: "kty" }
    )]
    #[case::p384(
        Jwk::generate_ec_key(EcCurve::P384).unwrap(),
        ExpectedJwePublicKeyError::UnsupportedJwk { field: "crv" }
    )]
    #[case::coordinates(
        serde_json::from_value(json!({"kty":"EC","crv":"P-256","alg":"ECDH-ES","kid":"test"})).unwrap(),
        ExpectedJwePublicKeyError::JwkParsing
    )]
    #[case::wrong_alg(
        serde_json::from_value(json!({
            "kty":"EC",
            "crv":"P-256",
            "alg":"ES256",
            "kid":"test",
            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
        }))
        .unwrap(),
        ExpectedJwePublicKeyError::JwkParsing
    )]
    #[case::non_string_crv(
        serde_json::from_value(json!({
            "kty":"EC",
            "crv": true,
            "alg":"ECDH-ES",
            "kid":"test"
        }))
        .unwrap(),
        ExpectedJwePublicKeyError::UnsupportedJwk { field: "crv" }
    )]
    #[case::missing_kid(
        serde_json::from_value(json!({
            "kty":"EC",
            "crv":"P-256",
            "alg":"ECDH-ES",
            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
        }))
        .unwrap(),
        ExpectedJwePublicKeyError::UnsupportedJwk { field: "kid" }
    )]
    #[case::non_string_kid(
        serde_json::from_value(json!({
            "kty":"EC",
            "crv":"P-256",
            "alg":"ECDH-ES",
            "kid": 1,
            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
        }))
        .unwrap(),
        ExpectedJwePublicKeyError::UnsupportedJwk { field: "kid" }
    )]
    fn test_jwe_public_key_validation(#[case] jwk: Jwk, #[case] expected_error: ExpectedJwePublicKeyError) {
        let error = JwePublicKey::try_new(jwk).expect_err("JwePublicKey validation should fail");

        match expected_error {
            ExpectedJwePublicKeyError::UnsupportedJwk { field: expected_field } => {
                assert_matches!(error, JwePublicKeyError::UnsupportedJwk { field, .. } if field == expected_field);
            }
            ExpectedJwePublicKeyError::JwkParsing => {
                assert_matches!(error, JwePublicKeyError::JwkParsing(_));
            }
        }
    }

    #[test]
    fn test_jwe_public_key_sha256_thumbprint_bytes() {
        // Source (edited): https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#appendix-B.2.6.1-7
        let jwk = JwePublicKey::try_new(
            serde_json::from_value(json!({
                "kty": "EC",
                "crv": "P-256",
                "alg": "ECDH-ES",
                "kid": "test",
                "x": "DxiH5Q4Yx3UrukE2lWCErq8N8bqC9CHLLrAwLz5BmE0",
                "y": "XtLM4-3h5o3HUH0MHVJV0kyq0iBlrBwlh8qEDMZ4-Pc"
            }))
            .unwrap(),
        )
        .unwrap();

        // Source: https://openid.net/specs/openid-4-verifiable-presentations-1_0.html#appendix-B.2.6.1-9
        let expected_thumbprint =
            hex::decode("4283ec927ae0f208daaa2d026a814f2b22dca52cf85ffa8f3f8626c6bd669047").unwrap();

        assert_eq!(jwk.sha256_thumbprint_bytes(), expected_thumbprint);
    }

    #[test]
    fn test_vp_authorization_error_code_serialization() {
        assert_eq!(
            serde_json::from_str::<VpAuthorizationErrorCode>(r#""invalid_request""#).unwrap(),
            VpAuthorizationErrorCode::AuthorizationError(AuthorizationErrorCode::InvalidRequest)
        );

        assert_eq!(
            serde_json::from_str::<VpAuthorizationErrorCode>(r#""vp_formats_not_supported""#).unwrap(),
            VpAuthorizationErrorCode::VpFormatsNotSupported,
        );
    }

    #[test]
    fn test_client_id_parse_and_display_x509_san_dns() {
        let client_id: ClientId = "x509_san_dns:example.com".into();

        assert_eq!(client_id.to_string(), "x509_san_dns:example.com");
        assert_matches!(client_id.scheme, Some(ClientIdScheme::X509SanDns));
        assert_eq!(client_id.id, "example.com");
    }

    #[test]
    fn test_client_id_parse_and_display_without_scheme() {
        let client_id: ClientId = "example.com".into();

        assert_eq!(client_id.to_string(), "example.com");
        assert_matches!(client_id.scheme, None);
        assert_eq!(client_id.id, "example.com");
    }

    #[test]
    fn test_client_id_parse_and_display_unknown_scheme() {
        let client_id: ClientId = "future_scheme:example.com".into();

        assert_eq!(client_id.to_string(), "future_scheme:example.com");
        assert_matches!(&client_id.scheme, Some(ClientIdScheme::Other(s)) if s == "future_scheme");
        assert_eq!(client_id.id, "example.com");
    }

    #[test]
    fn test_client_id_parse_and_display_decentralized_identifier() {
        let client_id: ClientId = "decentralized_identifier:did:example:123".into();

        assert_eq!(client_id.to_string(), "decentralized_identifier:did:example:123");
        assert_matches!(client_id.scheme, Some(ClientIdScheme::DecentralizedIdentifier));
        assert_eq!(client_id.id, "did:example:123");
    }

    #[test]
    fn test_client_id_parse_and_display_x509_hash() {
        let client_id: ClientId = "x509_hash:abcdef".into();

        assert_eq!(client_id.to_string(), "x509_hash:abcdef");
        assert_matches!(client_id.scheme, Some(ClientIdScheme::X509Hash));
        assert_eq!(client_id.id, "abcdef");
    }

    fn setup_mdoc() -> (
        TrustAnchor<'static>,
        KeyPair,
        EcKeyPair,
        NormalizedVpAuthorizationRequest,
    ) {
        setup_with_credential_requests(NormalizedCredentialRequests::new_mock_mdoc_iso_example())
    }

    fn setup_with_credential_requests(
        credential_requests: NormalizedCredentialRequests,
    ) -> (
        TrustAnchor<'static>,
        KeyPair,
        EcKeyPair,
        NormalizedVpAuthorizationRequest,
    ) {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let trust_anchor = ca.to_trust_anchor().to_owned();
        let rp_keypair = ca.generate_reader_mock().unwrap();

        let encryption_privkey = EcKeyPair::generate(EcCurve::P256).unwrap();
        let mut encryption_jwk = encryption_privkey.to_jwk_public_key();
        encryption_jwk.set_algorithm("ECDH-ES");
        encryption_jwk.set_key_id("test-kid");
        let rp_fqdn = rp_keypair.certificate().san_dns_name().unwrap().unwrap();
        let response_uri = format!("https://{rp_fqdn}/response_uri").parse().unwrap();

        let auth_request = NormalizedVpAuthorizationRequest::new_from_certificate(
            credential_requests,
            rp_keypair.certificate(),
            "nonce".to_string(),
            encryption_jwk.try_into().unwrap(),
            response_uri,
            None,
        );

        (trust_anchor, rp_keypair, encryption_privkey, auth_request)
    }

    #[test]
    fn test_encrypt_decrypt_authorization_response() {
        let (_, _, encryption_privkey, auth_request) = setup_mdoc();

        // NB: the example DeviceResponse verifies as an ISO 18013-5 DeviceResponse while here we use it in
        // an OpenID4VP setting, i.e. with different SessionTranscript contents, so it can't be verified.
        // This is not an issue here because this code only deals with en/decrypting the DeviceResponse into
        // an Authorization Response JWE.
        let encryption_nonce = "encryption_nonce".to_string();
        let encrypted_device_response = DeviceResponse::example();
        // the ISO examples only use one Mdoc and therefore there is no need for a PoA
        let auth_response = VpAuthorizationResponse::new(
            HashMap::from([(
                "id".try_into().unwrap(),
                VerifiablePresentation::MsoMdoc(vec_nonempty![encrypted_device_response.clone()]),
            )]),
            None,
            None,
        );
        let encryption_algorithm = JweEncryptionAlgorithm::default();
        let jwe = auth_response
            .encrypt(&auth_request, &encryption_algorithm, &encryption_nonce)
            .unwrap();
        let decrypter = EcdhEsJweAlgorithm::EcdhEs
            .decrypter_from_jwk(&encryption_privkey.to_jwk_key_pair())
            .unwrap();
        let (_, header) = josekit::jwt::decode_with_decrypter(&jwe, &decrypter).unwrap();
        assert_eq!(header.key_id(), Some(auth_request.encryption_pubkey.kid()));

        let decrypted = VpAuthorizationResponse::decrypt(&jwe, &encryption_privkey).unwrap();

        assert_eq!(decrypted.vp_token.len(), 1);

        let (decrypted_identifier, VerifiablePresentation::MsoMdoc(decrypted_device_responses)) =
            decrypted.vp_token.into_iter().next().unwrap()
        else {
            panic!("decrypted format should be mdoc")
        };

        assert_eq!(decrypted_identifier.as_ref(), "id");
        assert_eq!(decrypted_device_responses.len().get(), 1);

        let decrypted_device_response = decrypted_device_responses.into_first();

        assert_eq!(
            decrypted_device_response
                .documents
                .as_ref()
                .map(|documents| documents.len())
                .unwrap_or_default(),
            1
        );

        let encrypted_document = encrypted_device_response.documents.unwrap().into_iter().next().unwrap();
        let decrypted_document = decrypted_device_response.documents.unwrap().into_iter().next().unwrap();

        assert_eq!(decrypted_document.doc_type, encrypted_document.doc_type);
        assert_eq!(decrypted_document.issuer_signed, encrypted_document.issuer_signed);
    }

    #[test]
    fn test_authorization_request_jwt() {
        let (trust_anchor, rp_keypair, _, auth_request) = setup_mdoc();

        let auth_request_jwt =
            SignedJwt::sign_with_certificate(&VpAuthorizationRequest::from(auth_request), &rp_keypair)
                .now_or_never()
                .unwrap()
                .unwrap();

        let (auth_request, cert) = VpAuthorizationRequest::try_new(&auth_request_jwt.into(), &[trust_anchor]).unwrap();
        let _ = auth_request.validate(&cert, None).unwrap();
    }

    #[test]
    fn test_authorization_request_validate_unauthorized_client_id() {
        let (_, rp_keypair, _, auth_request) = setup_mdoc();
        let mut auth_request = VpAuthorizationRequest::from(auth_request);
        let wrong_dns_san = "wrong.example.com";
        let cert_dns_san = rp_keypair.certificate().san_dns_name().unwrap().unwrap();

        auth_request.oauth_request.client_id = format!("x509_san_dns:{wrong_dns_san}");
        auth_request.response_uri = Some(format!("https://{wrong_dns_san}/response_uri").parse().unwrap());

        let err = auth_request.validate(rp_keypair.certificate(), None).unwrap_err();
        assert_matches!(
            err,
            AuthRequestValidationError::UnauthorizedClientId { client_id, dns_san }
            if client_id == format!("x509_san_dns:{wrong_dns_san}") && dns_san.contains(cert_dns_san)
        );
    }

    #[test]
    fn test_authorization_request_validate_unsupported_client_id_scheme() {
        let (_, rp_keypair, _, auth_request) = setup_mdoc();
        let mut auth_request = VpAuthorizationRequest::from(auth_request);
        let dns_san = rp_keypair.certificate().san_dns_name().unwrap().unwrap();
        auth_request.oauth_request.client_id = format!("redirect_uri:{dns_san}");

        let err = auth_request.validate(rp_keypair.certificate(), None).unwrap_err();
        assert_matches!(
            err,
            AuthRequestValidationError::UnsupportedClientIdScheme {
                scheme: ClientIdScheme::RedirectUri
            }
        );
    }

    #[test]
    fn test_authorization_request_validate_unsupported_client_id_without_scheme() {
        let (_, rp_keypair, _, auth_request) = setup_mdoc();
        let mut auth_request = VpAuthorizationRequest::from(auth_request);
        let dns_san = rp_keypair.certificate().san_dns_name().unwrap().unwrap();
        auth_request.oauth_request.client_id = dns_san.to_string();

        let err = auth_request.validate(rp_keypair.certificate(), None).unwrap_err();
        assert_matches!(err, AuthRequestValidationError::UnsupportedClientIdWithoutScheme);
    }

    #[test]
    fn test_authorization_request_validate_unmatched_response_fqdn() {
        let (_, rp_keypair, _, auth_request) = setup_mdoc();
        let mut auth_request = VpAuthorizationRequest::from(auth_request);
        let wrong_fqdn = "wrong.example.com";
        let expected_client_id = rp_keypair.certificate().san_dns_name().unwrap().unwrap().to_string();

        auth_request.response_uri = Some(format!("https://{wrong_fqdn}/response_uri").parse().unwrap());

        let err = auth_request.validate(rp_keypair.certificate(), None).unwrap_err();
        assert_matches!(
            err,
            AuthRequestValidationError::UnmatchedResponseFqdn { fqdn, id }
            if fqdn == wrong_fqdn && id == expected_client_id
        );
    }

    #[test]
    fn test_authorization_request_validate_unmatched_response_fqdn_without_fqdn() {
        let (_, rp_keypair, _, auth_request) = setup_mdoc();
        let mut auth_request = VpAuthorizationRequest::from(auth_request);
        let response_uri: BaseUrl = "file:///response_uri".parse().unwrap();
        let expected_response_uri = response_uri.to_string();
        let expected_client_id = rp_keypair.certificate().san_dns_name().unwrap().unwrap().to_string();

        auth_request.response_uri = Some(response_uri);

        let err = auth_request.validate(rp_keypair.certificate(), None).unwrap_err();
        assert_matches!(
            err,
            AuthRequestValidationError::UnmatchedResponseFqdn { fqdn, id }
            if fqdn == expected_response_uri && id == expected_client_id
        );
    }

    #[test]
    fn deserialize_authorization_request_example() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "my-key-id",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false },
                            { "path": ["org.iso.18013.5.1", "birth_date"], "intent_to_retain": false },
                            { "path": ["org.iso.18013.5.1", "document_number"], "intent_to_retain": false },
                            { "path": ["org.iso.18013.5.1", "driving_privileges"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap();
    }

    #[test]
    fn deserialize_authorization_request_single_encryption_enc_supported() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "my-key-id",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let (normalized_request, _) =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap();

        assert_eq!(
            normalized_request
                .client_metadata
                .encrypted_response_enc_values_supported,
            Some(vec_nonempty![JweEncryptionAlgorithm::A256Gcm])
        );
    }

    #[test]
    fn deserialize_authorization_request_default_encryption_enc_supported() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "my-key-id",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let (normalized_request, _) =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap();

        assert!(
            normalized_request
                .client_metadata
                .encrypted_response_enc_values_supported
                .is_none()
        );
    }

    #[test]
    fn select_encryption_algorithm_should_default_to_a128gcm_when_not_advertised() {
        let (_, _, _, auth_request) = setup_mdoc();
        let mut client_metadata = auth_request.client_metadata.clone();
        client_metadata.encrypted_response_enc_values_supported = None;

        let encryption_algorithm =
            NormalizedVpAuthorizationRequest::select_encryption_algorithm(&client_metadata).unwrap();

        assert_eq!(encryption_algorithm, JweEncryptionAlgorithm::A128Gcm);
    }

    #[test]
    fn select_encryption_algorithm_should_ignore_unknown_algorithms() {
        let (_, _, _, auth_request) = setup_mdoc();
        let mut client_metadata = auth_request.client_metadata.clone();
        client_metadata.encrypted_response_enc_values_supported = Some(vec_nonempty![
            JweEncryptionAlgorithm::Other("A512GCM".to_string()),
            JweEncryptionAlgorithm::A256Gcm
        ]);

        let encryption_algorithm =
            NormalizedVpAuthorizationRequest::select_encryption_algorithm(&client_metadata).unwrap();

        assert_eq!(encryption_algorithm, JweEncryptionAlgorithm::A256Gcm);
    }

    #[test]
    fn select_encryption_algorithm_should_prefer_a256gcm_over_a128gcm() {
        let (_, _, _, auth_request) = setup_mdoc();
        let mut client_metadata = auth_request.client_metadata.clone();
        client_metadata.encrypted_response_enc_values_supported = Some(vec_nonempty![
            JweEncryptionAlgorithm::A128Gcm,
            JweEncryptionAlgorithm::A256Gcm
        ]);

        let encryption_algorithm =
            NormalizedVpAuthorizationRequest::select_encryption_algorithm(&client_metadata).unwrap();

        assert_eq!(encryption_algorithm, JweEncryptionAlgorithm::A256Gcm);
    }

    #[test]
    fn validate_should_return_selected_encryption_algorithm() {
        let (_, rp_keypair, _, auth_request) = setup_mdoc();
        let mut auth_request = VpAuthorizationRequest::from(auth_request);
        auth_request
            .client_metadata
            .as_mut()
            .unwrap()
            .encrypted_response_enc_values_supported = Some(vec_nonempty![
            JweEncryptionAlgorithm::Other("A512GCM".to_string()),
            JweEncryptionAlgorithm::A256Gcm
        ]);

        let (_, encryption_algorithm) = auth_request.validate(rp_keypair.certificate(), None).unwrap();

        assert_eq!(encryption_algorithm, JweEncryptionAlgorithm::A256Gcm);
    }

    #[test]
    fn deserialize_authorization_request_unknown_and_supported_encryption_enc_supported() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "my-key-id",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A512GCM", "A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let (normalized_request, _) =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap();

        assert_eq!(
            normalized_request
                .client_metadata
                .encrypted_response_enc_values_supported,
            Some(vec_nonempty![
                JweEncryptionAlgorithm::Other("A512GCM".to_string()),
                JweEncryptionAlgorithm::A256Gcm
            ])
        );
    }

    #[test]
    fn authorization_request_without_supported_encryption_enc_should_error() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "my-key-id",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A512GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let error =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap_err();
        assert_matches!(error, AuthRequestValidationError::NoSupportedEncryptedResponseEnc);
    }

    #[test]
    fn authorization_request_missing_kid_should_error() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let error =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap_err();
        assert_matches!(error, AuthRequestValidationError::MissingJwkKid(0));
    }

    #[test]
    fn authorization_request_non_string_kid_should_error() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": 1,
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let error =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap_err();
        assert_matches!(error, AuthRequestValidationError::MissingJwkKid(0));
    }

    #[test]
    fn authorization_request_should_select_first_supported_key() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [
                            {
                                "kty": "EC", "use": "enc", "crv": "P-384", "alg": "ECDH-ES", "kid": "unsupported"
                            },
                            {
                                "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "supported",
                                "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                                "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                            }
                        ]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let (normalized_request, _) =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap();

        assert_eq!(normalized_request.encryption_pubkey.kid(), "supported");
    }

    #[test]
    fn authorization_request_without_supported_keys_should_collect_jwk_errors() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [
                            {
                                "kty": "EC", "use": "enc", "crv": "P-384", "alg": "ECDH-ES", "kid": "unsupported-curve"
                            },
                            {
                                "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ES256", "kid": "unsupported-alg",
                                "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                                "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                            }
                        ]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let error =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap_err();
        let AuthRequestValidationError::NoSupportedJwk(errors) = error else {
            panic!("expected NoSupportedJwk error");
        };

        assert_eq!(errors.len(), 2);
        assert_matches!(&errors[0], JwePublicKeyError::UnsupportedJwk { field, .. } if field == &"crv");
        assert_matches!(&errors[1], JwePublicKeyError::JwkParsing(_));
    }

    #[test]
    fn authorization_request_empty_jwks_should_fail_to_deserialize() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": []
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        let error = serde_json::from_value::<VpAuthorizationRequest>(example_json).unwrap_err();
        assert!(error.to_string().contains("vector does not contain least 1 item"));
    }

    #[test]
    fn authorization_request_with_jwks_uri_should_fail_to_deserialize() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks_uri": "https://example.com/jwks.json",
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false }
                        ]
                    }]
                }
            }
        );

        assert!(serde_json::from_value::<VpAuthorizationRequest>(example_json).is_err());
    }

    #[test]
    fn authorization_request_with_transaction_data_should_error() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id": "x509_san_dns:example.com",
                "response_uri": "https://example.com/post",
                "nonce": "%%2_fsd32434!==r",
                "client_metadata": {
                    "jwks": {
                        "keys": [{
                            "kty": "EC", "use": "enc", "crv": "P-256", "alg": "ECDH-ES", "kid": "my-key-id",
                            "x": "xVLtZaPPK-xvruh1fEClNVTR6RCZBsQai2-DrnyKkxg",
                            "y": "-5-QtFqJqGwOjEL3Ut89nrE0MeaUp5RozksKHpBiyw0"
                        }]
                    },
                    "encrypted_response_enc_values_supported": ["A256GCM"],
                    "vp_formats": {
                        "mso_mdoc": {
                            "alg": [ "ES256" ]
                        }
                    }
                },
                "dcql_query": {
                    "credentials": [{
                        "id": "pid",
                        "format": "mso_mdoc",
                        "meta": {
                            "doctype_value": "org.iso.18013.5.1.mDL",
                        },
                        "claims": [
                            { "path": ["org.iso.18013.5.1", "family_name"], "intent_to_retain": false },
                        ]
                    }]
                },
                "transaction_data": ["eyJ0eXBlIjoiZXhhbXBsZSJ9"]
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        let error =
            NormalizedVpAuthorizationRequest::try_from_with_selected_encryption_algorithm(auth_request).unwrap_err();
        assert_matches!(error, AuthRequestValidationError::UnsupportedField("transaction_data"));
    }

    #[test]
    fn transaction_data_should_be_non_empty() {
        let error = serde_json::from_value::<TransactionDataEntries>(json!([])).unwrap_err();
        assert!(error.to_string().contains("vector does not contain least 1 item"));
    }

    #[test]
    fn transaction_data_entries_should_be_base64url_encoded_json_objects() {
        let invalid_base64_error = serde_json::from_value::<TransactionDataEntries>(json!(["not-base64"])).unwrap_err();
        assert!(
            invalid_base64_error
                .to_string()
                .contains("error decoding entry as base64")
        );

        let encoded_json_array = BASE64_URL_SAFE_NO_PAD.encode(br#"["not-an-object"]"#);
        let json_array_error =
            serde_json::from_value::<TransactionDataEntries>(json!([encoded_json_array])).unwrap_err();
        assert!(json_array_error.to_string().contains("error parsing entry as JSON"));
    }

    #[test]
    fn deserialize_authorization_response_mdoc_example() {
        let example_json = json!(
            {
                "vp_token": {
                    "example_credential_id": [
                        "o2d2ZXJzaW9uYzEuMGlkb2N1bWVudHOBo2dkb2NUeXBldW9yZy5pc28uMTgwMTMuNS4xLm1ETGxpc3N1\
                         ZXJTaWduZWSiam5hbWVTcGFjZXOhcW9yZy5pc28uMTgwMTMuNS4xi9gYWF-kaGRpZ2VzdElEGhU-n8Jm\
                         cmFuZG9tUBhhBdaBj6yzbcAptxJFt5NxZWxlbWVudElkZW50aWZpZXJqYmlydGhfZGF0ZWxlbGVtZW50\
                         VmFsdWXZA-xqMTk5MC0wMS0wMdgYWF-kaGRpZ2VzdElEGgGfQ2JmcmFuZG9tUD_vjxEDDiHVNPYQrc-z\
                         3qJxZWxlbWVudElkZW50aWZpZXJvZG9jdW1lbnRfbnVtYmVybGVsZW1lbnRWYWx1ZWhBQkNEMTIzNNgY\
                         WPOkaGRpZ2VzdElEGhYhPvdmcmFuZG9tUPeQCdM61nPIh-T2KdDLzJ9xZWxlbWVudElkZW50aWZpZXJy\
                         ZHJpdmluZ19wcml2aWxlZ2VzbGVsZW1lbnRWYWx1ZYKjamlzc3VlX2RhdGXZA-xqMjAyMC0wMS0wMWtl\
                         eHBpcnlfZGF0ZdkD7GoyMDI1LTAxLTAxdXZlaGljbGVfY2F0ZWdvcnlfY29kZWFCo2ppc3N1ZV9kYXRl\
                         2QPsajIwMjAtMDEtMDFrZXhwaXJ5X2RhdGXZA-xqMjAyNS0wMS0wMXV2ZWhpY2xlX2NhdGVnb3J5X2Nv\
                         ZGViQkXYGFhgpGhkaWdlc3RJRBo23jMjZnJhbmRvbVBRkUqBtZ0-cdgL-Ah55BRHcWVsZW1lbnRJZGVu\
                         dGlmaWVya2V4cGlyeV9kYXRlbGVsZW1lbnRWYWx1ZdkD7GoyMDI1LTAxLTAx2BhYWKRoZGlnZXN0SUQa\
                         ZYFFSmZyYW5kb21QdKpwyVh1BG0egitavv8UWXFlbGVtZW50SWRlbnRpZmllcmtmYW1pbHlfbmFtZWxl\
                         bGVtZW50VmFsdWVlU21pdGjYGFhXpGhkaWdlc3RJRBoX9SvMZnJhbmRvbVBD8vu88PnK3lzRO9sRvnND\
                         cWVsZW1lbnRJZGVudGlmaWVyamdpdmVuX25hbWVsZWxlbWVudFZhbHVlZUFsaWNl2BhYX6RoZGlnZXN0\
                         SUQaMaFJlmZyYW5kb21Q9AoSQ1BmYmKEqfADoeKDunFlbGVtZW50SWRlbnRpZmllcmppc3N1ZV9kYXRl\
                         bGVsZW1lbnRWYWx1ZdkD7GoyMDIwLTAxLTAx2BhYX6RoZGlnZXN0SUQaA8azMWZyYW5kb21Qb5Fu5qMe\
                         qndj9esMYWzh5XFlbGVtZW50SWRlbnRpZmllcnFpc3N1aW5nX2F1dGhvcml0eWxlbGVtZW50VmFsdWVm\
                         TlksVVNB2BhYWaRoZGlnZXN0SUQaUUgWkmZyYW5kb21Qgh02uXoPCuF2NCY9MlUucHFlbGVtZW50SWRl\
                         bnRpZmllcm9pc3N1aW5nX2NvdW50cnlsZWxlbWVudFZhbHVlYlVT2BhZCD-kaGRpZ2VzdElEGmTXNGdm\
                         cmFuZG9tUE2OWXxsntQn-CrtHF_AfwVxZWxlbWVudElkZW50aWZpZXJocG9ydHJhaXRsZWxlbWVudFZh\
                         bHVlWQft_9j_4AAQSkZJRgABAQAAAAAAAAD_4gIoSUNDX1BST0ZJTEUAAQEAAAIYAAAAAAQwAABtbnRy\
                         UkdCIFhZWiAAAAAAAAAAAAAAAABhY3NwAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAA9tYAAQAA\
                         AADTLQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAlkZXNj\
                         AAAA8AAAAHRyWFlaAAABZAAAABRnWFlaAAABeAAAABRiWFlaAAABjAAAABRyVFJDAAABoAAAAChnVFJD\
                         AAABoAAAAChiVFJDAAABoAAAACh3dHB0AAAByAAAABRjcHJ0AAAB3AAAADxtbHVjAAAAAAAAAAEAAAAM\
                         ZW5VUwAAAFgAAAAcAHMAUgBHAEIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                         AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAFhZWiAAAAAAAABvogAAOPUAAAOQ\
                         WFlaIAAAAAAAAGKZAAC3hQAAGNpYWVogAAAAAAAAJKAAAA-EAAC2z3BhcmEAAAAAAAQAAAACZmYAAPKn\
                         AAANWQAAE9AAAApbAAAAAAAAAABYWVogAAAAAAAA9tYAAQAAAADTLW1sdWMAAAAAAAAAAQAAAAxlblVT\
                         AAAAIAAAABwARwBvAG8AZwBsAGUAIABJAG4AYwAuACAAMgAwADEANv_bAEMAEAsMDgwKEA4NDhIREBMY\
                         KBoYFhYYMSMlHSg6Mz08OTM4N0BIXE5ARFdFNzhQbVFXX2JnaGc-TXF5cGR4XGVnY__bAEMBERISGBUY\
                         LxoaL2NCOEJjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY__A\
                         ABEIALAAeQMBIgACEQEDEQH_xAAaAAADAQEBAQAAAAAAAAAAAAAAAwQFBgcB_8QALhAAAgIBAwIEBQMF\
                         AAAAAAAAAAMEEyMFFDNDUyRjc4MBBhU0oxZEkyU1UVWz_8QAFgEBAQEAAAAAAAAAAAAAAAAAAAME_8QA\
                         FhEBAQEAAAAAAAAAAAAAAAAAAAMT_9oADAMBAAIRAxEAPwDlwACSRoAAAAW1CrQGgS7pR93q_wDAFIEu\
                         6C0CoAAAAAAAAAFDRQ0AFNGksoBVtrRTQAqqAUFQAN9VoWh6oW-UA1TRvSJbQbykheptoCopUEigAAAA\
                         AAFSmq4gktqUSxVWtAbFgNlNOji_LihsBVRsqDVkxm_LiukQfpyUdkNCuTklfK-LK0y5WjNitqaegks9\
                         VqqgZODlaW2Ll6QrpG9PVa2pXS5TLnq2srymhJBxFRK1RUriCQAACQAAAglcpfpaiCVympACsnRxVGoo\
                         gil6g1GjQAkC2ogqbP8AKUX1Daiog2ClKqUo5zVIFVp2Rl6pFtUBwbcQKGz_ALpoqLyhlNAACQAAAllc\
                         pfFbUQSuUaFZOoiz1VF6p6u6ckpqhtVqrVBXR2SpQ23KcbAa1rVKOtqtihVVaNtOXlNlKxWios9tuWUB\
                         1pK3KSqa1vV9oqy9UDz7VMUpqhUXiKte_ukr1RSsSgy1AAASAChoCmqL1RbVClelaakVVTagrJLFgeUb\
                         KqlRaqhtQqVxBqZass9SlNqOoqaqL9-33VHL6Mrx9p2XKqoCDaqbF81uUg-lqt6vpGzAVVFUoaBlxYFT\
                         cXEXtt5VKyjahTW1NVUB59PU36o23ujRs_K22olDLUAKAJFDRQ0Bqi-LKqaQDVBWTe3TcWK0lntaNi8Q\
                         qVFtDUboLVW1HRnJRYErdYjqNq1qlZcoBbU1o1spSuVtXqjasQVEgq23iyg1XLbytKiCfPVAytbiKjkt\
                         Zbbqjf4iAbKlKbKa3utFBgKAAAUNFAA0aKADeit8KNVKtMuA3ul9SmhqbOl1Wl_dOXixVW27pqjZVU39\
                         1b7oVag3lJVKq4mjbSQGnJfNsrxSldo6OVKqU1rekcHKlbqU2U3qlUqlCgAqygPdAUAAABU0GtFNbUEr\
                         FiAqi8RswMpgwOIvU2okq62LAVUNVpaldUggT1VZTUVKV3QqbUDQ3XaBSiQy9exaW1rTiG8uI7L5jlKa\
                         3a9rK05dtTeWpRVKqACrYYrVNVUKapqiqRQAASXyorVKxKIKi_3be6StVb9r0gqlVyqKmqa1ouL8Pj8P\
                         jj-GVnGbsWKptrWt4lAZcA2dgQKgNixVN7p2UVXhVElWDF0trTZi6MpRepVQ20KhSlKINU1RUBXmip-s\
                         7XErlOXlSmz222gakC1Wlyp_VbiMvWVKVFV0ml8ptUCLF7SvymNryqpSovaUEilKtxWim7qBibaoa1VS\
                         jU1SA1XErF_yKpMZSlNVlKtr5opTVKblVxF-6i_6v8oCmxdhFU3lt5SVsVsXpNOolRbZSoHutMaU1u6a\
                         3KBjZVZTeVPV9LUpTfFN5RWsRlJ0uL3eUlbF2qospXE0DZVlgbXqqOogfaqOD0ue3f5fyneK4sRJqkGm\
                         XPlYsRqN4jnJ8rqtJDLlBAV_V2yv2o1VvVVla3ENbK8BUr-JpVI1VUrVMrcSsrfVMGU23VGt801GxZWl\
                         6Nl6pgxVNa3EVSXymq3SsuI3lSlSlYuXzTkpSmqlVHRq0tu1U0DLn2qnhaGsKlVKa1Rlkh__2dgYWGGk\
                         aGRpZ2VzdElEGnHvWL5mcmFuZG9tUPHruXHjv35Iu-rzOkKBD2xxZWxlbWVudElkZW50aWZpZXJ2dW5f\
                         ZGlzdGluZ3Vpc2hpbmdfc2lnbmxlbGVtZW50VmFsdWVjVVNBamlzc3VlckF1dGiEQ6EBJqEYIVkCvjCC\
                         ArowggJhoAMCAQICFA7VlOKXxKg_rJ6UiVqmXVQjpptXMAoGCCqGSM49BAMCMGAxCzAJBgNVBAYTAlVT\
                         MQswCQYDVQQIDAJOWTEZMBcGA1UECgwQSVNPbURMIFRlc3QgUm9vdDEpMCcGA1UEAwwgSVNPMTgwMTMt\
                         NSBUZXN0IENlcnRpZmljYXRlIFJvb3QwHhcNMjMxMTE0MTAwMTA1WhcNMjQxMTEzMTAwMTA1WjBrMQsw\
                         CQYDVQQGEwJVUzELMAkGA1UECAwCTlkxIjAgBgNVBAoMGUlTT21ETCBUZXN0IElzc3VlciBTaWduZXIx\
                         KzApBgNVBAMMIklTTzE4MDEzLTUgVGVzdCBDZXJ0aWZpY2F0ZSBTaWduZXIwWTATBgcqhkjOPQIBBggq\
                         hkjOPQMBBwNCAAQgaK6KmQ1mWKt-Vo6ixfHxsmX9YlGAuUPkOvQ_uHrxgsZLC6FheRwtU3v-5GGkHD70\
                         FJNmz7DJUiR6G8TWMYZGo4HtMIHqMB0GA1UdDgQWBBQEpN0hSF6BFZJCDvZwASaa6ewoXzAfBgNVHSME\
                         GDAWgBQ1RoOxz04dQvKPF76VhBf-jMv3EDAxBglghkgBhvhCAQ0EJBYiSVNPMTgwMTMtNSBUZXN0IFNp\
                         Z25lciBDZXJ0aWZpY2F0ZTAOBgNVHQ8BAf8EBAMCB4AwFQYDVR0lAQH_BAswCQYHKIGMXQUBAjAdBgNV\
                         HRIEFjAUgRJleGFtcGxlQGlzb21kbC5jb20wLwYDVR0fBCgwJjAkoCKgIIYeaHR0cHM6Ly9leGFtcGxl\
                         LmNvbS9JU09tREwuY3JsMAoGCCqGSM49BAMCA0cAMEQCIGV5CQ0EFGjFzVBSqWfaPVUMziescVQ4W-lx\
                         w5bq7nCBAiBf1D9SPeA05Sdf0iWHanW3N0FBtS7Iz5XdSKWT2IqMKFkFzNgYWQXHpmd2ZXJzaW9uYzEu\
                         MG9kaWdlc3RBbGdvcml0aG1nU0hBLTI1Nmx2YWx1ZURpZ2VzdHOhcW9yZy5pc28uMTgwMTMuNS4xuB4a\
                         AZ9DYlggxtL2LRFm_GWjYft8lw02WZS4CLbChc-NakfbNNyTuAcaA8azMVggpZCl5WRduZFAMb0vykZC\
                         xA-AdeM1R8eoiC1d9d3pkLUaEU3f71ggHp0cG-ZNraR6vcOgLZSORP9rDEipOlXzHh18YamiANoaFT6f\
                         wlggEphIDgQUoblEdUCq34aMJ50OQ8QmCVFJuQgiB1_YwhoaFiE-91ggeEtCLmPzCOD0suxhDwW43s7y\
                         Nc1x6Jd4DZ6tO4ObD6IaFxmGZVgguLSRAKQ1dwJGOS_soukGSZUkCqvW7zN4R4eoTO-phFAaF_UrzFgg\
                         J4LaSDWgaeLF8hkPWLaDZGrjYCOuLYCcZk5tWXug4aMaGthQz1ggg3jAWBkkHRE9l4AoDdqdFYgJ56cr\
                         lzeAf47JtJ662VMaKXLy5lggQBw9uLpXYKFP4ZdoO7zzb_vtymKttmA_qeaaEW_jbWUaLBiyUVggR3An\
                         IGXnmMTMHPjuR19-e4lLs6vi3digyHN0iyzc3PkaMQCGYVggKvs-nJVYFRwH_TbFGPy1X_t69MR1l94t\
                         oRIIK98UBvAaMaFJllggCoQYDBIY_rk6s0MhMbC8ibfzGegfY-Pfwauy9GHW_38aMqbHz1ggtrjS6GQs\
                         MtSQaKf7Voa6kxPLDqK24EZ-9WhB8JaO4f4aNUVesFggOzxV3ZrJ8FQMCuThmR6L7B4SMN5bxLy05i6v\
                         9wgpejYaNt4zI1ggzzpJiJuTxtgMDAxycYe7TYmsovh5Aw_EDfDX8rRqYV4aOTYKk1ggEnnTJHhcMseT\
                         jVtPRGnCRRCE0WvwelO1dECZvlLXeIQaSkPcgFggyKs6jeFkCIjai1k5xYqZyqjK45ImuVOzPVC8jPXP\
                         XksaT9EOg1ggO9johmBdbxTYTcMQDSB1K9jwdd350VIjMuCHDZ8DDUEaUUgWklggGz_ddBP3N_0mxg7f\
                         co-oJ2HorIFAptTj78ZseE5gfmAaUUqijlgglmqfTA5d9Wi85wDcpdTO1NrlH02nOx4zP7FZ8TE7Kroa\
                         XagLOFggt3m7aHSfgJ3rEl1nn-Pp8YitK572a64L2GAa-UZCiGkaXyMT31gghLUPLnlPsZaDTk7Yd_Js\
                         jeEuWwIeAyu5FNeYRDkajlUaYLUtAlggb0He6jVXV1OGqzZidHIWpba3yCffluLpOiKAiVzVeXEaZNc0\
                         Z1gggSJJmw3Dt0YsH38Flq1hxybrP4tWRR6nSUUPZOah6BUaZYFFSlgg9-83mx19kFY-tbUDXndIdXL1\
                         oXs_2nYgchpnvWuENjIaaMXpU1gg5iCCWrzVNvXZa8I_jfmTpwZFxdmEfv7D3rcYhza65Dcaa4ST6Vgg\
                         rz6sluAmWjOaVRmKC6lLFyqEglIyTwZlGA3Q6tbF_WcacYeIV1ggk1fOHKjmMaPlh7SaVtWk-6jC38Dq\
                         PcWz3UcH6xuiZ1Mace9YvlggJCOAb023GTSEcNt48NYF2ZOM2p9RIqO8W91zORSzMFQaecXSi1ggFZp2\
                         VG3VaCEgchdZPgbK8JSuYsMCmy9Dy4WXyZD1Z1RtZGV2aWNlS2V5SW5mb6FpZGV2aWNlS2V5pAECIAEh\
                         WCCS9P-a8TB2KJzTBif0C32CrhjX3XKMVykLFFTHdFXpnSJYIADctY3zP9kjfSptLs9kyUhDUDRf4xOS\
                         Is0FkbyjHnsFZ2RvY1R5cGV1b3JnLmlzby4xODAxMy41LjEubURMbHZhbGlkaXR5SW5mb6Nmc2lnbmVk\
                         wHQyMDIzLTExLTE2VDA5OjI1OjIyWml2YWxpZEZyb23AdDIwMjMtMTEtMTZUMDk6MjU6MjJaanZhbGlk\
                         VW50aWzAdDIwMjMtMTItMTZUMDk6MjU6MjJaWEC8OJJedu29mak8hVi1X__VJhpQ6QhgOhTqHZMtdrqy\
                         Wdalv457ykvXnq3U5Zl5NC1GDyIDdr23_L67HUOKFqCHbGRldmljZVNpZ25lZKJqbmFtZVNwYWNlc9gY\
                         QaBqZGV2aWNlQXV0aKFvZGV2aWNlU2lnbmF0dXJlhEOhASag9lhAX4O7CImR03EijrZDHYgdzQefwdix\
                         5l-hJ7ow05OvOyQj0f_kW9GYbvWbDYbHN_kreXHaXpDh5Swm1nc5X39N6mZzdGF0dXMA"
                    ]
                }
            }
        );

        let auth_response: VpAuthorizationResponse = serde_json::from_value(example_json).unwrap();

        assert_eq!(auth_response.vp_token.len(), 1);

        let VerifiablePresentation::MsoMdoc(device_responses) = auth_response.vp_token.into_values().next().unwrap()
        else {
            panic!("received format should be mdoc")
        };

        assert_eq!(device_responses.len().get(), 1);

        let decrypted_device_response = device_responses.into_first();

        assert_eq!(
            decrypted_device_response
                .documents
                .as_ref()
                .map(|documents| documents.len())
                .unwrap_or_default(),
            1
        );

        let decrypted_document = decrypted_device_response.documents.unwrap().into_iter().next().unwrap();

        assert_eq!(decrypted_document.doc_type, "org.iso.18013.5.1.mDL".to_string());
    }

    #[test]
    fn deserialize_authorization_response_sd_jwt_example() {
        let example_json = json!(
            {
                "vp_token": {
                    "example_credential_id": [WITH_KB_SD_JWT]
                }
            }
        );

        let auth_response = serde_json::from_value::<VpAuthorizationResponse>(example_json).unwrap();

        assert_eq!(auth_response.vp_token.len(), 1);

        let VerifiablePresentation::SdJwt(sd_jwt_presentations) = auth_response.vp_token.into_values().next().unwrap()
        else {
            panic!("received format should be SD-JWT")
        };

        assert_eq!(sd_jwt_presentations.len().get(), 1);

        // TODO (PVW-4817): Test the deserialized types once we no longer use `String` to transport SD-JWT.
    }

    /// Construct mock `VerifiablePresentation`s and a PoA using the given
    /// authorization request, partial mdocs and encryption nonce.
    fn setup_mdoc_vp_token(
        auth_request: &NormalizedVpAuthorizationRequest,
        partial_mdocs: HashMap<CredentialQueryIdentifier, VecNonEmpty<PartialMdoc>>,
        wscd: &MockRemoteWscd,
    ) -> (HashMap<CredentialQueryIdentifier, VerifiablePresentation>, Option<Poa>) {
        let poa_input = JwtPoaInput::new(Some(auth_request.nonce.clone()), auth_request.client_id.to_string());

        let (query_ids, partial_mdocs) = partial_mdocs
            .into_iter()
            .map(|(query_id, partial_mdocs)| (query_id, partial_mdocs.into_iter().exactly_one().unwrap()))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let (device_responses, poa) = DeviceResponse::sign_multiple_from_partial_mdocs(
            partial_mdocs.try_into().unwrap(),
            &auth_request.session_transcript(),
            wscd,
            poa_input,
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        let vp_token = query_ids
            .into_iter()
            .zip_eq(device_responses)
            .map(|(query_id, device_response)| {
                let presentation = VerifiablePresentation::MsoMdoc(vec_nonempty![device_response]);

                (query_id, presentation)
            })
            .collect();

        (vp_token, poa)
    }

    #[test]
    fn test_verify_mdoc_authorization_response() {
        let (_, _, _, auth_request) =
            setup_with_credential_requests(NormalizedCredentialRequests::new_mock_mdoc_from_slices(
                &[(
                    PID_ATTESTATION_TYPE,
                    &[
                        &[PID_ATTESTATION_TYPE, "given_name"],
                        &[PID_ATTESTATION_TYPE, "family_name"],
                    ],
                )],
                None,
            ));

        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let holder_key = MockRemoteEcdsaKey::new_random("mdoc_key".to_string());
        let partial_mdoc = PartialMdoc::try_new(
            Mdoc::new_mock_with_ca_and_key(&ca, &holder_key).now_or_never().unwrap(),
            auth_request.credential_requests.as_ref().first().unwrap().claim_paths(),
        )
        .unwrap();

        let wscd = MockRemoteWscd::new(vec![holder_key]);

        let partial_mdocs = HashMap::from([("mdoc_0".parse().unwrap(), vec_nonempty![partial_mdoc])]);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, &wscd);
        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);

        let attestations = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &MockTimeGenerator::default(),
                &[ca.to_trust_anchor()],
                &ExtendingVctRetrieverStub,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock().unwrap(),
                ))),
                false,
            )
            .now_or_never()
            .unwrap()
            .expect("VpAuthorizationResponse should be valid");

        assert_eq!(attestations.len().get(), 1);

        let disclosed_attestations = attestations.into_inner().pop().unwrap().attestations;

        assert_eq!(disclosed_attestations.len().get(), 1);

        let attestation = disclosed_attestations.into_first();

        assert_eq!(attestation.attestation_type.as_str(), PID_ATTESTATION_TYPE);
        let DisclosedAttributes::MsoMdoc(attributes) = &attestation.attributes else {
            panic!("should be mdoc attributes")
        };

        let (namespace, mdoc_attributes) = &attributes.first().unwrap();
        assert_eq!(namespace.as_str(), PID_ATTESTATION_TYPE);

        assert_eq!(
            vec!["given_name", "family_name"],
            mdoc_attributes.keys().map(|key| key.as_str()).collect_vec()
        );
    }

    #[test]
    fn test_verify_sd_jwt_authorization_response() {
        // Set up an authorization request with two credential queries.
        let (_, _, _, auth_request) =
            setup_with_credential_requests(NormalizedCredentialRequests::new_mock_sd_jwt_from_slices(&[
                (&[PID_ATTESTATION_TYPE], &[&["bsn"], &["birthdate"]]),
                (&[PID_ATTESTATION_TYPE], &[&["given_name"], &["family_name"]]),
            ]));

        // Setup both issuer and holder keys.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let issuer_key_pair = ca
            .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
            .unwrap();

        let holder_key1 = MockRemoteEcdsaKey::new_random("sd_jwt_key_1".to_string());
        let holder_public_key1 = *holder_key1.verifying_key();
        let holder_key2 = MockRemoteEcdsaKey::new_random("sd_jwt_key_2".to_string());
        let holder_public_key2 = *holder_key2.verifying_key();
        let wscd = MockRemoteWscd::new(vec![holder_key1, holder_key2]);

        // Create two `UnsignedSdJwtPresentation`s with the requested attributes.
        let verified_sd_jwt1 = SignedSdJwt::pid_example(&issuer_key_pair, &holder_public_key1).into_verified();
        let unsigned_presentation1 = verified_sd_jwt1
            .into_presentation_builder()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("bsn".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("birthdate".to_string())])
            .unwrap()
            .finish();

        let verified_sd_jwt2 = SignedSdJwt::pid_example(&issuer_key_pair, &holder_public_key2).into_verified();
        let unsigned_presentation2 = verified_sd_jwt2
            .into_presentation_builder()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("given_name".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())])
            .unwrap()
            .finish();

        // Sign these into two `SdJwtPresentation`s and a PoA.
        let kb_jwt_builder = KeyBindingJwtBuilder::new(auth_request.client_id.to_string(), auth_request.nonce.clone());
        let poa_input = JwtPoaInput::new(Some(auth_request.nonce.clone()), auth_request.client_id.to_string());
        let (sd_jwt_presentations, poa) = UnsignedSdJwtPresentation::sign_multiple(
            vec_nonempty![
                (unsigned_presentation1, "sd_jwt_key_1"),
                (unsigned_presentation2, "sd_jwt_key_2")
            ],
            kb_jwt_builder,
            &wscd,
            poa_input,
            &MockTimeGenerator::default(),
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        let (sd_jwt_presentation1, sd_jwt_presentation2) = sd_jwt_presentations.into_iter().collect_tuple().unwrap();

        // Create a `VpAuthorizationResponse` from these values, then verify it.
        let vp_token = HashMap::from([
            (
                "sd_jwt_0".try_into().unwrap(),
                VerifiablePresentation::SdJwt(vec_nonempty![sd_jwt_presentation1.into_unverified()]),
            ),
            (
                "sd_jwt_1".try_into().unwrap(),
                VerifiablePresentation::SdJwt(vec_nonempty![sd_jwt_presentation2.into_unverified()]),
            ),
        ]);
        let auth_response = VpAuthorizationResponse::new(vp_token, auth_request.state.clone(), poa);

        let attestations = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &MockTimeGenerator::default(),
                &[ca.to_trust_anchor()],
                &ExtendingVctRetrieverStub,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock().unwrap(),
                ))),
                false,
            )
            .now_or_never()
            .unwrap()
            .expect("VpAuthorizationResponse should be valid");

        assert_eq!(attestations.len().get(), 2);

        // `auth_response.verify()` should return the attestations in the same order
        // as `auth_request.credential_requests`.
        auth_request
            .credential_requests
            .as_ref()
            .iter()
            .zip_eq(attestations.as_ref())
            .for_each(|(cred_request, attestations)| assert_eq!(*cred_request.id(), attestations.id));

        // Check the contents of the returned `DisclosedAttestation`s.
        let (disclosed_attestations1, disclosed_attestations2) = attestations
            .into_iter()
            .map(|attestations| attestations.attestations.into_iter().exactly_one().unwrap())
            .collect_tuple()
            .unwrap();

        assert_eq!(disclosed_attestations1.attestation_type, PID_ATTESTATION_TYPE);

        let DisclosedAttributes::SdJwt(attributes) = &disclosed_attestations1.attributes else {
            panic!("should be SD-JWT attributes");
        };

        assert_eq!(
            attributes
                .claim_paths(AttributesTraversalBehaviour::OnlyLeaves)
                .into_iter()
                .collect::<HashSet<_>>(),
            HashSet::from([
                vec_nonempty![ClaimPath::SelectByKey("bsn".to_string())],
                vec_nonempty![ClaimPath::SelectByKey("birthdate".to_string())]
            ])
        );

        assert_eq!(disclosed_attestations2.attestation_type, PID_ATTESTATION_TYPE);

        let DisclosedAttributes::SdJwt(attributes) = &disclosed_attestations2.attributes else {
            panic!("should be SD-JWT attributes");
        };

        assert_eq!(
            attributes
                .claim_paths(AttributesTraversalBehaviour::OnlyLeaves)
                .into_iter()
                .collect::<HashSet<_>>(),
            HashSet::from([
                vec_nonempty![ClaimPath::SelectByKey("given_name".to_string())],
                vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())]
            ])
        );
    }

    #[test]
    fn test_verify_mixed_authorization_response() {
        // Set up an authorization request with two credential queries, one for mdoc and one for SD_JWT.
        let mdoc_request = NormalizedCredentialRequest::new_mock_mdoc_pid_example();
        let sd_jwt_request = NormalizedCredentialRequest::new_mock_sd_jwt_pid_example();
        let requests = vec![mdoc_request, sd_jwt_request].try_into().unwrap();

        let (_, _, _, auth_request) = setup_with_credential_requests(requests);

        // Setup the issuer keys.
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let sd_jwt_issuer_key_pair = ca
            .generate_key_pair(ISSUANCE_CERT_CN, CertificateUsage::Mdl, Default::default())
            .unwrap();

        // Create an mdoc holder key and device response using the WSCD. Note that the `PoaInput` is not actually used.
        let mdoc_holder_key = MockRemoteEcdsaKey::new_random("mdoc".to_string());
        let partial_mdoc = PartialMdoc::try_new(
            Mdoc::new_mock_with_ca_and_key(&ca, &mdoc_holder_key)
                .now_or_never()
                .unwrap(),
            auth_request.credential_requests.as_ref().first().unwrap().claim_paths(),
        )
        .unwrap();

        let wscd = MockRemoteWscd::new(vec![mdoc_holder_key.clone()]);

        let poa_input = JwtPoaInput::new(None, "".to_string());
        let (device_responses, _) = DeviceResponse::sign_multiple_from_partial_mdocs(
            vec_nonempty![partial_mdoc],
            &auth_request.session_transcript(),
            &wscd,
            poa_input,
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        let device_response = device_responses.into_first();

        // Create the a SD-JWT holder key and presentation.
        let sd_jwt_holder_key = MockRemoteEcdsaKey::new_random("sd_jwt".to_string());
        let verified_sd_jwt =
            SignedSdJwt::pid_example(&sd_jwt_issuer_key_pair, sd_jwt_holder_key.verifying_key()).into_verified();
        let unsigned_presentation = verified_sd_jwt
            .into_presentation_builder()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("bsn".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("given_name".to_string())])
            .unwrap()
            .disclose(&vec_nonempty![ClaimPath::SelectByKey("family_name".to_string())])
            .unwrap()
            .finish();

        let kb_jwt_builder = KeyBindingJwtBuilder::new(auth_request.client_id.to_string(), auth_request.nonce.clone());
        let sd_jwt_presentation = unsigned_presentation
            .sign(kb_jwt_builder, &sd_jwt_holder_key, &MockTimeGenerator::default())
            .now_or_never()
            .unwrap()
            .unwrap();

        // Combine the two in a VP token.
        let vp_token = HashMap::from([
            (
                "mdoc_pid_example".try_into().unwrap(),
                VerifiablePresentation::MsoMdoc(vec_nonempty![device_response]),
            ),
            (
                "sd_jwt_pid_example".try_into().unwrap(),
                VerifiablePresentation::SdJwt(vec_nonempty![sd_jwt_presentation.into_unverified()]),
            ),
        ]);

        // Manually create a PoA accross the two holder keys.
        let poa = Poa::new(
            vec![&mdoc_holder_key, &sd_jwt_holder_key].try_into().unwrap(),
            JwtPopClaims::new(
                Some(auth_request.nonce.clone()),
                MOCK_WALLET_CLIENT_ID.to_string(),
                auth_request.client_id.to_string(),
            ),
        )
        .now_or_never()
        .unwrap()
        .unwrap();

        // Create a `VpAuthorizationResponse` from these values, then verify it.
        let auth_response = VpAuthorizationResponse::new(vp_token, auth_request.state.clone(), Some(poa));

        let attestations = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &MockTimeGenerator::default(),
                &[ca.to_trust_anchor()],
                &ExtendingVctRetrieverStub,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock().unwrap(),
                ))),
                false,
            )
            .now_or_never()
            .unwrap()
            .expect("VpAuthorizationResponse should be valid");

        assert_eq!(attestations.len().get(), 2);
    }

    fn setup_poa_test(
        ca: &Ca,
    ) -> (
        NormalizedVpAuthorizationRequest,
        HashMap<CredentialQueryIdentifier, VecNonEmpty<PartialMdoc>>,
        MockRemoteWscd,
    ) {
        let test_credentials = nl_pid_credentials_full_name() + nl_pid_address_minimal_address();
        let credential_requests =
            test_credentials.to_normalized_credential_requests([CredentialFormat::MsoMdoc, CredentialFormat::MsoMdoc]);

        let (_, _, _, auth_request) = setup_with_credential_requests(credential_requests);

        let issuer_keypair = ca
            .generate_key_pair(PID_ISSUER_CERT_CN, CertificateUsage::Mdl, Default::default())
            .unwrap();
        let wscd = MockRemoteWscd::default();

        let partial_mdocs = test_credentials.to_partial_mdocs(&issuer_keypair, &wscd);

        (auth_request, partial_mdocs, wscd)
    }

    #[test]
    fn test_verify_poa() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, &wscd);

        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);
        auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &MockTimeGenerator::default(),
                &[ca.to_trust_anchor()],
                &ExtendingVctRetrieverStub,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock_with_dn(PID_ISSUER_CERT_CN).unwrap(),
                ))),
                false,
            )
            .now_or_never()
            .unwrap()
            .unwrap();
    }

    #[test]
    fn test_verify_missing_poa() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, _) = setup_mdoc_vp_token(&auth_request, partial_mdocs, &wscd);

        let auth_response = VpAuthorizationResponse::new(vp_token, None, None);
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &MockTimeGenerator::default(),
                &[ca.to_trust_anchor()],
                &ExtendingVctRetrieverStub,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock_with_dn(PID_ISSUER_CERT_CN).unwrap(),
                ))),
                false,
            )
            .now_or_never()
            .unwrap()
            .expect_err("should fail due to missing PoA");

        assert!(matches!(error, AuthResponseError::MissingPoa));
    }

    #[test]
    fn test_verify_invalid_poa() {
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, &wscd);

        let mut poa = poa.unwrap();
        poa.set_payload("edited".to_owned());

        let auth_response = VpAuthorizationResponse::new(vp_token, None, Some(poa));
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                &MockTimeGenerator::default(),
                &[ca.to_trust_anchor()],
                &ExtendingVctRetrieverStub,
                &RevocationVerifier::new_without_caching(Arc::new(StatusListClientStub::new(
                    ca.generate_status_list_mock_with_dn(PID_ISSUER_CERT_CN).unwrap(),
                ))),
                false,
            )
            .now_or_never()
            .unwrap()
            .expect_err("should fail due to missing PoA");

        assert!(matches!(error, AuthResponseError::PoaVerification(_)));
    }

    #[rstest]
    #[case::c1(&[Some(RevocationStatus::Valid)], false, Ok(()))]
    #[case::c2(&[Some(RevocationStatus::Valid), Some(RevocationStatus::Valid)], false, Ok(()))]
    #[case::c3(&[Some(RevocationStatus::Revoked)], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c4(&[Some(RevocationStatus::Corrupted)], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c5(&[Some(RevocationStatus::Undetermined)], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c6(&[Some(RevocationStatus::Revoked), Some(RevocationStatus::Valid)], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c7(&[Some(RevocationStatus::Corrupted), Some(RevocationStatus::Valid)], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c8(&[Some(RevocationStatus::Undetermined), Some(RevocationStatus::Valid)], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c9(&[Some(RevocationStatus::Valid), Some(RevocationStatus::Valid), None], false, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c10(&[Some(RevocationStatus::Valid), Some(RevocationStatus::Valid)], true, Ok(()))]
    #[case::c11(&[Some(RevocationStatus::Undetermined)], true, Ok(()))]
    #[case::c12(&[Some(RevocationStatus::Valid), Some(RevocationStatus::Undetermined)], true, Ok(()))]
    #[case::c13(&[Some(RevocationStatus::Corrupted), Some(RevocationStatus::Valid)], true, Err(AuthResponseError::RevocationStatusNotAllValid))]
    #[case::c14(&[Some(RevocationStatus::Corrupted), Some(RevocationStatus::Undetermined)], true, Err(AuthResponseError::RevocationStatusNotAllValid))]
    fn test_evaluate_revocation_policy(
        #[case] statuses: &[Option<RevocationStatus>],
        #[case] accept_undetermined: bool,
        #[case] expected: Result<(), AuthResponseError>,
    ) {
        match VpAuthorizationResponse::evaluate_revocation_policy(statuses.iter(), accept_undetermined) {
            Ok(()) => assert!(expected.is_ok()),
            Err(error) => assert_eq!(error.to_string(), expected.err().unwrap().to_string()),
        };
    }

    #[test]
    fn test_authorization_request_object_as_reference() {
        let query = "client_id=test_client&request_uri=https%3A%2F%2Fclient.example.org%2Frequest%2Fvapof4ql2i7m41m68uep&request_uri_method=post";
        let result = serde_urlencoded::from_str::<VpRequestUri>(query).unwrap();

        assert_matches!(
            result,
            VpRequestUri {
                client_id,
                object: VpRequestUriObject::AsReference { .. },
            } if client_id.to_string() == "test_client"
        );
    }

    #[test]
    fn test_authorization_request_object_as_value() {
        let query = "client_id=test_client&request=eyJhbGciOiJSUzI1NiJ9.eyJpc3MiOiJzIiwiYXVkIjoicyJ9.sig";
        let result = serde_urlencoded::from_str::<VpRequestUri>(query).unwrap();

        assert_matches!(
            result,
            VpRequestUri {
                client_id,
                object: VpRequestUriObject::AsValue { request },
            } if client_id.to_string() == "test_client" && request.starts_with("eyJ")
        );
    }

    #[test]
    fn test_authorization_request_object_as_query_parameters() {
        let query = "response_type=vp_token&client_id=test_client&nonce=abc123";
        let result = serde_urlencoded::from_str::<VpRequestUri>(query).unwrap();

        assert_matches!(
            result,
            VpRequestUri {
                client_id,
                object: VpRequestUriObject::AsQueryParameters { response_type, nonce },
            } if response_type == "vp_token" && client_id.to_string() == "test_client" && nonce == "abc123"
        );
    }

    #[test]
    fn test_authorization_request_unrecognized() {
        let query = "foo=bar";
        let result = serde_urlencoded::from_str::<VpRequestUri>(query)
            .map_err(crate::disclosure_session::VpClientError::RequestUri)
            .expect_err("unrecognized request object should fail to parse as VpRequestUri");

        assert_matches!(result, crate::disclosure_session::VpClientError::RequestUri(_));
    }
}
