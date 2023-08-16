use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    headers::{authorization::Bearer, Authorization},
    response::{IntoResponse, Response},
    routing::post,
    Json, Router, TypedHeader,
};
use base64::prelude::*;
use futures::TryFutureExt;
use http::StatusCode;
use tracing::{debug, error};

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    issuer::{self, MemorySessionStore, PrivateKey, SingleKeyRing},
    ServiceEngagement,
};

use crate::{
    openid::{self, BsnLookup},
    settings::Settings,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OIDC client error: {0}")]
    OpenId(#[from] openid::Error),
    #[error("starting mdoc session failed: {0}")]
    StartMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("mdoc session error: {0}")]
    Mdoc(#[source] nl_wallet_mdoc::Error),
}

// TODO: Implement proper error handling.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response()
    }
}

struct ApplicationState<A, B> {
    attributes_lookup: A,
    openid_client: B,
    issuer: issuer::Server<SingleKeyRing, MemorySessionStore>,
}

pub async fn create_router<A, B>(settings: Settings) -> anyhow::Result<Router>
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    let attributes_lookup = A::new(&settings);

    debug!("Discovering DigiD issuer...");
    let openid_client = B::new(&settings).await?;

    debug!("DigiD issuer discovered, starting HTTP server");

    let key = SingleKeyRing {
        doctype: settings.pid_doctype.clone(),
        issuance_key: PrivateKey::from_der(
            &BASE64_STANDARD.decode(&settings.issuer_key.private_key)?,
            &BASE64_STANDARD.decode(&settings.issuer_key.certificate)?,
        )?,
    };
    let application_state = Arc::new(ApplicationState {
        openid_client,
        issuer: issuer::Server::new(settings.public_url.to_string(), key, MemorySessionStore::new()),
        attributes_lookup,
    });

    let app = Router::new()
        .route("/mdoc/:session_token", post(mdoc_route))
        .route("/start", post(start_route))
        .with_state(application_state);

    Ok(app)
}

async fn mdoc_route<A, B>(
    State(state): State<Arc<ApplicationState<A, B>>>,
    Path(session_token): Path<String>,
    msg: Bytes,
) -> Result<Vec<u8>, Error> {
    let response = state
        .issuer
        .process_message(session_token.into(), &msg)
        .map_err(Error::Mdoc)?;
    Ok(response)
}

async fn start_route<A, B>(
    State(state): State<Arc<ApplicationState<A, B>>>,
    TypedHeader(authorization_header): TypedHeader<Authorization<Bearer>>,
) -> Result<Json<ServiceEngagement>, Error>
where
    A: AttributesLookup,
    B: BsnLookup,
{
    // Using the access_token that the user specified, lookup the user's BSN at the OIDC issuer (DigiD bridge)
    let access_token = authorization_header.token();
    let bsn: String = state
        .openid_client
        .bsn(access_token)
        .inspect_err(|error| error!("error while looking up BSN: {}", error))
        .await?;

    // Start the session, and return the initial mdoc protocol message (containing the URL at which the wallet can
    // find us) to the wallet
    let attributes = state.attributes_lookup.attributes(&bsn)?;
    let service_engagement = state.issuer.new_session(attributes).map_err(Error::StartMdoc)?;

    Ok(Json(service_engagement))
}

/// Given a BSN, determine the attributes to be issued.
pub trait AttributesLookup {
    fn new(settings: &Settings) -> Self;

    /// Given a BSN, determine the attributes to be issued.
    fn attributes(&self, bsn: &str) -> Result<Vec<UnsignedMdoc>, Error>;
}

#[cfg(feature = "mock")]
pub mod mock {
    use std::ops::Add;

    use async_trait::async_trait;
    use chrono::{Days, Utc};
    use ciborium::Value;
    use indexmap::IndexMap;

    use nl_wallet_mdoc::{
        basic_sa_ext::{Entry, UnsignedMdoc},
        Tdate,
    };

    use crate::{
        openid::{self, BsnLookup},
        settings::Settings,
    };

    use super::{AttributesLookup, Error};

    const MOCK_BSN: &str = "999991772";

    pub struct MockBsnLookup {}

    #[async_trait]
    impl BsnLookup for MockBsnLookup {
        async fn new(_: &Settings) -> Result<Self, openid::Error> {
            Ok(Self {})
        }

        async fn bsn(&self, _: &str) -> Result<String, openid::Error> {
            Ok(MOCK_BSN.to_string())
        }
    }

    pub struct MockAttributesLookup {
        doctype: String,
    }

    impl AttributesLookup for MockAttributesLookup {
        fn new(settings: &Settings) -> Self {
            Self {
                doctype: settings.pid_doctype.clone(),
            }
        }

        fn attributes(&self, bsn: &str) -> Result<Vec<UnsignedMdoc>, Error> {
            let pid = UnsignedMdoc {
                doc_type: self.doctype.clone(),
                count: 1,
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    self.doctype.clone(),
                    vec![Entry {
                        name: "bsn".to_string(),
                        value: Value::Text(bsn.to_string()),
                    }],
                )]),
            };
            Ok(vec![pid])
        }
    }
}
