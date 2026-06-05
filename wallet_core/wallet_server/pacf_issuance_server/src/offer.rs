use std::sync::Arc;

use attestation_types::credential_format::Format;
use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use issuer_common::IssuanceServerIssuer;
use openid4vc::credential_offer::CredentialOffer;
use openid4vc::credential_offer::CredentialOfferContainer;
use openid4vc::credential_offer::OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::server_state::SessionStoreError;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum OfferError {
    #[error("attestation type for format \"{0}\" not configured: {1}")]
    AttestationTypeNotConfigured(Format, String),

    #[error("failed to create session: {0}")]
    SessionStore(#[source] SessionStoreError),
}

impl axum::response::IntoResponse for OfferError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            OfferError::AttestationTypeNotConfigured(_, _) => StatusCode::BAD_REQUEST,
            OfferError::SessionStore(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

struct ApplicationState {
    issuer: Arc<IssuanceServerIssuer>,
}

pub fn create_offer_router(issuer: Arc<IssuanceServerIssuer>) -> Router {
    Router::new()
        .route("/offer", post(offer))
        .with_state(Arc::new(ApplicationState { issuer }))
}

#[derive(Serialize, Deserialize)]
pub struct OfferRequest {
    pub documents: VecNonEmpty<IssuableDocument>,
}

#[derive(Serialize, Deserialize)]
pub struct OfferResponse {
    pub credential_offer_url: Url,
}

/// Accepts a list of issuable documents, creates a pre-authorized issuance session, and returns
/// the resulting `openid-credential-offer://` URL to be shown to the user.
async fn offer(
    State(state): State<Arc<ApplicationState>>,
    Json(request): Json<OfferRequest>,
) -> Result<Json<OfferResponse>, OfferError> {
    let credential_configuration_ids: VecNonEmpty<_> = request
        .documents
        .iter()
        .map(|document| {
            state
                .issuer
                .credential_config_id_by_format_and_attestation_type(document.format, &document.attestation_type)
                .cloned()
                .ok_or_else(|| {
                    OfferError::AttestationTypeNotConfigured(document.format, document.attestation_type.clone())
                })
        })
        .collect::<Result<Vec<_>, _>>()?
        .try_into()
        .unwrap(); // we started with a VecNonEmpty

    let token = state
        .issuer
        .new_session(request.documents)
        .await
        .map_err(OfferError::SessionStore)?;

    let credential_offer = CredentialOffer::new_pre_authorized(
        state.issuer.issuer_identifier().clone(),
        credential_configuration_ids,
        token.into(),
    );

    let offer = CredentialOfferContainer::new_offer(credential_offer);
    let credential_offer_url = format!(
        "{OPENID4VCI_CREDENTIAL_OFFER_URL_SCHEME}://?{}",
        serde_urlencoded::to_string(&offer).expect("credential offer serialization should not fail")
    )
    .parse()
    .unwrap();

    Ok(Json(OfferResponse { credential_offer_url }))
}
