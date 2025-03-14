use std::string::FromUtf8Error;

use base64::DecodeError;
use chrono::DateTime;
use chrono::Utc;
use indexmap::IndexSet;
use itertools::Itertools;
use josekit::jwe::alg::ecdh_es::EcdhEsJweAlgorithm;
use josekit::jwe::JweHeader;
use josekit::jwk::alg::ec::EcKeyPair;
use josekit::jwk::Jwk;
use josekit::jwt::JwtPayload;
use josekit::JoseError;
use nutype::nutype;
use p256::ecdsa::VerifyingKey;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::formats::PreferOne;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::OneOrMany;

use error_category::ErrorCategory;
use jwt::error::JwtX5cError;
use jwt::Jwt;
use jwt::NL_WALLET_CLIENT_ID;
use nl_wallet_mdoc::errors::Error as MdocError;
use nl_wallet_mdoc::utils::serialization::CborBase64;
use nl_wallet_mdoc::utils::x509::BorrowingCertificate;
use nl_wallet_mdoc::utils::x509::CertificateError;
use nl_wallet_mdoc::verifier::DisclosedAttributes;
use nl_wallet_mdoc::verifier::ItemsRequests;
use nl_wallet_mdoc::DeviceResponse;
use nl_wallet_mdoc::SessionTranscript;
use poa::Poa;
use poa::PoaVerificationError;
use wallet_common::generator::Generator;
use wallet_common::generator::TimeGenerator;
use wallet_common::urls::BaseUrl;
use wallet_common::utils::random_string;

use crate::authorization::AuthorizationRequest;
use crate::authorization::ResponseMode;
use crate::authorization::ResponseType;
use crate::presentation_exchange::InputDescriptorMappingObject;
use crate::presentation_exchange::PdConversionError;
use crate::presentation_exchange::PresentationDefinition;
use crate::presentation_exchange::PresentationSubmission;
use crate::presentation_exchange::PsError;
use crate::Format;

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
    #[serde(flatten)]
    pub presentation_definition: VpPresentationDefinition,

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

#[derive(Debug, Clone, Default, Serialize, Deserialize, strum::Display)]
pub enum VpAuthorizationRequestAudience {
    #[default]
    #[serde(rename = "https://self-issued.me/v2")]
    #[strum(to_string = "https://self-issued.me/v2")]
    SelfIssued,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpPresentationDefinition {
    #[serde(rename = "presentation_definition")]
    Direct(PresentationDefinition),
    #[serde(rename = "presentation_definition_url")]
    Indirect(BaseUrl),
}

impl VpPresentationDefinition {
    pub fn direct(self) -> Option<PresentationDefinition> {
        match self {
            VpPresentationDefinition::Direct(pd) => Some(pd),
            VpPresentationDefinition::Indirect(_) => None,
        }
    }
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
    #[error("no attributes were requested")]
    #[category(critical)]
    NoAttributesRequested,
    #[error("unsupported Presentation Definition: {0}")]
    UnsupportedPresentationDefinition(#[from] PdConversionError),
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

impl VpAuthorizationRequest {
    /// Construct a new Authorization Request by verifying an Authorization Request JWT against
    /// the specified trust anchors.
    pub fn try_new(
        jws: &Jwt<VpAuthorizationRequest>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(VpAuthorizationRequest, BorrowingCertificate), AuthRequestValidationError> {
        Ok(jws.verify_against_trust_anchors(
            &[VpAuthorizationRequestAudience::SelfIssued],
            trust_anchors,
            &TimeGenerator,
        )?)
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
    ) -> Result<IsoVpAuthorizationRequest, AuthRequestValidationError> {
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

        let validated_auth_request = IsoVpAuthorizationRequest::try_from(self)?;

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
pub struct IsoVpAuthorizationRequest {
    pub client_id: String,
    pub nonce: String,
    pub encryption_pubkey: Jwk,
    pub response_uri: BaseUrl,
    pub items_requests: ItemsRequests,
    pub presentation_definition: PresentationDefinition,
    pub client_metadata: ClientMetadata,
    pub state: Option<String>,
    pub wallet_nonce: Option<String>,
}

impl IsoVpAuthorizationRequest {
    pub fn new(
        items_requests: &ItemsRequests,
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
            presentation_definition: items_requests.into(),
            items_requests: items_requests.clone(),
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

impl From<IsoVpAuthorizationRequest> for VpAuthorizationRequest {
    fn from(value: IsoVpAuthorizationRequest) -> Self {
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
            presentation_definition: VpPresentationDefinition::Direct(value.presentation_definition),
            client_metadata: Some(VpClientMetadata::Direct(value.client_metadata)),
            client_id_scheme: Some(ClientIdScheme::X509SanDns),
            response_uri: Some(value.response_uri),
            wallet_nonce: value.wallet_nonce,
        }
    }
}

impl TryFrom<VpAuthorizationRequest> for IsoVpAuthorizationRequest {
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
        let Some(presentation_definition) = vp_auth_request.presentation_definition.direct() else {
            return Err(AuthRequestValidationError::UriVariantNotSupported(
                "presentation_definition",
            ));
        };
        let Some(client_metadata) = client_metadata.direct() else {
            return Err(AuthRequestValidationError::UriVariantNotSupported("client_metadata"));
        };
        let Some(jwks) = client_metadata.jwks.direct() else {
            return Err(AuthRequestValidationError::UriVariantNotSupported(
                "presentation_definition",
            ));
        };

        // check that we received exactly one EC/P256 curve
        if jwks.len() != 1 {
            return Err(AuthRequestValidationError::UnexpectedJwkAmount(jwks.len()));
        }
        let jwk = jwks.first().unwrap().clone();
        JwePublicKey::validate(&jwk)?;

        if presentation_definition
            .input_descriptors
            .iter()
            .all(|i| i.constraints.fields.is_empty())
        {
            return Err(AuthRequestValidationError::NoAttributesRequested);
        }

        Ok(IsoVpAuthorizationRequest {
            client_id: vp_auth_request.oauth_request.client_id,
            nonce: vp_auth_request.oauth_request.nonce.unwrap(),
            encryption_pubkey: jwk,
            items_requests: (&presentation_definition).try_into()?,
            response_uri: vp_auth_request.response_uri.unwrap(),
            presentation_definition,
            client_metadata,
            state: vp_auth_request.oauth_request.state,
            wallet_nonce: vp_auth_request.wallet_nonce,
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum AuthResponseError {
    #[error("error (de)serializing JWE payload: {0}")]
    Json(#[from] serde_json::Error),
    #[error("error parsing JWK: {0}")]
    JwkConversion(#[source] JoseError),
    #[error("error encrypting/decrypting JWE: {0}")]
    Jwe(#[source] JoseError),
    #[error("apv (nonce) field in JWE had incorrect value")]
    NonceIncorrect,
    #[error("state had incorrect value: expected {expected:?}, found {found:?}")]
    StateIncorrect {
        expected: Option<String>,
        found: Option<String>,
    },
    #[error("failed to base64 decode JWE header fields: {0}")]
    Base64(#[from] DecodeError),
    #[error("missing apu field from JWE")]
    MissingApu,
    #[error("missing apv field from JWE")]
    MissingApv,
    #[error("failed to decode apu/apv field from JWE")]
    Utf8(#[from] FromUtf8Error),
    #[error("error verifying disclosed mdoc(s): {0}")]
    Verification(#[source] nl_wallet_mdoc::Error),
    #[error("missing requested attributes: {0}")]
    MissingAttributes(#[source] nl_wallet_mdoc::Error),
    #[error("received unexpected amount of Verifiable Presentations: expected 1, found {0}")]
    UnexpectedVpCount(usize),
    #[error("error in Presentation Submission: {0}")]
    PresentationSubmission(#[from] PsError),
    #[error("error collecting keys to verify PoA: {0}")]
    PoaKeys(#[source] nl_wallet_mdoc::Error),
    #[error("missing PoA")]
    MissingPoa,
    #[error("error verifying PoA: {0}")]
    PoaVerification(#[from] PoaVerificationError),
}

// We do not reuse or embed the `AuthorizationResponse` struct from `authorization.rs`, because in no variant
// of OpenID4VP that we (plan to) support do we need the `code` field from that struct, which is its primary citizen.
/// An OpenID4VP Authorization Response, with the wallet's disclosed credentials/attributes in the `vp_token`.
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpAuthorizationResponse {
    /// One or more Verifiable Presentations.
    #[serde_as(as = "OneOrMany<_, PreferOne>")]
    pub vp_token: Vec<VerifiablePresentation>,

    pub presentation_submission: PresentationSubmission,

    /// MUST equal the `state` from the Authorization Request.
    /// May be used by the RP to link incoming Authorization Responses to its corresponding Authorization Request,
    /// for example in case the `response_uri` contains no session token or other identifier.
    pub state: Option<String>,

    pub poa: Option<Poa>,
}

/// Disclosure of an credential, generally containing the issuer-signed credential itself, the disclosed attributes,
/// and a holder signature over some nonce provided by the verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerifiablePresentation {
    // NB: a `DeviceResponse` can contain disclosures of multiple mdocs.
    // In case of other (not yet supported) formats, each credential is expected to result in a separate
    // Verifiable Presentation. See e.g. this example:
    // https://openid.github.io/OpenID4VP/openid-4-verifiable-presentations-wg-draft.html#section-6.1-13
    MsoMdoc(CborBase64<DeviceResponse>),
}

impl VpAuthorizationResponse {
    fn new(device_response: DeviceResponse, auth_request: &IsoVpAuthorizationRequest, poa: Option<Poa>) -> Self {
        let presentation_submission = PresentationSubmission {
            id: random_string(16),
            definition_id: auth_request.presentation_definition.id.clone(),
            descriptor_map: device_response
                .documents
                .as_ref()
                .unwrap() // we never produce DeviceResponse instances without documents in it
                .iter()
                .map(|doc| InputDescriptorMappingObject {
                    id: doc.doc_type.clone(),
                    format: Format::MsoMdoc,
                    path: "$".to_string(),
                })
                .collect(),
        };

        VpAuthorizationResponse {
            vp_token: vec![VerifiablePresentation::MsoMdoc(device_response.into())],
            presentation_submission,
            state: auth_request.state.clone(),
            poa,
        }
    }

    /// Create a JWE containing a new encrypted Authorization Request.
    pub fn new_encrypted(
        device_response: DeviceResponse,
        auth_request: &IsoVpAuthorizationRequest,
        mdoc_nonce: &str,
        poa: Option<Poa>,
    ) -> Result<String, AuthResponseError> {
        Self::new(device_response, auth_request, poa).encrypt(auth_request, mdoc_nonce)
    }

    fn encrypt(&self, auth_request: &IsoVpAuthorizationRequest, mdoc_nonce: &str) -> Result<String, AuthResponseError> {
        let mut header = JweHeader::new();
        header.set_token_type("JWT");

        // Set the `apu` and `apv` fields to the mdoc nonce and nonce, per the ISO 18013-7 profile.
        header.set_agreement_partyuinfo(mdoc_nonce);
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

    pub fn decrypt_and_verify(
        jwe: &str,
        private_key: &EcKeyPair,
        auth_request: &IsoVpAuthorizationRequest,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<DisclosedAttributes, AuthResponseError> {
        let (response, mdoc_nonce) = Self::decrypt(jwe, private_key, &auth_request.nonce)?;

        response.verify(auth_request, &mdoc_nonce, time, trust_anchors)
    }

    pub fn decrypt(
        jwe: &str,
        private_key: &EcKeyPair,
        nonce: &str,
    ) -> Result<(VpAuthorizationResponse, String), AuthResponseError> {
        let decrypter = EcdhEsJweAlgorithm::EcdhEs
            .decrypter_from_jwk(&private_key.to_jwk_key_pair())
            .map_err(AuthResponseError::JwkConversion)?;
        let (payload, header) = josekit::jwt::decode_with_decrypter(jwe, &decrypter).map_err(AuthResponseError::Jwe)?;

        let jwe_nonce = String::from_utf8(header.agreement_partyvinfo().ok_or(AuthResponseError::MissingApv)?)?;
        if nonce != jwe_nonce {
            return Err(AuthResponseError::NonceIncorrect);
        }
        let mdoc_nonce = String::from_utf8(header.agreement_partyuinfo().ok_or(AuthResponseError::MissingApu)?)?;

        let payload = serde_json::from_value(serde_json::Value::Object(payload.into()))?;
        Ok((payload, mdoc_nonce))
    }

    fn device_response(&self) -> Result<&DeviceResponse, AuthResponseError> {
        if self.vp_token.len() != 1 {
            return Err(AuthResponseError::UnexpectedVpCount(self.vp_token.len()));
        }

        let VerifiablePresentation::MsoMdoc(device_response) = &self.vp_token.first().unwrap();
        Ok(&device_response.0)
    }

    fn keys(&self) -> Result<Vec<VerifyingKey>, MdocError> {
        let keys = self
            .vp_token
            .iter()
            .map(|vp| match vp {
                VerifiablePresentation::MsoMdoc(device_response) => device_response
                    .0
                    .documents
                    .as_ref()
                    .map(|docs| {
                        docs.iter()
                            .map(|doc| {
                                doc.issuer_signed.public_key() // VerifyingKey implements Copy, thereby saving use a
                                                               // clone here
                            })
                            .collect::<Result<Vec<_>, MdocError>>()
                    })
                    .unwrap_or(Ok(Default::default())), // empty list if no documents are present
            })
            .collect::<Result<Vec<_>, MdocError>>()?
            .into_iter()
            .flatten()
            .collect_vec();

        Ok(keys)
    }

    pub fn verify(
        &self,
        auth_request: &IsoVpAuthorizationRequest,
        mdoc_nonce: &str,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<DisclosedAttributes, AuthResponseError> {
        // Verify the cryptographic integrity of the disclosed attributes.
        let session_transcript = SessionTranscript::new_oid4vp(
            &auth_request.response_uri,
            &auth_request.client_id,
            auth_request.nonce.clone(),
            mdoc_nonce,
        );
        let device_response = self.device_response()?;
        let disclosed_attrs = device_response
            .verify(None, &session_transcript, time, trust_anchors)
            .map_err(AuthResponseError::Verification)?;

        // Verify PoA
        let used_keys = self
            .keys()
            .expect("should always succeed when called after DeviceResponse::verify");
        if used_keys.len() >= 2 {
            self.poa.as_ref().ok_or(AuthResponseError::MissingPoa)?.clone().verify(
                &used_keys,
                auth_request.client_id.as_str(),
                NL_WALLET_CLIENT_ID,
                mdoc_nonce,
            )?
        }

        // Check that we received all attributes that we requested
        auth_request
            .items_requests
            .match_against_response(device_response)
            .map_err(AuthResponseError::MissingAttributes)?;

        // Safe: if we have found all requested items in the documents, then the documents are not absent.
        let documents = device_response.documents.as_ref().unwrap();

        // Check that the Presentation Submission is what it should be per the Presentation Exchange spec and ISO
        // 18013-7.
        self.presentation_submission
            .verify(documents, &auth_request.presentation_definition)?;

        // If `state` is provided it must equal the `state` from the Authorization Request.
        if self.state != auth_request.state {
            return Err(AuthResponseError::StateIncorrect {
                expected: auth_request.state.clone(),
                found: self.state.clone(),
            });
        }

        Ok(disclosed_attrs)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpResponse {
    pub redirect_uri: Option<BaseUrl>,
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::num::NonZeroU8;

    use chrono::DateTime;
    use chrono::Utc;
    use futures::future::join_all;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use jwt::Jwt;
    use rustls_pki_types::TrustAnchor;
    use serde_json::json;

    use nl_wallet_mdoc::examples::example_items_requests;
    use nl_wallet_mdoc::examples::Example;
    use nl_wallet_mdoc::examples::IsoCertTimeGenerator;
    use nl_wallet_mdoc::server_keys::generate::Ca;
    use nl_wallet_mdoc::server_keys::KeyPair;
    use nl_wallet_mdoc::test::data::addr_street;
    use nl_wallet_mdoc::test::data::pid_full_name;
    use nl_wallet_mdoc::utils::serialization::cbor_serialize;
    use nl_wallet_mdoc::utils::serialization::CborBase64;
    use nl_wallet_mdoc::utils::serialization::CborSeq;
    use nl_wallet_mdoc::utils::serialization::TaggedBytes;
    use nl_wallet_mdoc::verifier::ItemsRequests;
    use nl_wallet_mdoc::DeviceAuthenticationKeyed;
    use nl_wallet_mdoc::DeviceResponse;
    use nl_wallet_mdoc::DeviceResponseVersion;
    use nl_wallet_mdoc::DeviceSigned;
    use nl_wallet_mdoc::Document;
    use nl_wallet_mdoc::IssuerSigned;
    use nl_wallet_mdoc::SessionTranscript;
    use poa::factory::PoaFactory;
    use poa::Poa;
    use wallet_common::generator::mock::MockTimeGenerator;
    use wallet_common::generator::Generator;
    use wallet_common::generator::TimeGenerator;
    use wallet_common::keys::examples::Examples;
    use wallet_common::keys::examples::EXAMPLE_KEY_IDENTIFIER;
    use wallet_common::keys::mock_remote::MockRemoteEcdsaKey;
    use wallet_common::keys::mock_remote::MockRemoteKeyFactory;
    use wallet_common::vec_at_least::VecAtLeastTwoUnique;

    use crate::openid4vp::AuthResponseError;
    use crate::openid4vp::IsoVpAuthorizationRequest;
    use crate::AuthorizationErrorCode;
    use crate::VpAuthorizationErrorCode;

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

    fn setup() -> (TrustAnchor<'static>, KeyPair, EcKeyPair, VpAuthorizationRequest) {
        setup_with_items_requests(&example_items_requests())
    }

    fn setup_with_items_requests(
        items_request: &ItemsRequests,
    ) -> (TrustAnchor<'static>, KeyPair, EcKeyPair, VpAuthorizationRequest) {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let trust_anchor = ca.to_trust_anchor().to_owned();
        let rp_keypair = ca.generate_reader_mock(None).unwrap();

        let encryption_privkey = EcKeyPair::generate(EcCurve::P256).unwrap();

        let auth_request = IsoVpAuthorizationRequest::new(
            items_request,
            rp_keypair.certificate(),
            "nonce".to_string(),
            encryption_privkey.to_jwk_public_key().try_into().unwrap(),
            "https://example.com/response_uri".parse().unwrap(),
            None,
        )
        .unwrap()
        .into();

        (trust_anchor, rp_keypair, encryption_privkey, auth_request)
    }

    #[test]
    fn test_encrypt_decrypt_authorization_response() {
        let (_, _, encryption_privkey, auth_request) = setup();

        // NB: the example DeviceResponse verifies as an ISO 18013-5 DeviceResponse while here we use it in
        // an OpenID4VP setting, i.e. with different SessionTranscript contents, so it can't be verified.
        // This is not an issue here because this code only deals with en/decrypting the DeviceResponse into
        // an Authorization Response JWE.
        let mdoc_nonce = "mdoc_nonce".to_string();
        let device_response = DeviceResponse::example();
        let auth_request = IsoVpAuthorizationRequest::try_from(auth_request).unwrap();
        // the ISO examples only use one Mdoc and therefore there is no need for a PoA
        let auth_response = VpAuthorizationResponse::new(device_response, &auth_request, None);
        let jwe = auth_response.encrypt(&auth_request, &mdoc_nonce).unwrap();

        let (decrypted, jwe_mdoc_nonce) =
            VpAuthorizationResponse::decrypt(&jwe, &encryption_privkey, &auth_request.nonce).unwrap();
        assert_eq!(mdoc_nonce, jwe_mdoc_nonce);

        let VerifiablePresentation::MsoMdoc(CborBase64(encrypted_device_response)) =
            auth_response.vp_token.first().unwrap();
        let VerifiablePresentation::MsoMdoc(CborBase64(decrypted_device_response)) =
            decrypted.vp_token.first().unwrap();
        let encrypted_document = encrypted_device_response.documents.as_ref().unwrap().first().unwrap();
        let decrypted_document = decrypted_device_response.documents.as_ref().unwrap().first().unwrap();

        assert_eq!(decrypted_document.doc_type, encrypted_document.doc_type);
        assert_eq!(decrypted_document.issuer_signed, encrypted_document.issuer_signed);
    }

    #[tokio::test]
    async fn test_authorization_request_jwt() {
        let (trust_anchor, rp_keypair, _, auth_request) = setup();

        let auth_request_jwt = Jwt::sign_with_certificate(&auth_request, &rp_keypair).await.unwrap();

        let (auth_request, cert) = VpAuthorizationRequest::try_new(&auth_request_jwt, &[trust_anchor]).unwrap();
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
                "presentation_definition": {
                    "id": "mDL-sample-req",
                    "input_descriptors": [{
                        "format": {
                            "mso_mdoc": {
                                "alg": [ "ES256" ]
                            }
                        },
                        "id": "org.iso.18013.5.1.mDL",
                        "constraints": {
                            "limit_disclosure": "required",
                            "fields": [
                                { "path": ["$['org.iso.18013.5.1']['family_name']"], "intent_to_retain": false },
                                { "path": ["$['org.iso.18013.5.1']['birth_date']"], "intent_to_retain": false },
                                { "path": ["$['org.iso.18013.5.1']['document_number']"], "intent_to_retain": false },
                                { "path": ["$['org.iso.18013.5.1']['driving_privileges']"], "intent_to_retain": false }
                            ]
                        }
                    }]
                }
            }
        );

        let auth_request: VpAuthorizationRequest = serde_json::from_value(example_json).unwrap();
        IsoVpAuthorizationRequest::try_from(auth_request).unwrap();
    }

    #[test]
    fn deserialize_authorization_response_example() {
        let example_json = json!(
            {
                "presentation_submission": {
                    "definition_id": "mDL-sample-req",
                    "id": "mDL-sample-res",
                    "descriptor_map": [
                        {
                            "id": "org.iso.18013.5.1.mDL",
                            "format": "mso_mdoc",
                            "path": "$"
                        }
                    ]
                },
                "vp_token":
                "o2d2ZXJzaW9uYzEuMGlkb2N1bWVudHOBo2dkb2NUeXBldW9yZy5pc28uMTgwMTMuNS4xLm1ETGxpc3N1ZXJTaWduZWSiam5hbWVTc\
                 GFjZXOhcW9yZy5pc28uMTgwMTMuNS4xi9gYWF-kaGRpZ2VzdElEGhU-n8JmcmFuZG9tUBhhBdaBj6yzbcAptxJFt5NxZWxlbWVudE\
                 lkZW50aWZpZXJqYmlydGhfZGF0ZWxlbGVtZW50VmFsdWXZA-xqMTk5MC0wMS0wMdgYWF-kaGRpZ2VzdElEGgGfQ2JmcmFuZG9tUD_\
                 vjxEDDiHVNPYQrc-z3qJxZWxlbWVudElkZW50aWZpZXJvZG9jdW1lbnRfbnVtYmVybGVsZW1lbnRWYWx1ZWhBQkNEMTIzNNgYWPOk\
                 aGRpZ2VzdElEGhYhPvdmcmFuZG9tUPeQCdM61nPIh-T2KdDLzJ9xZWxlbWVudElkZW50aWZpZXJyZHJpdmluZ19wcml2aWxlZ2Vzb\
                 GVsZW1lbnRWYWx1ZYKjamlzc3VlX2RhdGXZA-xqMjAyMC0wMS0wMWtleHBpcnlfZGF0ZdkD7GoyMDI1LTAxLTAxdXZlaGljbGVfY2\
                 F0ZWdvcnlfY29kZWFCo2ppc3N1ZV9kYXRl2QPsajIwMjAtMDEtMDFrZXhwaXJ5X2RhdGXZA-xqMjAyNS0wMS0wMXV2ZWhpY2xlX2N\
                 hdGVnb3J5X2NvZGViQkXYGFhgpGhkaWdlc3RJRBo23jMjZnJhbmRvbVBRkUqBtZ0-cdgL-Ah55BRHcWVsZW1lbnRJZGVudGlmaWVy\
                 a2V4cGlyeV9kYXRlbGVsZW1lbnRWYWx1ZdkD7GoyMDI1LTAxLTAx2BhYWKRoZGlnZXN0SUQaZYFFSmZyYW5kb21QdKpwyVh1BG0eg\
                 itavv8UWXFlbGVtZW50SWRlbnRpZmllcmtmYW1pbHlfbmFtZWxlbGVtZW50VmFsdWVlU21pdGjYGFhXpGhkaWdlc3RJRBoX9SvMZn\
                 JhbmRvbVBD8vu88PnK3lzRO9sRvnNDcWVsZW1lbnRJZGVudGlmaWVyamdpdmVuX25hbWVsZWxlbWVudFZhbHVlZUFsaWNl2BhYX6R\
                 oZGlnZXN0SUQaMaFJlmZyYW5kb21Q9AoSQ1BmYmKEqfADoeKDunFlbGVtZW50SWRlbnRpZmllcmppc3N1ZV9kYXRlbGVsZW1lbnRW\
                 YWx1ZdkD7GoyMDIwLTAxLTAx2BhYX6RoZGlnZXN0SUQaA8azMWZyYW5kb21Qb5Fu5qMeqndj9esMYWzh5XFlbGVtZW50SWRlbnRpZ\
                 mllcnFpc3N1aW5nX2F1dGhvcml0eWxlbGVtZW50VmFsdWVmTlksVVNB2BhYWaRoZGlnZXN0SUQaUUgWkmZyYW5kb21Qgh02uXoPCu\
                 F2NCY9MlUucHFlbGVtZW50SWRlbnRpZmllcm9pc3N1aW5nX2NvdW50cnlsZWxlbWVudFZhbHVlYlVT2BhZCD-kaGRpZ2VzdElEGmT\
                 XNGdmcmFuZG9tUE2OWXxsntQn-CrtHF_AfwVxZWxlbWVudElkZW50aWZpZXJocG9ydHJhaXRsZWxlbWVudFZhbHVlWQft_9j_4AAQ\
                 SkZJRgABAQAAAAAAAAD_4gIoSUNDX1BST0ZJTEUAAQEAAAIYAAAAAAQwAABtbnRyUkdCIFhZWiAAAAAAAAAAAAAAAABhY3NwAAAAA\
                 AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQAA9tYAAQAAAADTLQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAAAAAlkZXNjAAAA8AAAAHRyWFlaAAABZAAAABRnWFlaAAABeAAAABRiWFlaAAABjAAAABRyVFJDAAABoAAAAChnVFJ\
                 DAAABoAAAAChiVFJDAAABoAAAACh3dHB0AAAByAAAABRjcHJ0AAAB3AAAADxtbHVjAAAAAAAAAAEAAAAMZW5VUwAAAFgAAAAcAHMA\
                 UgBHAEIAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA\
                 AAAAAAAAAAAAFhZWiAAAAAAAABvogAAOPUAAAOQWFlaIAAAAAAAAGKZAAC3hQAAGNpYWVogAAAAAAAAJKAAAA-EAAC2z3BhcmEAAA\
                 AAAAQAAAACZmYAAPKnAAANWQAAE9AAAApbAAAAAAAAAABYWVogAAAAAAAA9tYAAQAAAADTLW1sdWMAAAAAAAAAAQAAAAxlblVTAAA\
                 AIAAAABwARwBvAG8AZwBsAGUAIABJAG4AYwAuACAAMgAwADEANv_bAEMAEAsMDgwKEA4NDhIREBMYKBoYFhYYMSMlHSg6Mz08OTM4\
                 N0BIXE5ARFdFNzhQbVFXX2JnaGc-TXF5cGR4XGVnY__bAEMBERISGBUYLxoaL2NCOEJjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY\
                 2NjY2NjY2NjY2NjY2NjY2NjY2NjY2NjY__AABEIALAAeQMBIgACEQEDEQH_xAAaAAADAQEBAQAAAAAAAAAAAAAAAwQFBgcB_8QALh\
                 AAAgIBAwIEBQMFAAAAAAAAAAMEEyMFFDNDUyRjc4MBBhU0oxZEkyU1UVWz_8QAFgEBAQEAAAAAAAAAAAAAAAAAAAME_8QAFhEBAQE\
                 AAAAAAAAAAAAAAAAAAAMT_9oADAMBAAIRAxEAPwDlwACSRoAAAAW1CrQGgS7pR93q_wDAFIEu6C0CoAAAAAAAAAFDRQ0AFNGksoBV\
                 trRTQAqqAUFQAN9VoWh6oW-UA1TRvSJbQbykheptoCopUEigAAAAAAFSmq4gktqUSxVWtAbFgNlNOji_LihsBVRsqDVkxm_LiukQf\
                 pyUdkNCuTklfK-LK0y5WjNitqaegks9VqqgZODlaW2Ll6QrpG9PVa2pXS5TLnq2srymhJBxFRK1RUriCQAACQAAAglcpfpaiCVymp\
                 ACsnRxVGoogil6g1GjQAkC2ogqbP8AKUX1Daiog2ClKqUo5zVIFVp2Rl6pFtUBwbcQKGz_ALpoqLyhlNAACQAAAllcpfFbUQSuUaF\
                 ZOoiz1VF6p6u6ckpqhtVqrVBXR2SpQ23KcbAa1rVKOtqtihVVaNtOXlNlKxWios9tuWUB1pK3KSqa1vV9oqy9UDz7VMUpqhUXiKte\
                 _ukr1RSsSgy1AAASAChoCmqL1RbVClelaakVVTagrJLFgeUbKqlRaqhtQqVxBqZass9SlNqOoqaqL9-33VHL6Mrx9p2XKqoCDaqbF\
                 81uUg-lqt6vpGzAVVFUoaBlxYFTcXEXtt5VKyjahTW1NVUB59PU36o23ujRs_K22olDLUAKAJFDRQ0Bqi-LKqaQDVBWTe3TcWK0ln\
                 taNi8QqVFtDUboLVW1HRnJRYErdYjqNq1qlZcoBbU1o1spSuVtXqjasQVEgq23iyg1XLbytKiCfPVAytbiKjktZbbqjf4iAbKlKbK\
                 a3utFBgKAAAUNFAA0aKADeit8KNVKtMuA3ul9SmhqbOl1Wl_dOXixVW27pqjZVU391b7oVag3lJVKq4mjbSQGnJfNsrxSldo6OVKq\
                 U1rekcHKlbqU2U3qlUqlCgAqygPdAUAAABU0GtFNbUErFiAqi8RswMpgwOIvU2okq62LAVUNVpaldUggT1VZTUVKV3QqbUDQ3XaBS\
                 iQy9exaW1rTiG8uI7L5jlKa3a9rK05dtTeWpRVKqACrYYrVNVUKapqiqRQAASXyorVKxKIKi_3be6StVb9r0gqlVyqKmqa1ouL8Pj\
                 8Pjj-GVnGbsWKptrWt4lAZcA2dgQKgNixVN7p2UVXhVElWDF0trTZi6MpRepVQ20KhSlKINU1RUBXmip-s7XErlOXlSmz222gakC1\
                 Wlyp_VbiMvWVKVFV0ml8ptUCLF7SvymNryqpSovaUEilKtxWim7qBibaoa1VSjU1SA1XErF_yKpMZSlNVlKtr5opTVKblVxF-6i_6\
                 v8oCmxdhFU3lt5SVsVsXpNOolRbZSoHutMaU1u6a3KBjZVZTeVPV9LUpTfFN5RWsRlJ0uL3eUlbF2qospXE0DZVlgbXqqOogfaqOD\
                 0ue3f5fyneK4sRJqkGmXPlYsRqN4jnJ8rqtJDLlBAV_V2yv2o1VvVVla3ENbK8BUr-JpVI1VUrVMrcSsrfVMGU23VGt801GxZWl6N\
                 l6pgxVNa3EVSXymq3SsuI3lSlSlYuXzTkpSmqlVHRq0tu1U0DLn2qnhaGsKlVKa1Rlkh__2dgYWGGkaGRpZ2VzdElEGnHvWL5mcmF\
                 uZG9tUPHruXHjv35Iu-rzOkKBD2xxZWxlbWVudElkZW50aWZpZXJ2dW5fZGlzdGluZ3Vpc2hpbmdfc2lnbmxlbGVtZW50VmFsdWVj\
                 VVNBamlzc3VlckF1dGiEQ6EBJqEYIVkCvjCCArowggJhoAMCAQICFA7VlOKXxKg_rJ6UiVqmXVQjpptXMAoGCCqGSM49BAMCMGAxC\
                 zAJBgNVBAYTAlVTMQswCQYDVQQIDAJOWTEZMBcGA1UECgwQSVNPbURMIFRlc3QgUm9vdDEpMCcGA1UEAwwgSVNPMTgwMTMtNSBUZX\
                 N0IENlcnRpZmljYXRlIFJvb3QwHhcNMjMxMTE0MTAwMTA1WhcNMjQxMTEzMTAwMTA1WjBrMQswCQYDVQQGEwJVUzELMAkGA1UECAw\
                 CTlkxIjAgBgNVBAoMGUlTT21ETCBUZXN0IElzc3VlciBTaWduZXIxKzApBgNVBAMMIklTTzE4MDEzLTUgVGVzdCBDZXJ0aWZpY2F0\
                 ZSBTaWduZXIwWTATBgcqhkjOPQIBBggqhkjOPQMBBwNCAAQgaK6KmQ1mWKt-Vo6ixfHxsmX9YlGAuUPkOvQ_uHrxgsZLC6FheRwtU\
                 3v-5GGkHD70FJNmz7DJUiR6G8TWMYZGo4HtMIHqMB0GA1UdDgQWBBQEpN0hSF6BFZJCDvZwASaa6ewoXzAfBgNVHSMEGDAWgBQ1Ro\
                 Oxz04dQvKPF76VhBf-jMv3EDAxBglghkgBhvhCAQ0EJBYiSVNPMTgwMTMtNSBUZXN0IFNpZ25lciBDZXJ0aWZpY2F0ZTAOBgNVHQ8\
                 BAf8EBAMCB4AwFQYDVR0lAQH_BAswCQYHKIGMXQUBAjAdBgNVHRIEFjAUgRJleGFtcGxlQGlzb21kbC5jb20wLwYDVR0fBCgwJjAk\
                 oCKgIIYeaHR0cHM6Ly9leGFtcGxlLmNvbS9JU09tREwuY3JsMAoGCCqGSM49BAMCA0cAMEQCIGV5CQ0EFGjFzVBSqWfaPVUMziesc\
                 VQ4W-lxw5bq7nCBAiBf1D9SPeA05Sdf0iWHanW3N0FBtS7Iz5XdSKWT2IqMKFkFzNgYWQXHpmd2ZXJzaW9uYzEuMG9kaWdlc3RBbG\
                 dvcml0aG1nU0hBLTI1Nmx2YWx1ZURpZ2VzdHOhcW9yZy5pc28uMTgwMTMuNS4xuB4aAZ9DYlggxtL2LRFm_GWjYft8lw02WZS4CLb\
                 Chc-NakfbNNyTuAcaA8azMVggpZCl5WRduZFAMb0vykZCxA-AdeM1R8eoiC1d9d3pkLUaEU3f71ggHp0cG-ZNraR6vcOgLZSORP9r\
                 DEipOlXzHh18YamiANoaFT6fwlggEphIDgQUoblEdUCq34aMJ50OQ8QmCVFJuQgiB1_YwhoaFiE-91ggeEtCLmPzCOD0suxhDwW43\
                 s7yNc1x6Jd4DZ6tO4ObD6IaFxmGZVgguLSRAKQ1dwJGOS_soukGSZUkCqvW7zN4R4eoTO-phFAaF_UrzFggJ4LaSDWgaeLF8hkPWL\
                 aDZGrjYCOuLYCcZk5tWXug4aMaGthQz1ggg3jAWBkkHRE9l4AoDdqdFYgJ56crlzeAf47JtJ662VMaKXLy5lggQBw9uLpXYKFP4Zd\
                 oO7zzb_vtymKttmA_qeaaEW_jbWUaLBiyUVggR3AnIGXnmMTMHPjuR19-e4lLs6vi3digyHN0iyzc3PkaMQCGYVggKvs-nJVYFRwH\
                 _TbFGPy1X_t69MR1l94toRIIK98UBvAaMaFJllggCoQYDBIY_rk6s0MhMbC8ibfzGegfY-Pfwauy9GHW_38aMqbHz1ggtrjS6GQsM\
                 tSQaKf7Voa6kxPLDqK24EZ-9WhB8JaO4f4aNUVesFggOzxV3ZrJ8FQMCuThmR6L7B4SMN5bxLy05i6v9wgpejYaNt4zI1ggzzpJiJ\
                 uTxtgMDAxycYe7TYmsovh5Aw_EDfDX8rRqYV4aOTYKk1ggEnnTJHhcMseTjVtPRGnCRRCE0WvwelO1dECZvlLXeIQaSkPcgFggyKs\
                 6jeFkCIjai1k5xYqZyqjK45ImuVOzPVC8jPXPXksaT9EOg1ggO9johmBdbxTYTcMQDSB1K9jwdd350VIjMuCHDZ8DDUEaUUgWklgg\
                 Gz_ddBP3N_0mxg7fco-oJ2HorIFAptTj78ZseE5gfmAaUUqijlgglmqfTA5d9Wi85wDcpdTO1NrlH02nOx4zP7FZ8TE7KroaXagLO\
                 Fggt3m7aHSfgJ3rEl1nn-Pp8YitK572a64L2GAa-UZCiGkaXyMT31gghLUPLnlPsZaDTk7Yd_JsjeEuWwIeAyu5FNeYRDkajlUaYL\
                 UtAlggb0He6jVXV1OGqzZidHIWpba3yCffluLpOiKAiVzVeXEaZNc0Z1gggSJJmw3Dt0YsH38Flq1hxybrP4tWRR6nSUUPZOah6BU\
                 aZYFFSlgg9-83mx19kFY-tbUDXndIdXL1oXs_2nYgchpnvWuENjIaaMXpU1gg5iCCWrzVNvXZa8I_jfmTpwZFxdmEfv7D3rcYhza6\
                 5Dcaa4ST6Vggrz6sluAmWjOaVRmKC6lLFyqEglIyTwZlGA3Q6tbF_WcacYeIV1ggk1fOHKjmMaPlh7SaVtWk-6jC38DqPcWz3UcH6\
                 xuiZ1Mace9YvlggJCOAb023GTSEcNt48NYF2ZOM2p9RIqO8W91zORSzMFQaecXSi1ggFZp2VG3VaCEgchdZPgbK8JSuYsMCmy9Dy4\
                 WXyZD1Z1RtZGV2aWNlS2V5SW5mb6FpZGV2aWNlS2V5pAECIAEhWCCS9P-a8TB2KJzTBif0C32CrhjX3XKMVykLFFTHdFXpnSJYIAD\
                 ctY3zP9kjfSptLs9kyUhDUDRf4xOSIs0FkbyjHnsFZ2RvY1R5cGV1b3JnLmlzby4xODAxMy41LjEubURMbHZhbGlkaXR5SW5mb6Nm\
                 c2lnbmVkwHQyMDIzLTExLTE2VDA5OjI1OjIyWml2YWxpZEZyb23AdDIwMjMtMTEtMTZUMDk6MjU6MjJaanZhbGlkVW50aWzAdDIwM\
                 jMtMTItMTZUMDk6MjU6MjJaWEC8OJJedu29mak8hVi1X__VJhpQ6QhgOhTqHZMtdrqyWdalv457ykvXnq3U5Zl5NC1GDyIDdr23_L\
                 67HUOKFqCHbGRldmljZVNpZ25lZKJqbmFtZVNwYWNlc9gYQaBqZGV2aWNlQXV0aKFvZGV2aWNlU2lnbmF0dXJlhEOhASag9lhAX4O\
                 7CImR03EijrZDHYgdzQefwdix5l-hJ7ow05OvOyQj0f_kW9GYbvWbDYbHN_kreXHaXpDh5Swm1nc5X39N6mZzdGF0dXMA"
            }
        );

        let auth_response: VpAuthorizationResponse = serde_json::from_value(example_json).unwrap();

        let VerifiablePresentation::MsoMdoc(CborBase64(decrypted_device_response)) =
            auth_response.vp_token.first().unwrap();
        let decrypted_document = decrypted_device_response.documents.as_ref().unwrap().first().unwrap();
        assert_eq!(decrypted_document.doc_type, "org.iso.18013.5.1.mDL".to_string());
    }

    fn challenges(auth_request: &IsoVpAuthorizationRequest, session_transcript: &SessionTranscript) -> Vec<Vec<u8>> {
        auth_request
            .items_requests
            .0
            .iter()
            .map(|it| {
                // Assemble the challenge (serialized Device Authentication) to sign with the mdoc key
                let device_authentication = TaggedBytes(CborSeq(DeviceAuthenticationKeyed {
                    device_authentication: Default::default(),
                    session_transcript: Cow::Borrowed(session_transcript),
                    doc_type: Cow::Borrowed(&it.doc_type),
                    device_name_spaces_bytes: IndexMap::new().into(),
                }));

                cbor_serialize(&device_authentication).unwrap()
            })
            .collect_vec()
    }

    /// Build a valid `DeviceResponse` using the given `issuer_signed`, `device_signed` and `session_transcript`.
    fn device_response(
        issuer_signed: Vec<IssuerSigned>,
        device_signed: Vec<DeviceSigned>,
        session_transcript: &SessionTranscript,
        cas: &[TrustAnchor],
        time: &impl Generator<DateTime<Utc>>,
    ) -> DeviceResponse {
        let documents = Some(
            issuer_signed
                .into_iter()
                .zip(device_signed)
                .map(|(issuer_signed, device_signed)| {
                    let doc_type = issuer_signed.issuer_auth.doc_type().unwrap();

                    Document {
                        doc_type,
                        issuer_signed,
                        device_signed,
                        errors: None,
                    }
                })
                .collect(),
        );

        let device_response = DeviceResponse {
            version: DeviceResponseVersion::V1_0,
            documents,
            document_errors: None,
            status: 0,
        };
        device_response.verify(None, session_transcript, time, cas).unwrap();

        device_response
    }

    /// Helper function to use `setup_device_response`, given the examples from ISO 18013-5, resigned with a newly
    /// generated CA.
    async fn example_device_response(
        auth_request: &IsoVpAuthorizationRequest,
        mdoc_nonce: &str,
        ca: &Ca,
    ) -> (DeviceResponse, Option<Poa>) {
        let issuer_signed_and_keys = DeviceResponse::example_resigned(ca)
            .await
            .documents
            .unwrap()
            .iter()
            .map(|d| {
                (
                    d.issuer_signed.clone(),
                    MockRemoteEcdsaKey::new(EXAMPLE_KEY_IDENTIFIER.to_owned(), Examples::static_device_key()),
                )
            })
            .collect_vec();

        setup_device_response(
            auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            &[ca.to_trust_anchor()],
            &IsoCertTimeGenerator,
        )
        .await
    }

    /// Manually construct a mock `DeviceResponse` and PoA using the given `auth_request`, `issuer_signed` and
    /// `mdoc_nonce`.
    async fn setup_device_response(
        auth_request: &IsoVpAuthorizationRequest,
        mdoc_nonce: &str,
        issuer_signed_and_keys: &[(IssuerSigned, MockRemoteEcdsaKey)],
        cas: &[TrustAnchor<'_>],
        time: &impl Generator<DateTime<Utc>>,
    ) -> (DeviceResponse, Option<Poa>) {
        let session_transcript = SessionTranscript::new_oid4vp(
            &auth_request.response_uri,
            &auth_request.client_id,
            auth_request.nonce.clone(),
            mdoc_nonce,
        );

        let (issuer_signed, keys): (_, Vec<MockRemoteEcdsaKey>) =
            issuer_signed_and_keys.iter().map(ToOwned::to_owned).unzip();

        let challenges = challenges(auth_request, &session_transcript);
        let keys_and_challenges = challenges
            .iter()
            .zip(&keys)
            .map(|(c, k)| (k.to_owned(), c.as_ref()))
            .collect_vec();

        // Sign the challenges using the mdoc key
        let key_factory = MockRemoteKeyFactory::default();
        let device_signed = DeviceSigned::new_signatures(keys_and_challenges, &key_factory)
            .await
            .unwrap()
            .0;

        let device_response = device_response(issuer_signed, device_signed, &session_transcript, cas, time);

        let poa = match VecAtLeastTwoUnique::try_from(keys) {
            Ok(keys) => {
                let keys = keys.as_slice().iter().collect_vec().try_into().unwrap();
                let poa = key_factory
                    .poa(keys, auth_request.client_id.clone(), Some(mdoc_nonce.to_owned()))
                    .await
                    .unwrap();
                Some(poa)
            }
            Err(_) => None,
        };

        (device_response, poa)
    }

    #[tokio::test]
    async fn test_verify_authorization_response() {
        let (_, _, _, auth_request) = setup();
        let mdoc_nonce = "mdoc_nonce";

        let auth_request = IsoVpAuthorizationRequest::try_from(auth_request).unwrap();
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let (device_response, poa) = example_device_response(&auth_request, mdoc_nonce, &ca).await;
        let auth_response = VpAuthorizationResponse::new(device_response, &auth_request, poa);

        auth_response
            .verify(
                &auth_request,
                mdoc_nonce,
                &IsoCertTimeGenerator,
                &[ca.to_trust_anchor()],
            )
            .unwrap();
    }

    async fn setup_poa_test(ca: &Ca) -> (Vec<(IssuerSigned, MockRemoteEcdsaKey)>, IsoVpAuthorizationRequest) {
        let stored_documents = pid_full_name() + addr_street();
        let items_request = stored_documents.clone().into();

        let (_, _, _, auth_request) = setup_with_items_requests(&items_request);

        let auth_request = IsoVpAuthorizationRequest::try_from(auth_request).unwrap();

        let key_factory = MockRemoteKeyFactory::default();
        let issuer_signed_and_keys = join_all(
            stored_documents
                .into_iter()
                .map(|doc| doc.issuer_signed(ca, &key_factory, NonZeroU8::new(10).unwrap())),
        )
        .await;

        (issuer_signed_and_keys, auth_request)
    }

    #[tokio::test]
    async fn test_verify_poa() {
        let mdoc_nonce = "mdoc_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];
        let (issuer_signed_and_keys, auth_request) = setup_poa_test(&ca).await;
        let (device_response, poa) = setup_device_response(
            &auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            trust_anchors,
            &MockTimeGenerator::default(),
        )
        .await;

        let auth_response = VpAuthorizationResponse::new(device_response, &auth_request, poa);
        auth_response
            .verify(&auth_request, mdoc_nonce, &TimeGenerator, trust_anchors)
            .unwrap();
    }

    #[tokio::test]
    async fn test_verify_missing_poa() {
        let mdoc_nonce = "mdoc_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];
        let (issuer_signed_and_keys, auth_request) = setup_poa_test(&ca).await;
        let (device_response, _) = setup_device_response(
            &auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            trust_anchors,
            &MockTimeGenerator::default(),
        )
        .await;

        let auth_response = VpAuthorizationResponse::new(device_response, &auth_request, None);
        let error = auth_response
            .verify(&auth_request, mdoc_nonce, &TimeGenerator, trust_anchors)
            .expect_err("should fail due to missing PoA");
        assert!(matches!(error, AuthResponseError::MissingPoa));
    }

    #[tokio::test]
    async fn test_verify_invalid_poa() {
        let mdoc_nonce = "mdoc_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];
        let (issuer_signed_and_keys, auth_request) = setup_poa_test(&ca).await;
        let (device_response, poa) = setup_device_response(
            &auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            trust_anchors,
            &MockTimeGenerator::default(),
        )
        .await;

        let mut poa = poa.unwrap();
        poa.set_payload("edited".to_owned());

        let auth_response = VpAuthorizationResponse::new(device_response, &auth_request, Some(poa));
        let error = auth_response
            .verify(&auth_request, mdoc_nonce, &TimeGenerator, trust_anchors)
            .expect_err("should fail due to missing PoA");
        assert!(matches!(error, AuthResponseError::PoaVerification(_)));
    }
}
