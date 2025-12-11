use std::collections::HashMap;
use std::string::FromUtf8Error;
use std::sync::LazyLock;
use std::time::Duration;

use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
use futures::future::try_join_all;
use indexmap::IndexSet;
use itertools::Itertools;
use josekit::JoseError;
use josekit::jwe::JweHeader;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use josekit::jwk::Jwk;
use josekit::jwk::alg::ec::EcKeyPair;
use josekit::jwt::JwtPayload;
use nutype::nutype;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::de::value::StringDeserializer;
use serde_with::DeserializeAs;
use serde_with::DeserializeFromStr;
use serde_with::SerializeDisplay;
use serde_with::serde_as;
use serde_with::skip_serializing_none;

use attestation_data::disclosure::DisclosedAttestation;
use attestation_data::disclosure::DisclosedAttestationError;
use attestation_data::disclosure::DisclosedAttestations;
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
use wscd::Poa;
use wscd::PoaVerificationError;

use crate::authorization::AuthorizationRequest;
use crate::authorization::ResponseMode;
use crate::authorization::ResponseType;

/// Leeway used in the lower end of the `iat` verification, used to account for clock skew.
const SD_JWT_IAT_LEEWAY: Duration = Duration::from_secs(5);
const SD_JWT_IAT_WINDOW: Duration = Duration::from_secs(15 * 60);

#[derive(Debug, thiserror::Error)]
pub enum AuthRequestError {
    #[error("error parsing X.509 certificate: {0}")]
    CertificateParsing(#[from] CertificateError),
    #[error("Subject Alternative Name missing from X.509 certificate")]
    MissingSAN,
}

/// A Request URI object, as defined in RFC 9101.
/// Contains URL from which the wallet is to retrieve the Authorization Request.
/// To be URL-encoded in the wallet's UL, which can then be put on a website directly, or in a QR code.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VpRequestUriObject {
    /// URL at which the full Authorization Request is to be retrieved.
    pub request_uri: BaseUrl,

    /// Whether or not the `request_uri` supports `POST`, instead of only `GET` as defined by RFC 9101.
    pub request_uri_method: Option<RequestUriMethod>,

    /// MUST equal the client_id from the full Authorization Request.
    pub client_id: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")] // Keeping these names as is might make more sense, but the spec says lowercase
pub enum RequestUriMethod {
    #[default]
    GET,
    POST,
}

/// An OpenID4VP Authorization Request, allowing an RP to request a set of credentials/attributes from a wallet.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpAuthorizationRequest {
    pub aud: VpAuthorizationRequestAudience,

    #[serde(flatten)]
    pub oauth_request: AuthorizationRequest,

    /// Contains requirements on the credentials and/or attributes to be disclosed.
    pub dcql_query: Query,

    /// Metadata about the verifier such as their encryption key(s).
    #[serde(flatten)]
    pub client_metadata: Option<VpClientMetadata>,

    /// Determines how the verifier is to be authenticated.
    pub client_id_scheme: Option<ClientIdScheme>,

    /// REQUIRED if the ResponseMode `direct_post` or `direct_post.jwt` is used.
    /// In that case, the Authorization Response is to be posted to this URL by the wallet.
    pub response_uri: Option<BaseUrl>,

    pub wallet_nonce: Option<String>,
}

impl JwtTyp for VpAuthorizationRequest {}

#[derive(Debug, Clone, Default, SerializeDisplay, DeserializeFromStr, strum::EnumString, strum::Display)]
pub enum VpAuthorizationRequestAudience {
    #[default]
    #[strum(to_string = "https://self-issued.me/v2")]
    SelfIssued,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpClientMetadata {
    #[serde(rename = "client_metadata")]
    Direct(ClientMetadata),
    #[serde(rename = "client_metadata_url")]
    Indirect(BaseUrl),
}

impl VpClientMetadata {
    pub fn direct(self) -> Option<ClientMetadata> {
        match self {
            VpClientMetadata::Direct(c) => Some(c),
            VpClientMetadata::Indirect(_) => None,
        }
    }
}

/// Metadata of the verifier (which acts as the "client" in OAuth).
/// OpenID4VP refers to https://openid.net/specs/openid-connect-registration-1_0.html and
/// https://www.rfc-editor.org/rfc/rfc7591.html for this, but here we implement only what we need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMetadata {
    #[serde(flatten)]
    pub jwks: VpJwks,
    pub vp_formats: VpFormat,

    // These two are defined in https://openid.net/specs/oauth-v2-jarm-final.html
    pub authorization_encryption_alg_values_supported: VpAlgValues,
    pub authorization_encryption_enc_values_supported: VpEncValues,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientIdScheme {
    #[serde(rename = "pre-registered")]
    PreRegistered,
    RedirectUri,
    EntityId,
    Did,
    VerifierAttestation,
    X509SanDns,
    X509SanUri,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpJwks {
    #[serde(rename = "jwks")]
    Direct { keys: Vec<Jwk> },
    #[serde(rename = "jwks_uri")]
    Indirect(BaseUrl),
}

impl VpJwks {
    pub fn direct(&self) -> Option<&Vec<Jwk>> {
        match self {
            VpJwks::Direct { keys } => Some(keys),
            VpJwks::Indirect(_) => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpAlgValues {
    #[serde(rename = "ECDH-ES")]
    EcdhEs,
}

#[derive(Debug, Clone, Serialize, Deserialize, strum::Display)]
pub enum VpEncValues {
    A128GCM,
    A192GCM,
    A256GCM,
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

#[nutype(
    derive(Debug, Clone, TryFrom, AsRef, Serialize, Deserialize),
    validate(predicate = |u| JwePublicKey::validate(u).is_ok()),
)]
pub struct JwePublicKey(Jwk);

impl JwePublicKey {
    fn validate(jwk: &Jwk) -> Result<(), AuthRequestValidationError> {
        // Avoid jwk.key_type() which panics if `kty` is not set.
        if jwk.parameter("kty").and_then(serde_json::Value::as_str) != Some("EC") {
            return Err(AuthRequestValidationError::UnsupportedJwk {
                field: "kty",
                expected: "EC",
                found: jwk.parameter("kty").cloned(),
            });
        }

        if jwk.curve() != Some("P-256") {
            return Err(AuthRequestValidationError::UnsupportedJwk {
                field: "crv",
                expected: "P-256",
                found: jwk.curve().map(serde_json::Value::from),
            });
        }

        Ok(())
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum AuthRequestValidationError {
    #[error("unexpected field: {0}")]
    #[category(critical)]
    UnexpectedField(&'static str),
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
    #[error("field {0}_uri found, expected field directly")]
    #[category(critical)]
    UriVariantNotSupported(&'static str),
    #[error("unexpected amount of JWKs found in client_metadata: expected 1, found {0}")]
    #[category(critical)]
    UnexpectedJwkAmount(usize),
    #[error("unsupported JWK: expected {expected}, found {found:?} in {field}")]
    #[category(pd)] // might leak sensitive data
    UnsupportedJwk {
        field: &'static str,
        expected: &'static str,
        found: Option<serde_json::Value>,
    },
    #[error("unsupported DCQL query: {0}")]
    UnsupportedDcqlQuery(#[from] UnsupportedDcqlFeatures),
    #[error(
        "client_id from Authorization Request was {client_id}, should have been equal to SAN DNSName from X.509 \
         certificate ({dns_san})"
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
    /// - the request contents are compliant with the profile from ISO 18013-7 Appendix B,
    /// - the `client_id` equals the DNS SAN name in the X.509 certificate, as required by the [`x509_san_dns` value for
    ///   `client_id_scheme`](https://openid.github.io/OpenID4VP/openid-4-verifiable-presentations-wg-draft.html#section-5.7-12.2),
    ///   which is used by the mentioned profile.
    ///
    /// This method consumes `self` and turns it into an [`IsoVpAuthorizationRequest`], which
    /// contains only the fields we need and use.
    pub fn validate(
        self,
        rp_cert: &BorrowingCertificate,
        wallet_nonce: Option<&str>,
    ) -> Result<NormalizedVpAuthorizationRequest, AuthRequestValidationError> {
        let dns_san = rp_cert.san_dns_name()?.ok_or(AuthRequestValidationError::MissingSAN)?;
        if dns_san != self.oauth_request.client_id {
            return Err(AuthRequestValidationError::UnauthorizedClientId {
                client_id: self.oauth_request.client_id,
                dns_san: String::from(dns_san),
            });
        }

        if wallet_nonce != self.wallet_nonce.as_deref() {
            return Err(AuthRequestValidationError::WalletNonceMismatch);
        }

        let validated_auth_request = NormalizedVpAuthorizationRequest::try_from(self)?;

        Ok(validated_auth_request)
    }
}

/// An OpenID4VP Authorization Request that has been validated to conform to the ISO 18013-7 profile:
/// a subset of [`VpAuthorizationRequest`] that always contains fields we require, and no fields we don't.
///
/// Note that this data type is internal to both the wallet and verifier, and not part of the OpenID4VP protocol,
/// so it is never sent over the wire. It implements (De)serialize so that the verifier can persist it to
/// the session store.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedVpAuthorizationRequest {
    pub client_id: String,
    pub nonce: String,
    pub encryption_pubkey: Jwk,
    pub response_uri: BaseUrl,
    pub credential_requests: NormalizedCredentialRequests,
    pub client_metadata: ClientMetadata,
    pub state: Option<String>,
    pub wallet_nonce: Option<String>,
}

impl NormalizedVpAuthorizationRequest {
    pub fn new(
        credential_requests: NormalizedCredentialRequests,
        rp_certificate: &BorrowingCertificate,
        nonce: String,
        encryption_pubkey: JwePublicKey,
        response_uri: BaseUrl,
        wallet_nonce: Option<String>,
    ) -> Result<Self, AuthRequestError> {
        let encryption_pubkey = encryption_pubkey.into_inner();

        Ok(Self {
            client_id: String::from(
                rp_certificate
                    .san_dns_name()
                    .map_err(AuthRequestError::CertificateParsing)?
                    .ok_or(AuthRequestError::MissingSAN)?,
            ),
            nonce,
            encryption_pubkey: encryption_pubkey.clone(),
            response_uri,
            credential_requests,
            client_metadata: ClientMetadata {
                jwks: VpJwks::Direct {
                    keys: vec![encryption_pubkey.clone()],
                },
                vp_formats: VpFormat::MsoMdoc {
                    alg: IndexSet::from([FormatAlg::ES256]),
                },
                authorization_encryption_alg_values_supported: VpAlgValues::EcdhEs,
                authorization_encryption_enc_values_supported: VpEncValues::A128GCM,
            },
            state: None,
            wallet_nonce,
        })
    }
}

impl From<NormalizedVpAuthorizationRequest> for VpAuthorizationRequest {
    fn from(value: NormalizedVpAuthorizationRequest) -> Self {
        Self {
            aud: VpAuthorizationRequestAudience::SelfIssued,
            oauth_request: AuthorizationRequest {
                response_type: ResponseType::VpToken.into(),
                client_id: value.client_id,
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
            client_metadata: Some(VpClientMetadata::Direct(value.client_metadata)),
            client_id_scheme: Some(ClientIdScheme::X509SanDns),
            response_uri: Some(value.response_uri),
            wallet_nonce: value.wallet_nonce,
        }
    }
}

impl TryFrom<VpAuthorizationRequest> for NormalizedVpAuthorizationRequest {
    type Error = AuthRequestValidationError;

    fn try_from(vp_auth_request: VpAuthorizationRequest) -> Result<Self, Self::Error> {
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

        // Check presence of fields that must be present in an OpenID4VP Authorization Request
        if vp_auth_request.oauth_request.nonce.is_none() {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("nonce"));
        }
        if vp_auth_request.oauth_request.response_mode.is_none() {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("response_mode"));
        }
        if vp_auth_request.client_id_scheme.is_none() {
            return Err(AuthRequestValidationError::ExpectedFieldMissing("client_id_scheme"));
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
        if *vp_auth_request.oauth_request.response_mode.as_ref().unwrap() != ResponseMode::DirectPostJwt {
            return Err(AuthRequestValidationError::UnsupportedFieldValue {
                field: "response_mode",
                expected: "direct_post.jwt",
                found: serde_json::to_string(&vp_auth_request.oauth_request.response_mode).unwrap(),
            });
        }
        if *vp_auth_request.client_id_scheme.as_ref().unwrap() != ClientIdScheme::X509SanDns {
            return Err(AuthRequestValidationError::UnsupportedFieldValue {
                field: "client_id_scheme",
                expected: "x509_san_dns",
                found: serde_json::to_string(&vp_auth_request.client_id_scheme).unwrap(),
            });
        }

        // Of fields that have an "_uri" variant, check that they are not used
        let Some(client_metadata) = client_metadata.direct() else {
            return Err(AuthRequestValidationError::UriVariantNotSupported("client_metadata"));
        };
        let Some(jwks) = client_metadata.jwks.direct() else {
            return Err(AuthRequestValidationError::UriVariantNotSupported(
                "client_metadata.jwks",
            ));
        };

        // check that we received exactly one EC/P256 curve
        if jwks.len() != 1 {
            return Err(AuthRequestValidationError::UnexpectedJwkAmount(jwks.len()));
        }
        let jwk = jwks.first().unwrap().clone();
        JwePublicKey::validate(&jwk)?;

        Ok(NormalizedVpAuthorizationRequest {
            client_id: vp_auth_request.oauth_request.client_id,
            nonce: vp_auth_request.oauth_request.nonce.unwrap(),
            encryption_pubkey: jwk,
            credential_requests: vp_auth_request.dcql_query.try_into()?,
            response_uri: vp_auth_request.response_uri.unwrap(),
            client_metadata,
            state: vp_auth_request.oauth_request.state,
            wallet_nonce: vp_auth_request.wallet_nonce,
        })
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(unexpected)]
pub enum AuthResponseError {
    #[error("error (de)serializing JWE payload: {0}")]
    Json(#[from] serde_json::Error),

    #[error("error parsing JWK: {0}")]
    JwkConversion(#[source] JoseError),

    #[error("error encrypting/decrypting JWE: {0}")]
    Jwe(#[source] JoseError),

    #[error("state had incorrect value: expected {expected:?}, found {found:?}")]
    StateIncorrect {
        expected: Option<String>,
        found: Option<String>,
    },

    #[error("missing apu field from JWE")]
    MissingApu,

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
        encryption_nonce: &str,
        poa: Option<Poa>,
    ) -> Result<String, AuthResponseError> {
        Self::new(vp_token, auth_request.state.clone(), poa).encrypt(auth_request, encryption_nonce)
    }

    fn encrypt(
        &self,
        auth_request: &NormalizedVpAuthorizationRequest,
        encryption_nonce: &str,
    ) -> Result<String, AuthResponseError> {
        let mut header = JweHeader::new();
        header.set_token_type("JWT");

        // Set the `apu` and `apv` fields to the holder encryption nonce and nonce, per the ISO 18013-7 profile.
        // Even when not disclosing mdoc credentials, using a holder-generated nonce for this value is perfectly
        // acceptable, as the verifier should not make any assumptions about these values in that case.
        header.set_agreement_partyuinfo(encryption_nonce);
        header.set_agreement_partyvinfo(auth_request.nonce.clone());

        // Use the AES key size that the server wants.
        header.set_content_encryption(
            auth_request
                .client_metadata
                .authorization_encryption_enc_values_supported
                .to_string(),
        );

        // VpAuthorizationRequest always serializes to a JSON object useable as a JWT payload.
        let serde_json::Value::Object(payload) = serde_json::to_value(self)? else {
            panic!("VpAuthorizationResponse did not serialize to object")
        };
        let payload = JwtPayload::from_map(payload).unwrap();

        // The key the RP wants us to encrypt our response to.
        let encrypter = EcdhEsJweAlgorithm::EcdhEs
            .encrypter_from_jwk(&auth_request.encryption_pubkey)
            .map_err(AuthResponseError::JwkConversion)?;

        let jwe = josekit::jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(AuthResponseError::Jwe)?;
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
        let (response, encryption_nonce) = Self::decrypt(jwe, private_key)?;

        response
            .verify(
                auth_request,
                accepted_wallet_client_ids,
                encryption_nonce.as_deref(),
                time,
                trust_anchors,
                extending_vct_values,
                revocation_verifier,
                accept_undetermined_revocation_status,
            )
            .await
    }

    fn decrypt(
        jwe: &str,
        private_key: &EcKeyPair,
    ) -> Result<(VpAuthorizationResponse, Option<String>), AuthResponseError> {
        let decrypter = EcdhEsJweAlgorithm::EcdhEs
            .decrypter_from_jwk(&private_key.to_jwk_key_pair())
            .map_err(AuthResponseError::JwkConversion)?;
        let (payload, header) = josekit::jwt::decode_with_decrypter(jwe, &decrypter).map_err(AuthResponseError::Jwe)?;

        // Note that it is up to the holder to choose the contents of the `apv` and `apu` fields. We only need the `apu`
        // value in case a credential in mdoc format was disclosed, as according to the ISO 18013-7 profile this value
        // should be used when re-constructing the `SessionTranscript`.
        let encryption_nonce = header.agreement_partyuinfo().map(String::from_utf8).transpose()?;

        let payload = serde_json::from_value(serde_json::Value::Object(payload.into()))?;
        Ok((payload, encryption_nonce))
    }

    #[expect(clippy::too_many_arguments)]
    async fn verify<C>(
        self,
        auth_request: &NormalizedVpAuthorizationRequest,
        accepted_wallet_client_ids: &[String],
        encryption_nonce: Option<&str>,
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
        let mut holder_public_keys: Vec<VerifyingKey> = Vec::new();
        let mut disclosed_attestations = HashMap::new();

        for (id, presentation) in self.vp_token {
            let (attestations, public_keys) = match presentation {
                VerifiablePresentation::MsoMdoc(device_responses) => {
                    let (keys, attestations): (Vec<_>, Vec<_>) =
                        try_join_all(device_responses.iter().map(|device_response| async {
                            Self::mdoc_to_disclosed_attestation(
                                device_response,
                                auth_request,
                                encryption_nonce,
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
                auth_request.client_id.as_str(),
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
        auth_request: &NormalizedVpAuthorizationRequest,
        encryption_nonce: Option<&str>,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor<'_>],
        revocation_verifier: &RevocationVerifier<C>,
    ) -> Result<Vec<(VerifyingKey, DisclosedAttestation)>, AuthResponseError>
    where
        C: StatusListClient,
    {
        let session_transcript = encryption_nonce.map(|encryption_nonce|
            // The mdoc `SessionTranscript` may not be required, so initialize it lazily.
            SessionTranscript::new_oid4vp(
                &auth_request.response_uri,
                &auth_request.client_id,
                auth_request.nonce.clone(),
                encryption_nonce,
            ));

        // Verify the cryptographic integrity of each mdoc `DeviceResponse`
        // and obtain a `DisclosedDocuments` for each.
        let disclosed_documents = device_response
            .verify(
                None,
                &session_transcript.ok_or(AuthResponseError::MissingApu)?,
                time,
                trust_anchors,
                revocation_verifier,
            )
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
            expected_aud: &auth_request.client_id,
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

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::sync::Arc;

    use assert_matches::assert_matches;
    use futures::FutureExt;
    use itertools::Itertools;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use rstest::rstest;
    use rustls_pki_types::TrustAnchor;
    use serde_json::json;

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
    use jwt::SignedJwt;
    use jwt::pop::JwtPopClaims;
    use mdoc::DeviceResponse;
    use mdoc::SessionTranscript;
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
    use crate::mock::ExtendingVctRetrieverStub;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::openid4vp::AuthResponseError;
    use crate::openid4vp::NormalizedVpAuthorizationRequest;

    use super::VerifiablePresentation;
    use super::VpAuthorizationRequest;
    use super::VpAuthorizationResponse;

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

        let auth_request = NormalizedVpAuthorizationRequest::new(
            credential_requests,
            rp_keypair.certificate(),
            "nonce".to_string(),
            encryption_privkey.to_jwk_public_key().try_into().unwrap(),
            "https://example.com/response_uri".parse().unwrap(),
            None,
        )
        .unwrap();

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
        let jwe = auth_response.encrypt(&auth_request, &encryption_nonce).unwrap();

        let (decrypted, jwe_encryption_nonce) = VpAuthorizationResponse::decrypt(&jwe, &encryption_privkey).unwrap();

        assert_eq!(jwe_encryption_nonce, Some(encryption_nonce));
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
        auth_request.validate(&cert, None).unwrap();
    }

    #[test]
    fn deserialize_authorization_request_example() {
        let example_json = json!(
            {
                "aud": "https://self-issued.me/v2",
                "response_type": "vp_token",
                "response_mode": "direct_post.jwt",
                "client_id_scheme": "x509_san_dns",
                "client_id": "example.com",
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
                    "authorization_encryption_alg_values_supported": "ECDH-ES",
                    "authorization_encryption_enc_values_supported": "A256GCM",
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
        NormalizedVpAuthorizationRequest::try_from(auth_request).unwrap();
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
        encryption_nonce: &str,
        wscd: &MockRemoteWscd,
    ) -> (HashMap<CredentialQueryIdentifier, VerifiablePresentation>, Option<Poa>) {
        let session_transcript = SessionTranscript::new_oid4vp(
            &auth_request.response_uri,
            &auth_request.client_id,
            auth_request.nonce.clone(),
            encryption_nonce,
        );

        let poa_input = JwtPoaInput::new(Some(auth_request.nonce.clone()), auth_request.client_id.clone());

        let (query_ids, partial_mdocs) = partial_mdocs
            .into_iter()
            .map(|(query_id, partial_mdocs)| (query_id, partial_mdocs.into_iter().exactly_one().unwrap()))
            .unzip::<_, _, Vec<_>, Vec<_>>();

        let (device_responses, poa) = DeviceResponse::sign_multiple_from_partial_mdocs(
            partial_mdocs.try_into().unwrap(),
            &session_transcript,
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
        let encryption_nonce = "encryption_nonce";

        let partial_mdocs = HashMap::from([("mdoc_0".parse().unwrap(), vec_nonempty![partial_mdoc])]);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, encryption_nonce, &wscd);
        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);

        let attestations = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                Some(encryption_nonce),
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
        let kb_jwt_builder = KeyBindingJwtBuilder::new(auth_request.client_id.clone(), auth_request.nonce.clone());
        let poa_input = JwtPoaInput::new(Some(auth_request.nonce.clone()), auth_request.client_id.clone());
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
                None,
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

        let encryption_nonce = "encryption_nonce";
        let session_transcript = SessionTranscript::new_oid4vp(
            &auth_request.response_uri,
            &auth_request.client_id,
            auth_request.nonce.clone(),
            encryption_nonce,
        );

        let wscd = MockRemoteWscd::new(vec![mdoc_holder_key.clone()]);

        let poa_input = JwtPoaInput::new(None, "".to_string());
        let (device_responses, _) = DeviceResponse::sign_multiple_from_partial_mdocs(
            vec_nonempty![partial_mdoc],
            &session_transcript,
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

        let kb_jwt_builder = KeyBindingJwtBuilder::new(auth_request.client_id.clone(), auth_request.nonce.clone());
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
                auth_request.client_id.clone(),
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
                Some(encryption_nonce),
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
        let encryption_nonce = "encryption_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, encryption_nonce, &wscd);

        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);
        auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                Some(encryption_nonce),
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
    fn test_verify_apu() {
        let encryption_nonce = "encryption_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, encryption_nonce, &wscd);

        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                None,
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
            .expect_err("verifying authorization response should fail");

        assert_matches!(error, AuthResponseError::MissingApu);
    }

    #[test]
    fn test_verify_missing_poa() {
        let encryption_nonce = "encryption_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, _) = setup_mdoc_vp_token(&auth_request, partial_mdocs, encryption_nonce, &wscd);

        let auth_response = VpAuthorizationResponse::new(vp_token, None, None);
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                Some(encryption_nonce),
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
        let encryption_nonce = "encryption_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (auth_request, partial_mdocs, wscd) = setup_poa_test(&ca);
        let (vp_token, poa) = setup_mdoc_vp_token(&auth_request, partial_mdocs, encryption_nonce, &wscd);

        let mut poa = poa.unwrap();
        poa.set_payload("edited".to_owned());

        let auth_response = VpAuthorizationResponse::new(vp_token, None, Some(poa));
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                Some(encryption_nonce),
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
}
