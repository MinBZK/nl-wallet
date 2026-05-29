use std::sync::Arc;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::attributes::AttributesHandlingError;
use attestation_types::claim_path::ClaimPath;
use crypto::x509::CertificateError;
use hsm::service::HsmError;
use indexmap::IndexSet;
use issuer_common::pkce_store::IssuerPkceStore;
use issuer_common::pkce_store::IssuerPkceStoreError;
use jwk_simple::Key;
use jwt::nonce::Nonce;
use openid4vc::Format;
use openid4vc::authorization::OidcAuthorizationRequest;
use openid4vc::authorization::PkceCodeChallenge;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use openid4vc::store::Store;
use openid4vc::token::TokenRequest;
use openid4vc::token::TokenRequestGrantType;
use server_utils::keys::SecretKeyVariant;
use utils::vec_at_least::VecNonEmpty;
use utils::vec_nonempty;

use crate::pid::brp::client::BrpClient;
use crate::pid::brp::client::BrpError;
use crate::pid::brp::client::HttpBrpClient;
use crate::pid::constants::PID_ATTESTATION_TYPE;
use crate::pid::constants::PID_BSN;
use crate::pid::constants::PID_RECOVERY_CODE;
use crate::pid::digid;
use crate::pid::digid::DigidMetadataCache;
use crate::pid::digid::OpenIdClient;

/// Errors raised by [`UpstreamOidcAuthorizationCodeFlow`] on either half of the flow.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DigiD error: {0}")]
    Digid(#[source] digid::Error),

    #[error("only S256 code_challenge_method is supported")]
    UnsupportedCodeChallenge,

    #[error("PKCE bridge store error: {0}")]
    PkceStore(#[source] IssuerPkceStoreError),

    #[error("missing wallet code_verifier on token request")]
    MissingCodeVerifier,

    #[error("wallet code_verifier does not match any stored PKCE bridge entry")]
    PkceVerificationFailed,

    #[error("unexpected grant type for this flow")]
    InvalidGrantType,

    #[error("encoding upstream authorization request as query string failed: {0}")]
    Encode(#[source] serde_urlencoded::ser::Error),

    #[error("could not find attributes for BSN")]
    NoAttributesFound,

    #[error("error retrieving from BRP: {0}")]
    Brp(#[source] BrpError),

    #[error("error creating issuable documents")]
    InvalidIssuableDocuments,

    #[error("certificate error: {0}")]
    Certificate(#[source] CertificateError),

    #[error("could not find BSN attribute")]
    NoBsnFound,

    #[error("error retrieving BSN: {0}")]
    RetrievingBsn(#[source] AttributesHandlingError),

    #[error("BSN attribute had unexpected type (expected string)")]
    BsnUnexpectedType,

    #[error("failed to compute BSN HMAC: {0}")]
    Hmac(#[source] HsmError),

    #[error("error inserting recovery code: {0}")]
    InsertingRecoveryCode(#[source] AttributesHandlingError),
}

/// Concrete [`AuthorizationCodeFlow`] for the pid_issuer's upstream-OIDC (DigiD) flow.
///
/// Owns:
/// - upstream OIDC discovery cache + client (for the authorize-endpoint URL and the `/userinfo`-based BSN exchange);
/// - the wallet ↔ upstream PKCE bridge store, written at `authorize` and consumed at `issuables`;
/// - the BRP client (BSN → person attributes) and the recovery-code HMAC key.
pub struct UpstreamOidcAuthorizationCodeFlow {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient,
    recovery_code_secret_key: SecretKeyVariant,
    pkce_flow_store: Arc<IssuerPkceStore>,
    client_id: String,
    scopes: IndexSet<String>,
}

impl UpstreamOidcAuthorizationCodeFlow {
    pub fn try_new(
        brp_client: HttpBrpClient,
        bsn_privkey: &Key,
        client_id: impl Into<String>,
        digid_metadata_cache: Arc<DigidMetadataCache>,
        recovery_code_secret_key: SecretKeyVariant,
        pkce_flow_store: Arc<IssuerPkceStore>,
    ) -> Result<Self, Error> {
        let client_id: String = client_id.into();
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::try_new(bsn_privkey, client_id.clone(), digid_metadata_cache)
                .map_err(Error::Digid)?,
            recovery_code_secret_key,
            pkce_flow_store,
            client_id,
            scopes: IndexSet::from_iter([String::from("openid")]),
        })
    }

    async fn insert_recovery_code(
        mut attributes: Attributes,
        secret_key: &SecretKeyVariant,
    ) -> Result<Attributes, Error> {
        let bsn = match attributes
            .get(&vec_nonempty![ClaimPath::SelectByKey(PID_BSN.to_string())])
            .map_err(Error::RetrievingBsn)?
            .ok_or(Error::NoBsnFound)?
        {
            AttributeValue::Text(str) => str,
            _ => return Err(Error::BsnUnexpectedType),
        };

        let recovery_code = AttributeValue::Text(hex::encode(
            secret_key.sign_hmac(bsn.as_bytes()).await.map_err(Error::Hmac)?,
        ));

        attributes
            .insert(
                &vec_nonempty![ClaimPath::SelectByKey(PID_RECOVERY_CODE.to_string())],
                Attribute::Single(recovery_code),
            )
            .map_err(Error::InsertingRecoveryCode)?;

        Ok(attributes)
    }
}

impl AuthorizationCodeFlow for UpstreamOidcAuthorizationCodeFlow {
    type Error = Error;

    // TODO (PVW-5953): create separate nl-rdo-max AuthorizationRequest and remove `mut` qualifier
    async fn authorize(&self, mut request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error> {
        // Bridge PKCE: capture the wallet's code-challenge, generate a fresh upstream PKCE pair,
        // substitute the upstream challenge into the request, and store the upstream verifier
        // keyed by the wallet's challenge so `issuables` can recover it at `/token` time.
        // TODO (PVW-5953): move this logic to a separate function
        let wallet_code_challenge = match &request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge.clone(),
            PkceCodeChallenge::Plain { .. } => return Err(Error::UnsupportedCodeChallenge),
        };

        let upstream_pkce = S256PkcePair::generate();
        request.code_challenge = PkceCodeChallenge::S256 {
            code_challenge: upstream_pkce.code_challenge().to_string(),
        };

        self.pkce_flow_store
            .store(wallet_code_challenge, upstream_pkce.into_code_verifier())
            .await
            .map_err(Error::PkceStore)?;

        request.oauth_request.client_id = self.client_id.clone();
        request.scope = Some(self.scopes.clone());

        let oidc_request = OidcAuthorizationRequest {
            vci_request: request,
            nonce: Some(Nonce::new_random()),
        };

        let query_string = serde_urlencoded::to_string(&oidc_request).map_err(Error::Encode)?;
        let mut redirect_url = self
            .openid_client
            .authorization_endpoint()
            .await
            .map_err(Error::Digid)?;
        redirect_url.set_query(Some(&query_string));

        Ok(AuthorizeOutcome::RedirectTo(redirect_url))
    }

    async fn issuables(&self, token_request: TokenRequest) -> Result<VecNonEmpty<IssuableDocument>, Self::Error> {
        let authorization_code = match token_request.grant_type {
            TokenRequestGrantType::AuthorizationCode { code } => code,
            _ => return Err(Error::InvalidGrantType),
        };

        // Consume the PKCE bridge: recompute the wallet's challenge from its supplied verifier and
        // look up the upstream verifier stored at `authorize` time.
        let wallet_code_verifier = token_request
            .code_verifier
            .as_deref()
            .ok_or(Error::MissingCodeVerifier)?;
        let wallet_code_challenge = S256PkcePair::challenge_for(wallet_code_verifier);

        let upstream_code_verifier = self
            .pkce_flow_store
            .consume(&wallet_code_challenge)
            .await
            .map_err(Error::PkceStore)?
            .ok_or(Error::PkceVerificationFailed)?;

        let bsn = self
            .openid_client
            .bsn(authorization_code, upstream_code_verifier, token_request.redirect_uri)
            .await
            .map_err(Error::Digid)?;

        let mut persons = self.brp_client.get_person_by_bsn(&bsn).await.map_err(Error::Brp)?;
        if persons.persons.len() != 1 {
            return Err(Error::NoAttributesFound);
        }

        let person = persons.persons.remove(0);
        let attributes = Self::insert_recovery_code(person.into_attributes(), &self.recovery_code_secret_key).await?;

        // Supply both a SD-JWT and Mdoc PID credential, based on the same set of attributes.
        let issuable_documents = vec_nonempty![
            IssuableDocument::try_new_with_random_id(
                Format::SdJwt,
                PID_ATTESTATION_TYPE.to_string(),
                attributes.clone(),
            )
            .map_err(|_| Error::InvalidIssuableDocuments)?,
            IssuableDocument::try_new_with_random_id(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string(), attributes)
                .map_err(|_| Error::InvalidIssuableDocuments)?,
        ];

        Ok(issuable_documents)
    }
}

#[cfg(test)]
mod tests {
    use attestation_data::attributes::Attribute;
    use attestation_data::attributes::AttributeValue;
    use attestation_data::attributes::Attributes;
    use indexmap::IndexMap;
    use ring::hmac;
    use ring::hmac::HMAC_SHA256;
    use server_utils::keys::SecretKeyVariant;
    use server_utils::settings::SecretKey;

    use super::UpstreamOidcAuthorizationCodeFlow;

    fn attributes(bsn: &str) -> Attributes {
        IndexMap::from_iter([(
            "bsn".to_string(),
            Attribute::Single(AttributeValue::Text(bsn.to_string())),
        )])
        .into()
    }

    #[tokio::test]
    async fn test_recovery_code() {
        let bsn = "123";
        let key: Vec<_> = (0..32).collect();

        let attrs = attributes(bsn);

        let recovery_code_secret_key = SecretKeyVariant::from_settings(
            SecretKey::Software {
                secret_key: key.clone().try_into().unwrap(),
            },
            None,
        )
        .unwrap();

        let attrs = UpstreamOidcAuthorizationCodeFlow::insert_recovery_code(attrs, &recovery_code_secret_key)
            .await
            .unwrap();

        let hmac_key = &hmac::Key::new(HMAC_SHA256, &key);
        let expected_hmac = hex::encode(hmac::sign(hmac_key, bsn.as_bytes()));

        let expected_attrs = Attributes::from(IndexMap::from_iter(attributes(bsn).into_inner().into_iter().chain([(
            "recovery_code".to_string(),
            Attribute::Single(AttributeValue::Text(expected_hmac)),
        )])));

        assert_eq!(attrs, expected_attrs);
    }
}
