use jsonwebtoken::jwk::Jwk;
use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use x509_parser::{error::X509Error, extensions::GeneralName};

use nl_wallet_mdoc::{
    holder::TrustAnchor,
    server_keys::KeyPair,
    utils::x509::{Certificate, CertificateError},
    verifier::ItemsRequests,
};
use wallet_common::{
    config::wallet_config::BaseUrl,
    generator::TimeGenerator,
    jwt::{Jwt, JwtError},
};

use crate::{
    authorization::{AuthorizationRequest, ResponseMode, ResponseType},
    jwt::{self, JwkConversionError, JwtX5cError},
    presentation_exchange::{FormatAlg, PresentationDefinition},
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

impl VpAuthorizationRequest {
    pub async fn new_signed(
        items_requests: &ItemsRequests,
        keypair: KeyPair,
        nonce: String,
        encryption_pubkey: &VerifyingKey,
        response_uri: BaseUrl,
    ) -> Result<Jwt<Self>, AuthRequestError> {
        let auth_request = VpAuthorizationRequest {
            aud: VpAuthorizationRequestAudience::SelfIssued,
            auth_request: AuthorizationRequest {
                response_type: ResponseType::VpToken,
                client_id: parse_dns_san(keypair.certificate())?,
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
                    keys: vec![jwt::jwk_from_p256(encryption_pubkey)?],
                },
                vp_formats: VpFormat::MsoMdoc {
                    alg: vec![FormatAlg::ES256],
                },
                authorization_encryption_alg_values_supported: VpAlgValues::EcdhEs,
                authorization_encryption_enc_values_supported: VpEncValues::A128GCM,
            })),
            client_id_scheme: Some(ClientIdScheme::X509SanDns),
            response_uri: Some(response_uri),
        };

        let jws = jwt::sign_with_certificate(&auth_request, &keypair).await?;
        Ok(jws)
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

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use p256::{ecdsa::SigningKey, elliptic_curve::rand_core::OsRng};
    use serde_json::json;

    use nl_wallet_mdoc::{server_keys::KeyPair, test::example_items_requests};

    use crate::openid4vp::VpAuthorizationRequest;

    #[tokio::test]
    async fn test_authorization_request_jwt() {
        let ca = KeyPair::generate_ca("myca", Default::default()).unwrap();
        let rp_keypair = ca.generate_reader_mock(None).unwrap();

        let encryption_privkey = SigningKey::random(&mut OsRng);

        let auth_request_jwt = VpAuthorizationRequest::new_signed(
            &example_items_requests(),
            rp_keypair,
            "nonce".to_string(),
            encryption_privkey.verifying_key(),
            "https://example.com/response_uri".parse().unwrap(),
        )
        .await
        .unwrap();

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
