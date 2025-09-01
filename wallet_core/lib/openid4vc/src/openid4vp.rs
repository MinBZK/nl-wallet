use std::cell::LazyCell;
use std::collections::HashMap;
use std::string::FromUtf8Error;

use chrono::DateTime;
use chrono::Utc;
use derive_more::Constructor;
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
use serde::Serialize;
use serde_with::DeserializeFromStr;
use serde_with::skip_serializing_none;

use attestation_data::disclosure::DisclosedAttestation;
use attestation_data::disclosure::DisclosedAttestationError;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use dcql::CredentialQueryIdentifier;
use dcql::Query;
use dcql::disclosure::CredentialValidationError;
use dcql::disclosure::DisclosedCredential;
use dcql::normalized::NormalizedCredentialRequests;
use dcql::normalized::UnsupportedDcqlFeatures;
use error_category::ErrorCategory;
use http_utils::urls::BaseUrl;
use jwt::UnverifiedJwt;
use jwt::error::JwtX5cError;
use mdoc::DeviceResponse;
use mdoc::Document;
use mdoc::SessionTranscript;
use mdoc::errors::Error as MdocError;
use mdoc::utils::serialization::CborBase64;
use serde_with::SerializeDisplay;
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

impl VpAuthorizationRequest {
    /// Construct a new Authorization Request by verifying an Authorization Request JWT against
    /// the specified trust anchors.
    pub fn try_new(
        jws: &UnverifiedJwt<VpAuthorizationRequest>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<(VpAuthorizationRequest, BorrowingCertificate), AuthRequestValidationError> {
        let (auth_request, certificates) = jws.verify_against_trust_anchors_and_audience(
            &[VpAuthorizationRequestAudience::SelfIssued],
            trust_anchors,
            &TimeGenerator,
        )?;

        Ok((auth_request, certificates.into_first()))
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
    #[error("apv (nonce) field in JWE had incorrect value")]
    NonceIncorrect,
    #[error("state had incorrect value: expected {expected:?}, found {found:?}")]
    StateIncorrect {
        expected: Option<String>,
        found: Option<String>,
    },
    #[error("missing apu field from JWE")]
    MissingApu,
    #[error("missing apv field from JWE")]
    MissingApv,
    #[error("failed to decode apu/apv field from JWE")]
    Utf8(#[from] FromUtf8Error),
    #[error("no Document received in any DeviceResponse for credential query identifier: {0}")]
    #[category(pd)]
    NoMdocDocuments(CredentialQueryIdentifier),
    #[error("error verifying disclosed mdoc(s): {0}")]
    MdocVerification(#[source] mdoc::Error),
    #[error("response does not satisfy credential request(s): {0}")]
    UnsatisfiedCredentialRequest(#[source] CredentialValidationError),
    #[error("missing PoA")]
    MissingPoa,
    #[error("error verifying PoA: {0}")]
    PoaVerification(#[from] PoaVerificationError),
    #[error("error converting disclosed attestations: {0}")]
    #[category(pd)]
    DisclosedAttestation(#[source] DisclosedAttestationError),
}

/// Disclosure of a credential, generally containing the issuer-signed credential itself, the disclosed attributes,
/// and a holder signature over some nonce provided by the verifier.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum VerifiablePresentation {
    // NB: a `DeviceResponse` can contain disclosures of multiple mdocs. In case of other (not yet supported) formats,
    //     each credential is expected to result in a separate Verifiable Presentation.
    MsoMdoc(VecNonEmpty<CborBase64<DeviceResponse>>),
}

impl VerifiablePresentation {
    pub fn new_mdoc(device_responses: VecNonEmpty<DeviceResponse>) -> Self {
        let device_responses = device_responses.into_nonempty_iter().map(CborBase64).collect();

        Self::MsoMdoc(device_responses)
    }

    /// Helper function for iterating over all mdoc [`Document`]s in a list of encoded [`DeviceResponse`]s.
    fn mdoc_documents_iter<'a>(
        device_responses: impl IntoIterator<Item = &'a CborBase64<DeviceResponse>>,
    ) -> impl Iterator<Item = &'a Document> {
        device_responses
            .into_iter()
            .flat_map(|CborBase64(device_response)| device_response.documents.as_deref().unwrap_or_default())
    }

    /// Helper function for extracing all unique public keys in a set of [`VerifiablePresentation`]s.
    fn unique_keys<'a>(presentations: impl IntoIterator<Item = &'a Self>) -> Result<Vec<VerifyingKey>, MdocError> {
        presentations
            .into_iter()
            .flat_map(|presentation| match presentation {
                Self::MsoMdoc(device_responses) => {
                    Self::mdoc_documents_iter(device_responses).map(|document| document.issuer_signed.public_key())
                }
            })
            // Unfortunately `VerifyingKey` does not implement `Hash`, so we have to deduplicate manually.
            .process_results(|iter| iter.dedup().collect())
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
        mdoc_nonce: &str,
        poa: Option<Poa>,
    ) -> Result<String, AuthResponseError> {
        Self::new(vp_token, auth_request.state.clone(), poa).encrypt(auth_request, mdoc_nonce)
    }

    fn encrypt(
        &self,
        auth_request: &NormalizedVpAuthorizationRequest,
        mdoc_nonce: &str,
    ) -> Result<String, AuthResponseError> {
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
        auth_request: &NormalizedVpAuthorizationRequest,
        accepted_wallet_client_ids: &[String],
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<HashMap<CredentialQueryIdentifier, VecNonEmpty<DisclosedAttestation>>, AuthResponseError> {
        let (response, mdoc_nonce) = Self::decrypt(jwe, private_key, &auth_request.nonce)?;

        response.verify(
            auth_request,
            accepted_wallet_client_ids,
            &mdoc_nonce,
            time,
            trust_anchors,
        )
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

    pub fn verify(
        self,
        auth_request: &NormalizedVpAuthorizationRequest,
        accepted_wallet_client_ids: &[String],
        mdoc_nonce: &str,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<HashMap<CredentialQueryIdentifier, VecNonEmpty<DisclosedAttestation>>, AuthResponseError> {
        // Step 1: Verify the cryptographic integrity of the verifiable presentations
        //         and extract the disclosed attestations from them.
        let session_transcript = LazyCell::new(|| {
            // The mdoc `SessionTranscript` may not be required, so initialize it lazily.
            SessionTranscript::new_oid4vp(
                &auth_request.response_uri,
                &auth_request.client_id,
                auth_request.nonce.clone(),
                mdoc_nonce,
            )
        });

        let disclosed_attestations = self
            .vp_token
            .iter()
            .map(|(id, presentation)| {
                let attestations = match presentation {
                    VerifiablePresentation::MsoMdoc(device_responses) => device_responses
                        .iter()
                        .map(|CborBase64(device_response)| {
                            // Verify the cryptographic integrity of each mdoc `DeviceResponse`
                            // and extract its disclosed documents...
                            let disclosed_documents = device_response
                                .verify(None, &session_transcript, time, trust_anchors)
                                .map_err(AuthResponseError::MdocVerification)?;

                            // ...then attempt to convert them to `DisclosedAttestation`s.
                            let disclosed_attestations = disclosed_documents
                                .into_iter()
                                .map(|disclosed_document| {
                                    DisclosedAttestation::try_from(disclosed_document)
                                        .map_err(AuthResponseError::DisclosedAttestation)
                                })
                                .collect::<Result<Vec<_>, _>>()?;

                            Ok(disclosed_attestations)
                        })
                        // Simply concatentate all `DisclosedAttestation`s resulting from all `DeviceResponse`s.
                        // This prevents us from having to assume that all `DeviceResponse`s
                        // only contain a single credential.
                        .process_results::<_, _, AuthResponseError, _>(|iter| iter.flatten().collect_vec())?
                        .try_into()
                        .map_err(|_| AuthResponseError::NoMdocDocuments(id.clone()))?,
                };

                Ok((id.clone(), attestations))
            })
            .collect::<Result<_, AuthResponseError>>()?;

        // Step 2: Verify the PoA, if present.
        let used_keys = VerifiablePresentation::unique_keys(self.vp_token.values())
            .expect("should always succeed when called after DeviceResponse::verify");
        if used_keys.len() >= 2 {
            self.poa.ok_or(AuthResponseError::MissingPoa)?.verify(
                &used_keys,
                auth_request.client_id.as_str(),
                accepted_wallet_client_ids,
                mdoc_nonce,
            )?
        }

        // Step 3: Verify the `state` field, against that of the Authorization Request.
        if self.state != auth_request.state {
            return Err(AuthResponseError::StateIncorrect {
                expected: auth_request.state.clone(),
                found: self.state.clone(),
            });
        }

        // Step 4: Check that we received all the attributes that we requested.
        let disclosed_credentials = self
            .vp_token
            .iter()
            .map(|(id, presentation)| {
                let credentials = match presentation {
                    VerifiablePresentation::MsoMdoc(device_responses) => {
                        VerifiablePresentation::mdoc_documents_iter(device_responses)
                            .map(DisclosedCredential::MsoMdoc)
                            .collect_vec()
                            .try_into()
                            .expect("should always succeed when called after DeviceResponse::verify")
                    }
                };

                (id, credentials)
            })
            .collect();

        auth_request
            .credential_requests
            .is_satisfied_by_disclosed_credentials(&disclosed_credentials)
            .map_err(AuthResponseError::UnsatisfiedCredentialRequest)?;

        Ok(disclosed_attestations)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpResponse {
    pub redirect_uri: Option<BaseUrl>,
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;
    use std::collections::HashMap;

    use chrono::DateTime;
    use chrono::Utc;
    use futures::future::join_all;
    use indexmap::IndexMap;
    use itertools::Itertools;
    use josekit::jwk::alg::ec::EcCurve;
    use josekit::jwk::alg::ec::EcKeyPair;
    use p256::ecdsa::SigningKey;
    use rand_core::OsRng;
    use rustls_pki_types::TrustAnchor;
    use serde_json::json;

    use attestation_data::disclosure::DisclosedAttributes;
    use attestation_data::x509::generate::mock::generate_reader_mock;
    use crypto::mock_remote::MockRemoteEcdsaKey;
    use crypto::server_keys::KeyPair;
    use crypto::server_keys::generate::Ca;
    use dcql::CredentialQueryIdentifier;
    use dcql::normalized::FormatCredentialRequest;
    use dcql::normalized::NormalizedCredentialRequests;
    use jwt::UnverifiedJwt;
    use mdoc::DeviceAuthenticationKeyed;
    use mdoc::DeviceResponse;
    use mdoc::DeviceSigned;
    use mdoc::Document;
    use mdoc::IssuerSigned;
    use mdoc::SessionTranscript;
    use mdoc::examples::Example;
    use mdoc::holder::Mdoc;
    use mdoc::test::data::addr_street;
    use mdoc::test::data::pid_full_name;
    use mdoc::utils::serialization::CborBase64;
    use mdoc::utils::serialization::CborSeq;
    use mdoc::utils::serialization::TaggedBytes;
    use mdoc::utils::serialization::cbor_serialize;
    use utils::generator::Generator;
    use utils::generator::TimeGenerator;
    use utils::generator::mock::MockTimeGenerator;
    use utils::vec_nonempty;
    use wscd::Poa;
    use wscd::mock_remote::MockRemoteWscd;
    use wscd::wscd::JwtPoaInput;

    use crate::AuthorizationErrorCode;
    use crate::VpAuthorizationErrorCode;
    use crate::mock::MOCK_WALLET_CLIENT_ID;
    use crate::mock::test_document_to_issuer_signed;
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

    fn setup() -> (TrustAnchor<'static>, KeyPair, EcKeyPair, VpAuthorizationRequest) {
        setup_with_credential_requests(NormalizedCredentialRequests::new_mock_mdoc_iso_example())
    }

    fn setup_with_credential_requests(
        credential_requests: NormalizedCredentialRequests,
    ) -> (TrustAnchor<'static>, KeyPair, EcKeyPair, VpAuthorizationRequest) {
        let ca = Ca::generate("myca", Default::default()).unwrap();
        let trust_anchor = ca.to_trust_anchor().to_owned();
        let rp_keypair = generate_reader_mock(&ca, None).unwrap();

        let encryption_privkey = EcKeyPair::generate(EcCurve::P256).unwrap();

        let auth_request = NormalizedVpAuthorizationRequest::new(
            credential_requests,
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
        let auth_request = NormalizedVpAuthorizationRequest::try_from(auth_request).unwrap();
        // the ISO examples only use one Mdoc and therefore there is no need for a PoA
        let auth_response = VpAuthorizationResponse::new(
            HashMap::from([(
                "id".try_into().unwrap(),
                VerifiablePresentation::new_mdoc(vec_nonempty![device_response]),
            )]),
            None,
            None,
        );
        let jwe = auth_response.encrypt(&auth_request, &mdoc_nonce).unwrap();

        let (decrypted, jwe_mdoc_nonce) =
            VpAuthorizationResponse::decrypt(&jwe, &encryption_privkey, &auth_request.nonce).unwrap();

        assert_eq!(mdoc_nonce, jwe_mdoc_nonce);
        assert_eq!(decrypted.vp_token.len(), 1);

        let (encrypted_identifier, VerifiablePresentation::MsoMdoc(encrypted_device_responses)) =
            auth_response.vp_token.into_iter().next().unwrap();
        let (decrypted_identifier, VerifiablePresentation::MsoMdoc(decrypted_device_responses)) =
            decrypted.vp_token.into_iter().next().unwrap();

        assert_eq!(encrypted_identifier, decrypted_identifier);
        assert_eq!(decrypted_device_responses.len().get(), 1);

        let CborBase64(encrypted_device_response) = encrypted_device_responses.into_first();
        let CborBase64(decrypted_device_response) = decrypted_device_responses.into_first();

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

    #[tokio::test]
    async fn test_authorization_request_jwt() {
        let (trust_anchor, rp_keypair, _, auth_request) = setup();

        let auth_request_jwt = UnverifiedJwt::sign_with_certificate(&auth_request, &rp_keypair)
            .await
            .unwrap();

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
    fn deserialize_authorization_response_example() {
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

        let VerifiablePresentation::MsoMdoc(decrypted_device_responses) =
            auth_response.vp_token.into_values().next().unwrap();

        assert_eq!(decrypted_device_responses.len().get(), 1);

        let CborBase64(decrypted_device_response) = decrypted_device_responses.into_first();

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

    fn challenges(
        auth_request: &NormalizedVpAuthorizationRequest,
        session_transcript: &SessionTranscript,
    ) -> Vec<Vec<u8>> {
        auth_request
            .credential_requests
            .as_ref()
            .iter()
            .map(|request| {
                match &request.format_request {
                    FormatCredentialRequest::MsoMdoc { doctype_value, .. } => {
                        // Assemble the challenge (serialized Device Authentication) to sign with the mdoc key
                        let device_authentication = TaggedBytes(CborSeq(DeviceAuthenticationKeyed {
                            device_authentication: Default::default(),
                            session_transcript: Cow::Borrowed(session_transcript),
                            doc_type: Cow::Borrowed(doctype_value),
                            device_name_spaces_bytes: IndexMap::new().into(),
                        }));

                        cbor_serialize(&device_authentication).unwrap()
                    }
                    FormatCredentialRequest::SdJwt { .. } => todo!("PVW-4139 support SdJwt"),
                }
            })
            .collect_vec()
    }

    /// Build a valid `DeviceResponse` using the given `issuer_signed`, `device_signed` and `session_transcript`.
    fn device_responses(
        issuer_signed: Vec<IssuerSigned>,
        device_signed: Vec<DeviceSigned>,
        session_transcript: &SessionTranscript,
        cas: &[TrustAnchor],
        time: &impl Generator<DateTime<Utc>>,
    ) -> Vec<DeviceResponse> {
        let documents = issuer_signed
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
            .collect_vec();

        let device_responses = documents
            .into_iter()
            .map(|document| DeviceResponse::new(vec![document]))
            .collect_vec();

        for device_response in &device_responses {
            device_response
                .verify(None, session_transcript, time, cas)
                .expect("created DeviceResponse should be valid");
        }

        device_responses
    }

    /// Manually construct mock `VerifiablePresentation`s and a PoA using
    /// the given `auth_request`, `issuer_signed` and `mdoc_nonce`.
    async fn setup_vp_token(
        auth_request: &NormalizedVpAuthorizationRequest,
        mdoc_nonce: &str,
        issuer_signed_and_keys: &[(IssuerSigned, MockRemoteEcdsaKey)],
        cas: &[TrustAnchor<'_>],
        time: &impl Generator<DateTime<Utc>>,
    ) -> (HashMap<CredentialQueryIdentifier, VerifiablePresentation>, Option<Poa>) {
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
        let wscd = MockRemoteWscd::default();
        let poa_input = JwtPoaInput::new(Some(mdoc_nonce.to_string()), auth_request.client_id.clone());
        let (device_signed, poa) = DeviceSigned::new_signatures(keys_and_challenges, &wscd, poa_input)
            .await
            .unwrap();

        let device_responses = device_responses(issuer_signed, device_signed, &session_transcript, cas, time);

        let vp_token = auth_request
            .credential_requests
            .as_ref()
            .iter()
            .zip_eq(device_responses)
            .map(|(request, device_response)| {
                (
                    request.id.clone(),
                    VerifiablePresentation::new_mdoc(vec_nonempty![device_response]),
                )
            })
            .collect();

        (vp_token, poa)
    }

    #[tokio::test]
    async fn test_verify_authorization_response() {
        let (_, _, _, auth_request) =
            setup_with_credential_requests(NormalizedCredentialRequests::new_mock_mdoc_pid_example());
        let mdoc_nonce = "mdoc_nonce";

        let time_generator = MockTimeGenerator::default();
        let auth_request = NormalizedVpAuthorizationRequest::try_from(auth_request).unwrap();
        let ca = Ca::generate_issuer_mock_ca().unwrap();

        let mdoc_key = MockRemoteEcdsaKey::new(String::from("mdoc_key"), SigningKey::random(&mut OsRng));
        let mdoc = Mdoc::new_mock_with_ca_and_key(&ca, &mdoc_key).await;

        let (vp_token, poa) = setup_vp_token(
            &auth_request,
            mdoc_nonce,
            &vec![(mdoc.issuer_signed, mdoc_key)],
            &[ca.to_trust_anchor()],
            &time_generator,
        )
        .await;

        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);

        let attestations = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                mdoc_nonce,
                &time_generator,
                &[ca.to_trust_anchor()],
            )
            .unwrap();

        assert_eq!(attestations.len(), 1);

        let disclosed_attestations = attestations.into_values().next().unwrap();

        assert_eq!(disclosed_attestations.len().get(), 1);

        let attestation = disclosed_attestations.into_first();

        assert_eq!("urn:eudi:pid:nl:1", attestation.attestation_type.as_str());
        let DisclosedAttributes::MsoMdoc(attributes) = &attestation.attributes else {
            panic!("should be mdoc attributes")
        };

        let (namespace, mdoc_attributes) = &attributes.first().unwrap();
        assert_eq!("urn:eudi:pid:nl:1", namespace.as_str());

        assert_eq!(
            vec!["bsn", "given_name", "family_name"],
            mdoc_attributes.keys().map(|key| key.as_str()).collect_vec()
        );
    }

    async fn setup_poa_test(
        ca: &Ca,
    ) -> (
        Vec<(IssuerSigned, MockRemoteEcdsaKey)>,
        NormalizedVpAuthorizationRequest,
    ) {
        let stored_documents = pid_full_name() + addr_street();
        let credential_requests = stored_documents.clone().into();

        let (_, _, _, auth_request) = setup_with_credential_requests(credential_requests);

        let auth_request = NormalizedVpAuthorizationRequest::try_from(auth_request).unwrap();

        let wscd = MockRemoteWscd::default();
        let issuer_signed_and_keys = join_all(
            stored_documents
                .into_iter()
                .map(|doc| test_document_to_issuer_signed(doc, ca, &wscd)),
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
        let (vp_token, poa) = setup_vp_token(
            &auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            trust_anchors,
            &MockTimeGenerator::default(),
        )
        .await;

        let auth_response = VpAuthorizationResponse::new(vp_token, None, poa);
        auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                mdoc_nonce,
                &TimeGenerator,
                trust_anchors,
            )
            .unwrap();
    }

    #[tokio::test]
    async fn test_verify_missing_poa() {
        let mdoc_nonce = "mdoc_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];
        let (issuer_signed_and_keys, auth_request) = setup_poa_test(&ca).await;
        let (vp_token, _) = setup_vp_token(
            &auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            trust_anchors,
            &MockTimeGenerator::default(),
        )
        .await;

        let auth_response = VpAuthorizationResponse::new(vp_token, None, None);
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                mdoc_nonce,
                &TimeGenerator,
                trust_anchors,
            )
            .expect_err("should fail due to missing PoA");
        assert!(matches!(error, AuthResponseError::MissingPoa));
    }

    #[tokio::test]
    async fn test_verify_invalid_poa() {
        let mdoc_nonce = "mdoc_nonce";
        let ca = Ca::generate_issuer_mock_ca().unwrap();
        let trust_anchors = &[ca.to_trust_anchor()];
        let (issuer_signed_and_keys, auth_request) = setup_poa_test(&ca).await;
        let (vp_token, poa) = setup_vp_token(
            &auth_request,
            mdoc_nonce,
            &issuer_signed_and_keys,
            trust_anchors,
            &MockTimeGenerator::default(),
        )
        .await;

        let mut poa = poa.unwrap();
        poa.set_payload("edited".to_owned());

        let auth_response = VpAuthorizationResponse::new(vp_token, None, Some(poa));
        let error = auth_response
            .verify(
                &auth_request,
                &[MOCK_WALLET_CLIENT_ID.to_string()],
                mdoc_nonce,
                &TimeGenerator,
                trust_anchors,
            )
            .expect_err("should fail due to missing PoA");
        assert!(matches!(error, AuthResponseError::PoaVerification(_)));
    }
}
