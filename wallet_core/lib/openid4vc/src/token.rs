use std::num::NonZeroU8;
use std::time::Duration;

use derive_more::Debug;
use derive_more::From;
use indexmap::IndexMap;
use indexmap::IndexSet;
use rustls_pki_types::TrustAnchor;
use serde::Deserialize;
use serde::Serialize;
use serde_with::base64::Base64;
use serde_with::formats::SpaceSeparator;
use serde_with::serde_as;
use serde_with::skip_serializing_none;
use serde_with::DurationSeconds;
use serde_with::StringWithSeparator;
use url::Url;

use attestation_data::auth::issuer_auth::IssuerRegistration;
use attestation_data::credential_payload::PreviewableCredentialPayload;
use attestation_data::x509::CertificateType;
use crypto::utils::random_string;
use crypto::utils::sha256;
use crypto::x509::BorrowingCertificate;
use crypto::x509::CertificateError;
use crypto::x509::CertificateUsage;
use error_category::ErrorCategory;
use http_utils::urls::HttpsUri;
use sd_jwt_vc_metadata::TypeMetadataDocuments;
use utils::generator::TimeGenerator;
use utils::vec_at_least::VecNonEmpty;

use crate::authorization::AuthorizationDetails;
use crate::server_state::SessionToken;
use crate::Format;

#[derive(Serialize, Deserialize, Debug, Clone, From)]
pub struct AuthorizationCode(String);

impl AsRef<str> for AuthorizationCode {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, From)]
pub struct AccessToken(String);

impl AsRef<str> for AccessToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<AuthorizationCode> for SessionToken {
    fn from(value: AuthorizationCode) -> Self {
        SessionToken::from(value.0)
    }
}

impl AccessToken {
    /// Construct a new random access token, with the specified authorization code appended to it.
    pub(crate) fn new(code: &AuthorizationCode) -> Self {
        Self(random_string(32) + code.as_ref())
    }

    /// Returns the authorization code appended to this access token.
    pub(crate) fn code(&self) -> Option<AuthorizationCode> {
        self.as_ref().get(32..).map(|code| AuthorizationCode(code.to_string()))
    }

    pub(crate) fn sha256(&self) -> Vec<u8> {
        sha256(self.as_ref().as_bytes())
    }
}

/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-token-request>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-4.1.3>.
/// Sent URL-encoded in request body to POST /token.
/// A DPoP HTTP header may be included.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenRequest {
    #[serde(flatten)]
    pub grant_type: TokenRequestGrantType,

    pub code_verifier: Option<String>,
    pub client_id: Option<String>,

    /// MUST be the redirect URI value as passed to the authorization request
    pub redirect_uri: Option<Url>,
}

impl TokenRequest {
    /// Retrieve either the authorization code or the pre-authorized code, depending on the authorization grant type.
    pub fn code(&self) -> &AuthorizationCode {
        match &self.grant_type {
            TokenRequestGrantType::AuthorizationCode { code } => code,
            TokenRequestGrantType::PreAuthorizedCode { pre_authorized_code } => pre_authorized_code,
        }
    }
}

#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename = "snake_case")]
#[serde(tag = "grant_type")]
pub enum TokenRequestGrantType {
    #[serde(rename = "authorization_code")]
    AuthorizationCode { code: AuthorizationCode },
    #[serde(rename = "urn:ietf:params:oauth:grant-type:pre-authorized_code")]
    PreAuthorizedCode {
        #[serde(rename = "pre-authorized_code")]
        pre_authorized_code: AuthorizationCode,
    },
}

/// See
/// <https://openid.net/specs/openid-4-verifiable-credential-issuance-1_0-13.html#name-successful-token-response>
/// and <https://www.rfc-editor.org/rfc/rfc6749.html#section-5.1>.
#[serde_as]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TokenResponse {
    pub access_token: AccessToken,
    pub token_type: TokenType,
    pub refresh_token: Option<String>,
    pub c_nonce: Option<String>,

    #[serde_as(as = "Option<StringWithSeparator::<SpaceSeparator, String>>")]
    pub scope: Option<IndexSet<String>>,

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub c_nonce_expires_in: Option<Duration>, // lifetime of `c_nonce`

    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    pub expires_in: Option<Duration>,

    /// "REQUIRED when authorization_details parameter is used to request issuance of a certain Credential type
    /// as defined in Section 5.1.1. MUST NOT be used otherwise."
    pub authorization_details: Option<AuthorizationDetails>,
}

/// A [`TokenResponse`] with an extra field for the credential previews.
/// This is a custom field so other implementations might not send it. For now however we assume that it is always
/// present so it is not an [`Option`].
#[derive(Debug, Serialize, Deserialize)]
pub struct TokenResponseWithPreviews {
    #[serde(flatten)]
    pub token_response: TokenResponse,
    pub credential_previews: VecNonEmpty<CredentialPreview>,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialPreviewContent {
    /// The amount of copies of this attestation that the holder will receive per credential format.
    #[serde(with = "indexmap::map::serde_seq")]
    pub copies_per_format: IndexMap<Format, NonZeroU8>,

    pub credential_payload: PreviewableCredentialPayload,

    #[serde_as(as = "Base64")]
    pub issuer_certificate: BorrowingCertificate,
}

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CredentialPreview {
    #[serde(flatten)]
    pub content: CredentialPreviewContent,

    #[debug(skip)]
    pub type_metadata: TypeMetadataDocuments,
}

impl CredentialPreview {
    pub fn verify(&self, trust_anchors: &[TrustAnchor<'_>]) -> Result<(), CredentialPreviewError> {
        let Self { content, .. } = self;

        // Verify the issuer certificates that the issuer presents for each credential to be issued.
        // NB: this only proves the authenticity of the data inside the certificates (the
        // [`IssuerRegistration`]s), but does not authenticate the issuer that presents them.
        // Anyone that has ever seen these certificates (such as other wallets that received them during
        // issuance) could present them here in the protocol without needing the corresponding
        // issuer private key. This is not a problem, because at the end of the issuance
        // protocol each mdoc is verified against the corresponding certificate in the
        // credential preview, which implicitly authenticates the issuer because only it could
        // have produced an mdoc against that certificate.
        content
            .issuer_certificate
            .verify(CertificateUsage::Mdl, &[], &TimeGenerator, trust_anchors)?;

        // Verify that the issuer_uri is among the SAN DNS names or URIs in the issuer_certificate
        if !content
            .issuer_certificate
            .san_dns_name_or_uris()?
            .as_ref()
            .contains(&content.credential_payload.issuer)
        {
            return Err(CredentialPreviewError::IssuerUriNotFoundInSan(
                content.credential_payload.issuer.clone(),
                content.issuer_certificate.san_dns_name_or_uris()?,
            ));
        }

        Ok(())
    }

    pub fn issuer_registration(&self) -> Result<IssuerRegistration, CredentialPreviewError> {
        let CertificateType::Mdl(Some(issuer)) = CertificateType::from_certificate(&self.content.issuer_certificate)?
        else {
            Err(CredentialPreviewError::NoIssuerRegistration)?
        };
        Ok(*issuer)
    }
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
pub enum CredentialPreviewError {
    #[error("certificate error: {0}")]
    #[category(defer)]
    Certificate(#[from] CertificateError),

    #[error("issuer registration not found in certificate")]
    #[category(critical)]
    NoIssuerRegistration,

    #[error("issuer URI {0} not found in SAN {1:?}")]
    #[category(pd)]
    IssuerUriNotFoundInSan(HttpsUri, VecNonEmpty<HttpsUri>),
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub enum TokenType {
    #[default]
    Bearer,
    DPoP,
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use indexmap::IndexSet;
    use serde_json::json;

    use crate::token::TokenRequest;
    use crate::token::TokenRequestGrantType;
    use crate::token::TokenResponse;

    #[test]
    fn token_request_serialization() {
        #[rustfmt::skip]
        assert_eq!(
            serde_urlencoded::to_string(TokenRequest {
                grant_type: TokenRequestGrantType::PreAuthorizedCode {
                    pre_authorized_code: "123".to_string().into()
                },
                code_verifier: Some("myverifier".to_string()),
                client_id: Some("myclient".to_string()),
                redirect_uri: Some("https://example.com".parse().unwrap())
            })
            .unwrap(),
            "grant_type=urn%3Aietf%3Aparams%3Aoauth%3Agrant-type%3Apre-authorized_code\
                &pre-authorized_code=123\
                &code_verifier=myverifier\
                &client_id=myclient\
                &redirect_uri=https%3A%2F%2Fexample.com%2F",
        );
    }

    #[test]
    fn token_response_serialization() {
        assert_eq!(
            serde_json::to_string(&TokenResponse {
                access_token: "access_token".to_string().into(),
                token_type: crate::token::TokenType::Bearer,
                c_nonce: Some("c_nonce".to_string()),
                scope: Some(IndexSet::from_iter(["scope1".to_string(), "scope2".to_string()])),
                c_nonce_expires_in: Some(Duration::from_secs(10)),
                expires_in: None,
                refresh_token: None,
                authorization_details: None,
            })
            .unwrap(),
            json!({
                "access_token": "access_token",
                "token_type": "Bearer",
                "c_nonce": "c_nonce",
                "scope": "scope1 scope2",
                "c_nonce_expires_in": 10
            })
            .to_string(),
        );
    }
}
