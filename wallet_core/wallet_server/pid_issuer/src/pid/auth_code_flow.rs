use std::sync::Arc;

use attestation_data::attributes::Attribute;
use attestation_data::attributes::AttributeValue;
use attestation_data::attributes::Attributes;
use attestation_data::attributes::AttributesHandlingError;
use attestation_types::claim_path::ClaimPath;
use axum::Router;
use axum::extract::Query;
use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use crypto::utils::random_string;
use crypto::x509::CertificateError;
use hsm::service::HsmError;
use indexmap::IndexSet;
use issuer_common::state_bridge_store::IssuerStateBridgeStore;
use issuer_common::state_bridge_store::IssuerStateBridgeStoreError;
use jwk_simple::Key;
use jwt::nonce::Nonce;
use openid4vc::Format;
use openid4vc::authorization::OidcAuthorizationRequest;
use openid4vc::authorization::PkceCodeChallenge;
use openid4vc::authorization::VciAuthorizationRequest;
use openid4vc::authorization_code_flow::AuthorizationCodeFlow;
use openid4vc::authorization_code_flow::AuthorizeOutcome;
use openid4vc::authorizing_issuer::AuthorizingIssuer;
use openid4vc::authorizing_issuer::CompleteAuthorizationError;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::IssuanceData;
use openid4vc::pkce::PkcePair;
use openid4vc::pkce::S256PkcePair;
use openid4vc::server_state::SessionStore;
use openid4vc::store::Store;
use openid4vc::token::AuthorizationCode;
use serde::Deserialize;
use serde::Serialize;
use server_utils::keys::SecretKeyVariant;
use tracing::warn;
use url::Url;
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

const ISSUER_STATE_LENGTH: usize = 32;

/// Errors raised by [`UpstreamOidcAuthorizationCodeFlow`] on either half of the flow.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("DigiD error: {0}")]
    Digid(#[source] digid::Error),

    #[error("only S256 code_challenge_method is supported")]
    UnsupportedCodeChallenge,

    #[error("state bridge store error: {0}")]
    StateBridge(#[source] IssuerStateBridgeStoreError),

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

    #[error("wallet did not supply a redirect_uri at /authorize, cannot finish the callback")]
    MissingWalletRedirectUri,

    #[error("error completing authorization: {0}")]
    CompleteAuthorization(#[source] CompleteAuthorizationError),
}

/// One state-bridge entry, written at `/authorize` and consumed by the upstream callback handler.
///
/// Linked to the upstream provider by the `issuer_state` random string we send as `state` in the
/// upstream redirect, which the upstream then echoes back to our callback. Carries everything the
/// callback needs to (a) complete the upstream `/token` + `/userinfo` exchange and (b) build the
/// wallet-facing redirect.
#[derive(Serialize, Deserialize)]
struct StateBridgeEntry {
    wallet_redirect_uri: Url,
    wallet_state: Option<String>,
    wallet_code_challenge: String,
    upstream_code_verifier: String,
}

/// Query parameters sent by the upstream provider when redirecting the user back to the issuer's
/// callback URL after a successful authentication.
#[derive(Deserialize)]
struct DigidCallbackQuery {
    code: AuthorizationCode,
    state: String,
}

/// Concrete [`AuthorizationCodeFlow`] for the pid_issuer's upstream-OIDC (DigiD) flow.
///
/// Owns:
/// - upstream OIDC discovery cache + client (for the authorize-endpoint URL and the `/userinfo`-based BSN exchange);
/// - the state-bridge store linking the issuer-generated `issuer_state` (sent to the upstream as `state`) to the
///   wallet's original `redirect_uri`, `state`, PKCE challenge and our upstream PKCE verifier;
/// - the BRP client (BSN → person attributes) and the recovery-code HMAC key;
/// - the issuer's own callback URL, used both as the upstream `redirect_uri` and as the `redirect_uri` parameter of the
///   upstream `/token` exchange.
pub struct UpstreamOidcAuthorizationCodeFlow {
    brp_client: HttpBrpClient,
    openid_client: OpenIdClient,
    recovery_code_secret_key: SecretKeyVariant,
    state_bridge_store: Arc<IssuerStateBridgeStore>,
    callback_uri: Url,
    client_id: String,
    scopes: IndexSet<String>,
}

impl UpstreamOidcAuthorizationCodeFlow {
    #[expect(clippy::too_many_arguments, reason = "Constructor wiring upstream-OIDC dependencies")]
    pub fn try_new(
        brp_client: HttpBrpClient,
        bsn_privkey: &Key,
        client_id: impl Into<String>,
        digid_metadata_cache: DigidMetadataCache,
        recovery_code_secret_key: SecretKeyVariant,
        state_bridge_store: Arc<IssuerStateBridgeStore>,
        callback_uri: Url,
    ) -> Result<Self, Error> {
        let client_id: String = client_id.into();
        Ok(Self {
            brp_client,
            openid_client: OpenIdClient::try_new(bsn_privkey, client_id.clone(), digid_metadata_cache)
                .map_err(Error::Digid)?,
            recovery_code_secret_key,
            state_bridge_store,
            callback_uri,
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

    /// Mount the `/digid/callback` route owned by this flow on a fresh [`Router`]. The
    /// pid_issuer binary merges this with the framework's authorization and issuance routers.
    /// The handler reads its flow state via [`AuthorizingIssuer::flow`].
    pub fn callback_router<K, L, S, N, PAS>(authorizing_issuer: Arc<AuthorizingIssuer<K, L, S, N, PAS, Self>>) -> Router
    where
        K: Send + Sync + 'static,
        L: Send + Sync + 'static,
        S: SessionStore<IssuanceData> + Send + Sync + 'static,
        N: Send + Sync + 'static,
        PAS: Send + Sync + 'static,
    {
        Router::new()
            .route("/digid/callback", get(digid_callback::<K, L, S, N, PAS>))
            .with_state(authorizing_issuer)
    }
}

impl AuthorizationCodeFlow for UpstreamOidcAuthorizationCodeFlow {
    type Error = Error;

    // TODO (PVW-5953): create separate nl-rdo-max AuthorizationRequest and remove `mut` qualifier
    async fn authorize(&self, mut request: VciAuthorizationRequest) -> Result<AuthorizeOutcome, Self::Error> {
        // Capture the wallet-side parameters we'll need at callback time to redirect the
        // user-agent back to the wallet, and the wallet's PKCE challenge that the framework's
        // /token handler will verify against.
        let wallet_code_challenge = match request.code_challenge {
            PkceCodeChallenge::S256 { code_challenge } => code_challenge,
            PkceCodeChallenge::Plain { .. } => return Err(Error::UnsupportedCodeChallenge),
        };
        let wallet_redirect_uri = request.redirect_uri.into_inner();
        let wallet_state = request.oauth_request.state.clone();

        // Generate the upstream PKCE pair and the random `issuer_state` we'll use as `state` in
        // the upstream redirect. The upstream provider will echo it back to our callback.
        let upstream_pkce = S256PkcePair::generate();
        let upstream_code_challenge = upstream_pkce.code_challenge().to_string();
        let upstream_code_verifier = upstream_pkce.into_code_verifier();
        let issuer_state = random_string(ISSUER_STATE_LENGTH);

        let entry = StateBridgeEntry {
            wallet_redirect_uri,
            wallet_state,
            wallet_code_challenge,
            upstream_code_verifier,
        };
        self.state_bridge_store
            .store(issuer_state.clone(), entry)
            .await
            .map_err(Error::StateBridge)?;

        // Substitute the upstream-facing parameters in the original wallet request, then encode.
        request.code_challenge = PkceCodeChallenge::S256 {
            code_challenge: upstream_code_challenge,
        };
        request.oauth_request.client_id = self.client_id.clone();
        request.oauth_request.state = Some(issuer_state);
        request.redirect_uri = self.callback_uri.clone().into();
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
}

/// `GET /digid/callback`: termination point for the upstream OIDC redirect. Exchanges the upstream
/// `code` for a BSN, looks up attributes in the BRP, builds the [`IssuableDocument`]s, and hands
/// them to the framework's [`AuthorizingIssuer::complete_authorization`] which mints the
/// issuer-side authorization code, writes the `AuthCodeIssued` session, and produces the wallet-facing
/// redirect URL. Errors during the BSN / BRP / issuable-build steps surface to the wallet as an
/// OAuth error redirect, since the wallet's redirect_uri is known by then.
type DigidCallbackAuthorizingIssuer<K, L, S, N, PAS> =
    Arc<AuthorizingIssuer<K, L, S, N, PAS, UpstreamOidcAuthorizationCodeFlow>>;

async fn digid_callback<K, L, S, N, PAS>(
    State(authorizing_issuer): State<DigidCallbackAuthorizingIssuer<K, L, S, N, PAS>>,
    Query(DigidCallbackQuery { code, state }): Query<DigidCallbackQuery>,
) -> Response
where
    K: Send + Sync + 'static,
    L: Send + Sync + 'static,
    S: SessionStore<IssuanceData> + Send + Sync + 'static,
    N: Send + Sync + 'static,
    PAS: Send + Sync + 'static,
{
    let flow = authorizing_issuer.flow();

    let entry: StateBridgeEntry = match flow.state_bridge_store.consume(state.as_str()).await {
        Ok(Some(entry)) => entry,
        Ok(None) => {
            warn!("digid callback: unknown or expired issuer_state");
            return (StatusCode::BAD_REQUEST, "unknown or expired state").into_response();
        }
        Err(error) => {
            warn!("digid callback: state bridge consume failed: {error}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "state bridge error").into_response();
        }
    };

    match complete_digid_callback(flow, authorizing_issuer.as_ref(), &entry, code).await {
        Ok(wallet_redirect_url) => {
            (StatusCode::FOUND, [(header::LOCATION, wallet_redirect_url.to_string())]).into_response()
        }
        Err(error) => {
            warn!("digid callback: completion failed: {error}");
            redirect_to_wallet_error(&entry, &error)
        }
    }
}

/// Exchange the upstream `code` for the user's BSN, look up attributes, build the issuable documents, and hand them to
/// the framework's [`AuthorizingIssuer::complete_authorization`], which mints the issuer-side authorization code,
/// writes the `AuthCodeIssued` session, and returns the wallet-facing redirect URL.
async fn complete_digid_callback<K, L, S, N, PAS>(
    flow: &UpstreamOidcAuthorizationCodeFlow,
    authorizing_issuer: &AuthorizingIssuer<K, L, S, N, PAS, UpstreamOidcAuthorizationCodeFlow>,
    entry: &StateBridgeEntry,
    upstream_code: AuthorizationCode,
) -> Result<Url, Error>
where
    S: SessionStore<IssuanceData>,
{
    let bsn = flow
        .openid_client
        .bsn(
            upstream_code,
            entry.upstream_code_verifier.clone(),
            Some(flow.callback_uri.clone()),
        )
        .await
        .map_err(Error::Digid)?;

    let mut persons = flow.brp_client.get_person_by_bsn(&bsn).await.map_err(Error::Brp)?;
    if persons.persons.len() != 1 {
        return Err(Error::NoAttributesFound);
    }
    let person = persons.persons.remove(0);
    let attributes = UpstreamOidcAuthorizationCodeFlow::insert_recovery_code(
        person.into_attributes(),
        &flow.recovery_code_secret_key,
    )
    .await?;

    let issuable_documents = vec_nonempty![
        IssuableDocument::try_new_with_random_id(Format::SdJwt, PID_ATTESTATION_TYPE.to_string(), attributes.clone(),)
            .map_err(|_| Error::InvalidIssuableDocuments)?,
        IssuableDocument::try_new_with_random_id(Format::MsoMdoc, PID_ATTESTATION_TYPE.to_string(), attributes)
            .map_err(|_| Error::InvalidIssuableDocuments)?,
    ];

    let (_code, wallet_redirect_url) = authorizing_issuer
        .complete_authorization(
            issuable_documents,
            entry.wallet_code_challenge.clone(),
            entry.wallet_redirect_uri.clone(),
            entry.wallet_state.clone(),
        )
        .await
        .map_err(Error::CompleteAuthorization)?;

    Ok(wallet_redirect_url)
}

fn redirect_to_wallet_error(entry: &StateBridgeEntry, error: &Error) -> Response {
    let mut url = entry.wallet_redirect_uri.clone();
    let query = serde_urlencoded::to_string([
        ("error", "server_error"),
        ("error_description", &error.to_string()),
        ("state", entry.wallet_state.as_deref().unwrap_or_default()),
    ])
    .expect("encoding error query string should never fail");
    url.set_query(Some(&query));
    (StatusCode::FOUND, [(header::LOCATION, url.to_string())]).into_response()
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

        assert_eq!(
            attrs, expected_attrs,
            "The result should be the attributes we started with, with a recovery_code attribute added to it."
        );
    }
}
