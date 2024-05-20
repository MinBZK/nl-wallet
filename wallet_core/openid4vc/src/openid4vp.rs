use std::string::FromUtf8Error;

use base64::DecodeError;
use chrono::{DateTime, Utc};
use josekit::{
    jwe::{alg::ecdh_es::EcdhEsJweAlgorithm, JweHeader},
    jwk::{alg::ec::EcKeyPair, Jwk},
    jwt::JwtPayload,
    JoseError,
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use x509_parser::{error::X509Error, extensions::GeneralName};

use nl_wallet_mdoc::{
    holder::TrustAnchor,
    utils::{
        serialization::CborBase64,
        x509::{Certificate, CertificateError},
    },
    verifier::{DisclosedAttributes, ItemsRequests},
    DeviceResponse, SessionTranscript,
};
use wallet_common::{
    config::wallet_config::BaseUrl,
    generator::{Generator, TimeGenerator},
    jwt::{Jwt, JwtError},
    utils::random_string,
};

use crate::{
    authorization::{AuthorizationRequest, ResponseMode, ResponseType},
    jwt::{self, JwkConversionError, JwtX5cError},
    presentation_exchange::{
        FormatAlg, FormatDesignation, InputDescriptorMappingObject, PresentationDefinition, PresentationSubmission,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum AuthRequestError {
    #[error("unexpected field: {0}")]
    UnexpectedField(&'static str),
    #[error("missing required field: {0}")]
    ExpectedFieldMissing(&'static str),
    #[error("unsupported value for field {field}: expected {expected}, found {found}")]
    UnsupportedFieldValue {
        field: &'static str,
        expected: &'static str,
        found: String,
    },
    #[error("field {0}_uri found, expected field directly")]
    UriVariantNotSupported(&'static str),
    #[error("unexpected amount of JWKs found in client_metadata: expected 1, found {0}")]
    UnexpectedJwkAmount(usize),
    #[error("JWK of unsupported type or curve (kty or crv field)")]
    UnsupportedJwk,

    #[error("error parsing X.509 certificate: {0}")]
    CertificateParsing(#[from] CertificateError),
    #[error("failed to verify Authorization Request JWT: {0}")]
    JwtVerification(#[from] JwtError),
    #[error("error reading Subject Alternative Name in X.509 certificate: {0}")]
    X509(#[source] X509Error),
    #[error("Subject Alternative Name missing from X.509 certificate")]
    MissingSAN,
    #[error("Subject Alternative Name in X.509 certificate was not a DNS name")]
    UnexpectedSANType,
    #[error("failed to convert encryption key to JWK format")]
    JwkConversion(#[from] JwkConversionError),

    #[error("error verifying Authorization Request JWT: {0}")]
    AuthRequestJwtVerification(#[from] JwtX5cError),

    #[error("client_id from Authorization Request was {client_id}, should have been equal to SAN DNSName from X.509 certificate ({dns_san})")]
    UnauthorizedClientId { client_id: String, dns_san: String },
}

/// Contains URL from which the wallet is to retrieve the Authorization Request.
/// To be URL-encoded in the wallet's UL, which can then be put on a website directly, or in a QR code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpAuthorizationRequestReference {
    /// URL at which the full Authorization Request is to be retrieved.
    pub request_uri: BaseUrl,

    /// MUST equal the client_id from the full Authorization Request.
    pub client_id: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpAuthorizationRequest {
    pub aud: VpAuthorizationRequestAudience,

    #[serde(flatten)]
    pub auth_request: AuthorizationRequest,

    /// Contains requirements on the attestations and/or attributes to be disclosed.
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
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum VpAuthorizationRequestAudience {
    #[default]
    #[serde(rename = "https://self-issued.me/v2")]
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
    pub fn direct(&self) -> &PresentationDefinition {
        match self {
            VpPresentationDefinition::Direct(pd) => pd,
            VpPresentationDefinition::Indirect(_) => panic!("uri variant not supported"),
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
    pub fn direct(&self) -> &ClientMetadata {
        match self {
            VpClientMetadata::Direct(c) => c,
            VpClientMetadata::Indirect(_) => panic!("uri variant not supported"),
        }
    }
}

/// Metadata of the verifier (which acts as the "client" in OAuth).
/// OpenID4VP refers to https://openid.net/specs/openid-connect-registration-1_0.html and
/// https://www.rfc-editor.org/rfc/rfc7591.html for this, but here we implement only what we need.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientMetadata {
    #[serde(flatten)]
    jwks: VpJwks,
    vp_formats: VpFormat,

    // These two are defined in https://openid.net/specs/oauth-v2-jarm-final.html
    authorization_encryption_alg_values_supported: VpAlgValues,
    authorization_encryption_enc_values_supported: VpEncValues,
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
    pub fn direct(&self) -> &Vec<Jwk> {
        match self {
            VpJwks::Direct { keys } => keys,
            VpJwks::Indirect(_) => panic!("uri variant not supported"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpAlgValues {
    #[serde(rename = "ECDH-ES")]
    EcdhEs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VpEncValues {
    A128GCM,
    A192GCM,
    A256GCM,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VpFormat {
    MsoMdoc { alg: Vec<FormatAlg> },
}

fn parse_dns_san(cert: &Certificate) -> Result<String, AuthRequestError> {
    match cert
        .to_x509()?
        .subject_alternative_name()
        .map_err(AuthRequestError::X509)?
        .ok_or(AuthRequestError::MissingSAN)?
        .value
        .general_names
        .first()
        .ok_or(AuthRequestError::MissingSAN)?
    {
        GeneralName::DNSName(name) => Ok(name.to_string()),
        _ => Err(AuthRequestError::UnexpectedSANType),
    }
}

use nutype::nutype;
#[nutype(
    derive(Debug, Clone, TryFrom, AsRef, Serialize, Deserialize),
    validate(predicate = |u| JwePublicKey::supported(u)),
)]
pub struct JwePublicKey(Jwk);

impl JwePublicKey {
    fn supported(jwk: &Jwk) -> bool {
        // Avoid jwk.key_type() which panics if `kty` is not set.
        jwk.parameter("kty").is_some_and(|kty| kty == "EC") && jwk.curve() == Some("P-256")
    }
}

impl VpAuthorizationRequest {
    pub fn new(
        items_requests: &ItemsRequests,
        rp_certificate: &Certificate,
        nonce: String,
        encryption_pubkey: JwePublicKey,
        response_uri: BaseUrl,
    ) -> Result<Self, AuthRequestError> {
        Ok(VpAuthorizationRequest {
            aud: VpAuthorizationRequestAudience::SelfIssued,
            auth_request: AuthorizationRequest {
                response_type: ResponseType::VpToken,
                client_id: parse_dns_san(rp_certificate)?,
                nonce: Some(nonce),
                response_mode: Some(ResponseMode::DirectPostJwt),
                redirect_uri: None,
                state: None,
                authorization_details: None,
                request_uri: None,
                code_challenge: None,
                scope: None,
            },
            presentation_definition: VpPresentationDefinition::Direct(items_requests.into()),
            client_metadata: Some(VpClientMetadata::Direct(ClientMetadata {
                jwks: VpJwks::Direct {
                    keys: vec![encryption_pubkey.into_inner()],
                },
                vp_formats: VpFormat::MsoMdoc {
                    alg: vec![FormatAlg::ES256],
                },
                authorization_encryption_alg_values_supported: VpAlgValues::EcdhEs,
                authorization_encryption_enc_values_supported: VpEncValues::A128GCM,
            })),
            client_id_scheme: Some(ClientIdScheme::X509SanDns),
            response_uri: Some(response_uri),
        })
    }

    pub fn verify(
        jws: &Jwt<VpAuthorizationRequest>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<VpAuthorizationRequest, AuthRequestError> {
        let (auth_request, rp_cert) = jwt::verify_against_trust_anchors(jws, trust_anchors, &TimeGenerator)?;

        let dns_san = parse_dns_san(&rp_cert)?;
        if dns_san != auth_request.auth_request.client_id {
            return Err(AuthRequestError::UnauthorizedClientId {
                client_id: auth_request.auth_request.client_id.clone(),
                dns_san,
            });
        }

        auth_request.validate()?;

        Ok(auth_request)
    }

    pub fn validate(&self) -> Result<(), AuthRequestError> {
        // Check absence of fields that may not be present in an OpenID4VP Authorization Request
        if self.auth_request.authorization_details.is_some() {
            return Err(AuthRequestError::UnexpectedField("authorization_details"));
        }
        if self.auth_request.code_challenge.is_some() {
            return Err(AuthRequestError::UnexpectedField("code_challenge"));
        }
        if self.auth_request.redirect_uri.is_some() {
            return Err(AuthRequestError::UnexpectedField("redirect_uri"));
        }
        if self.auth_request.scope.is_some() {
            return Err(AuthRequestError::UnexpectedField("scope"));
        }
        if self.auth_request.request_uri.is_some() {
            return Err(AuthRequestError::UnexpectedField("request_uri"));
        }

        // Check presence of fields that must be present in an OpenID4VP Authorization Request
        if self.auth_request.nonce.is_none() {
            return Err(AuthRequestError::ExpectedFieldMissing("nonce"));
        }
        if self.auth_request.response_mode.is_none() {
            return Err(AuthRequestError::ExpectedFieldMissing("response_mode"));
        }
        if self.client_metadata.is_none() {
            return Err(AuthRequestError::ExpectedFieldMissing("client_metadata"));
        }
        if self.client_id_scheme.is_none() {
            return Err(AuthRequestError::ExpectedFieldMissing("client_id_scheme"));
        }
        if self.response_uri.is_none() {
            return Err(AuthRequestError::ExpectedFieldMissing("response_uri"));
        }

        // Check that various enums have the expected values
        if self.auth_request.response_type != ResponseType::VpToken {
            return Err(AuthRequestError::UnsupportedFieldValue {
                field: "response_type",
                expected: "vp_token",
                found: serde_json::to_string(&self.auth_request.response_type).unwrap(),
            });
        }
        if *self.auth_request.response_mode.as_ref().unwrap() != ResponseMode::DirectPostJwt {
            return Err(AuthRequestError::UnsupportedFieldValue {
                field: "response_mode",
                expected: "direct_post.jwt",
                found: serde_json::to_string(&self.auth_request.response_type).unwrap(),
            });
        }
        if *self.client_id_scheme.as_ref().unwrap() != ClientIdScheme::X509SanDns {
            return Err(AuthRequestError::UnsupportedFieldValue {
                field: "client_id_scheme",
                expected: "x509_san_dns",
                found: serde_json::to_string(&self.client_id_scheme).unwrap(),
            });
        }

        // Of fields that have an "_uri" variant, check that they are not used
        if matches!(self.presentation_definition, VpPresentationDefinition::Indirect(..)) {
            return Err(AuthRequestError::UriVariantNotSupported("presentation_definition"));
        }
        if matches!(*self.client_metadata.as_ref().unwrap(), VpClientMetadata::Indirect(..)) {
            return Err(AuthRequestError::UriVariantNotSupported("client_metadata"));
        }
        let client_metadata = self.client_metadata.as_ref().unwrap().direct();
        if matches!(client_metadata.jwks, VpJwks::Indirect(..)) {
            return Err(AuthRequestError::UriVariantNotSupported("presentation_definition"));
        }

        // check that we received exactly one EC/P256 curve
        let jwks = client_metadata.jwks.direct();
        if jwks.len() != 1 {
            return Err(AuthRequestError::UnexpectedJwkAmount(jwks.len()));
        }
        if !JwePublicKey::supported(jwks.first().unwrap()) {
            return Err(AuthRequestError::UnsupportedJwk);
        }

        Ok(())
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
    #[error("failed to base64 decode JWE header fields: {0}")]
    Base64(#[from] DecodeError),
    #[error("missing apu field from JWE")]
    MissingApu,
    #[error("missing apv field from JWE")]
    MissingApv,
    #[error("failed to decode apu/apv field from JWE")]
    Utf8(#[from] FromUtf8Error),
    #[error("error verifying disclosed mdoc: {0}")]
    Verification(#[source] nl_wallet_mdoc::Error),
    #[error("missing requested attributes: {0}")]
    MissingAttributes(#[source] nl_wallet_mdoc::Error),
    #[error("unexpected amount of Presentation Submission descriptors: expected {expected}, found {found}")]
    UnexpectedDescriptorCount { expected: usize, found: usize },
    #[error("received unexpected Presentation Submission ID: expected {expected}, found {found}")]
    UnexpectedSubmissionId { expected: String, found: String },
    #[error("received unexpected path in Presentation Submission Input Descriptor: expected '$', found '{0}'")]
    UnexpectedInputDescriptorPath(String),
    #[error("received unexpected Presentation Submission Input Descriptor ID: expected {expected}, found {found}")]
    UnexpectedInputDescriptorId { expected: String, found: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VpAuthorizationResponse {
    pub vp_token: CborBase64<DeviceResponse>,
    pub presentation_submission: PresentationSubmission,
}

impl VpAuthorizationResponse {
    fn new(device_response: DeviceResponse, presentation_definition: &PresentationDefinition) -> Self {
        let presentation_submission = PresentationSubmission {
            id: random_string(16),
            definition_id: presentation_definition.id.clone(),
            descriptor_map: device_response
                .documents
                .as_ref()
                .unwrap() // we never produce DeviceResponse instances without documents in it
                .iter()
                .map(|doc| InputDescriptorMappingObject {
                    id: doc.doc_type.clone(),
                    format: FormatDesignation::MsoMdoc,
                    path: "$".to_string(),
                })
                .collect(),
        };

        VpAuthorizationResponse {
            vp_token: device_response.into(),
            presentation_submission,
        }
    }

    /// Create a JWE containing a new encrypted Authorization Request.
    ///
    /// NB: this method assumes that the provided Authorization Request has been validated with
    ///  `auth_request.validate()?` before.
    pub fn new_encrypted(
        device_response: DeviceResponse,
        auth_request: &VpAuthorizationRequest,
        mdoc_nonce: String,
    ) -> Result<String, AuthResponseError> {
        Self::new(device_response, auth_request.presentation_definition.direct()).encrypt(auth_request, mdoc_nonce)
    }

    fn encrypt(&self, auth_request: &VpAuthorizationRequest, mdoc_nonce: String) -> Result<String, AuthResponseError> {
        // All .unwrap() and .direct() calls on `auth_request` are safe because they are checked earlier
        // by auth_request.validate().

        let mut header = JweHeader::new();
        header.set_token_type("JWT");

        // Set the `apu` and `apv` fields to the mdoc nonce and nonce, per the ISO 18013-7 profile.
        header.set_agreement_partyuinfo(mdoc_nonce);
        header.set_agreement_partyvinfo(auth_request.auth_request.nonce.as_ref().unwrap().clone());

        // Use the AES key size that the server wants.
        let client_metadata = auth_request.client_metadata.as_ref().unwrap().direct();
        header.set_content_encryption(match client_metadata.authorization_encryption_enc_values_supported {
            VpEncValues::A128GCM => "A128GCM",
            VpEncValues::A192GCM => "A192GCM",
            VpEncValues::A256GCM => "A256GCM",
        });

        // VpAuthorizationRequest always serializes to a JSON object useable as a JWT payload.
        let payload = serde_json::to_value(self)?.as_object().unwrap().clone();
        let payload = JwtPayload::from_map(payload).unwrap();

        // The key the RP wants us to encrypt our response to.
        let jwk = client_metadata.jwks.direct().first().unwrap();

        let encrypter = EcdhEsJweAlgorithm::EcdhEs
            .encrypter_from_jwk(jwk)
            .map_err(AuthResponseError::JwkConversion)?;
        let jwe = josekit::jwt::encode_with_encrypter(&payload, &header, &encrypter).map_err(AuthResponseError::Jwe)?;

        Ok(jwe)
    }

    pub fn decrypt(
        jwe: String,
        private_key: EcKeyPair,
        nonce: String,
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
        &self,
        auth_request: &VpAuthorizationRequest,
        mdoc_nonce: String,
        time: &impl Generator<DateTime<Utc>>,
        trust_anchors: &[TrustAnchor],
    ) -> Result<DisclosedAttributes, AuthResponseError> {
        // All .unwrap() and .direct() calls on `auth_request` are safe because we (the RP) constructed
        // the `auth_request` ourselves earlier in the session.

        // Verify the cryptographic integrity of the disclosed attributes.
        let session_transcript = SessionTranscript::new_oid4vp(
            &auth_request.response_uri.as_ref().unwrap().clone(),
            auth_request.auth_request.client_id.clone(),
            auth_request.auth_request.nonce.as_ref().unwrap().clone(),
            mdoc_nonce,
        );
        let device_response = &self.vp_token.0;
        let disclosed_attrs = device_response
            .verify(None, &session_transcript, time, trust_anchors)
            .map_err(AuthResponseError::Verification)?;

        // Check that we received all attributes that we requested.
        let items_requests: ItemsRequests = auth_request.presentation_definition.direct().try_into().unwrap();
        items_requests
            .match_against_response(device_response)
            .map_err(AuthResponseError::MissingAttributes)?;

        // Check that the Presentation Submission is what it should be per the Presentation Exchange spec and ISO 18013-7.
        self.verify_presentation_submission(auth_request.presentation_definition.direct())?;

        Ok(disclosed_attrs)
    }

    fn verify_presentation_submission(
        &self,
        presentation_definition: &PresentationDefinition,
    ) -> Result<(), AuthResponseError> {
        if self.presentation_submission.definition_id != presentation_definition.id {
            return Err(AuthResponseError::UnexpectedSubmissionId {
                expected: presentation_definition.id.clone(),
                found: self.presentation_submission.definition_id.clone(),
            });
        }

        if self.presentation_submission.descriptor_map.len()
            != self.vp_token.0.documents.as_ref().map_or(0, |docs| docs.len())
        {
            return Err(AuthResponseError::UnexpectedDescriptorCount {
                expected: self.vp_token.0.documents.as_ref().map_or(0, |docs| docs.len()),
                found: self.presentation_submission.descriptor_map.len(),
            });
        }

        self.vp_token
            .0
            .documents
            .as_ref()
            .unwrap()
            .iter()
            .zip(&self.presentation_submission.descriptor_map)
            .try_for_each(|(doc, input_descriptor)| {
                if input_descriptor.path != "$" {
                    return Err(AuthResponseError::UnexpectedInputDescriptorPath(
                        input_descriptor.path.to_string(),
                    ));
                }
                if input_descriptor.id != doc.doc_type {
                    return Err(AuthResponseError::UnexpectedInputDescriptorId {
                        expected: doc.doc_type.clone(),
                        found: input_descriptor.id.clone(),
                    });
                }
                Ok::<_, AuthResponseError>(())
            })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use josekit::jwk::alg::ec::{EcCurve, EcKeyPair};
    use serde_json::json;

    use nl_wallet_mdoc::{examples::Example, server_keys::KeyPair, test::example_items_requests, DeviceResponse};
    use wallet_common::utils::random_string;

    use crate::{
        jwt,
        openid4vp::{VpAuthorizationRequest, VpAuthorizationResponse},
    };

    #[test]
    fn test_encrypt_decrypt_authorization_response() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let rp_keypair = ca.generate_reader_mock(None).unwrap();

        let nonce = "nonce".to_string();

        let encryption_privkey = EcKeyPair::generate(EcCurve::P256).unwrap();
        let auth_request = VpAuthorizationRequest::new(
            &example_items_requests(),
            rp_keypair.certificate(),
            nonce.clone(),
            encryption_privkey.to_jwk_public_key().try_into().unwrap(),
            "https://example.com/response_uri".parse().unwrap(),
        )
        .unwrap();

        // NB: the example DeviceResponse verifies as an ISO 18013-5 DeviceResponse while here we use it in
        // an OpenID4VP setting, i.e. with different SessionTranscript contents, so it can't be verified.
        // This is not an issue here because this code only deals with en/decrypting the DeviceResponse into
        // an Authorization Response JWE.
        let mdoc_nonce = random_string(16);
        let device_response = DeviceResponse::example();
        let jwe = VpAuthorizationResponse::new(device_response, auth_request.presentation_definition.direct())
            .encrypt(&auth_request, mdoc_nonce.clone())
            .unwrap();

        let (_decrypted, jwe_mdoc_nonce) = VpAuthorizationResponse::decrypt(jwe, encryption_privkey, nonce).unwrap();

        assert_eq!(mdoc_nonce, jwe_mdoc_nonce);
    }

    #[tokio::test]
    async fn test_authorization_request_jwt() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let rp_keypair = ca.generate_reader_mock(None).unwrap();

        let encryption_privkey = EcKeyPair::generate(EcCurve::P256).unwrap();
        dbg!(encryption_privkey.to_jwk_public_key());

        let auth_request = VpAuthorizationRequest::new(
            &example_items_requests(),
            rp_keypair.certificate(),
            "nonce".to_string(),
            encryption_privkey.to_jwk_public_key().try_into().unwrap(),
            "https://example.com/response_uri".parse().unwrap(),
        )
        .unwrap();
        let auth_request_jwt = jwt::sign_with_certificate(&auth_request, &rp_keypair).await.unwrap();

        VpAuthorizationRequest::verify(&auth_request_jwt, &[ca.certificate().try_into().unwrap()]).unwrap();

        dbg!(auth_request_jwt.0);
    }

    #[test]
    fn deserialize_example() {
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
        auth_request.validate().unwrap();

        dbg!(auth_request);
    }
}
