use std::sync::Arc;

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use issuer_common::IssuanceServerIssuer;
use openid4vc::credential_offer::CredentialOfferContainer;
use openid4vc::issuable_document::IssuableDocument;
use openid4vc::issuer::IssuanceError;
use serde::Deserialize;
use serde::Serialize;
use url::Url;
use utils::vec_at_least::VecNonEmpty;

#[derive(Debug, thiserror::Error)]
pub enum OfferError {
    #[error("issuance error: {0}")]
    Issuer(#[source] IssuanceError),
}

impl axum::response::IntoResponse for OfferError {
    fn into_response(self) -> axum::response::Response {
        let status = match self {
            OfferError::Issuer(IssuanceError::AttestationTypeNotConfigured(_, _)) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
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
    let credential_offer = state
        .issuer
        .pre_authorized_offer_from_documents(request.documents)
        .await
        .map_err(OfferError::Issuer)?;

    let credential_offer_url = CredentialOfferContainer::new_offer(credential_offer).to_credential_offer_url();
    Ok(Json(OfferResponse { credential_offer_url }))
}
