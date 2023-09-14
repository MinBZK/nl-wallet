use std::sync::Arc;

use async_trait::async_trait;
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
use tower_http::trace::TraceLayer;
use tracing::{debug, error, warn};

use nl_wallet_mdoc::{
    basic_sa_ext::UnsignedMdoc,
    issuer::{self, MemorySessionStore, PrivateKey, SingleKeyRing},
    ServiceEngagement,
};

use crate::{digid, settings::Settings};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("OIDC client error: {0}")]
    Digid(#[from] digid::Error),
    #[error("starting mdoc session failed: {0}")]
    StartMdoc(#[source] nl_wallet_mdoc::Error),
    #[error("mdoc session error: {0}")]
    Mdoc(#[source] nl_wallet_mdoc::Error),
}

// TODO: Implement proper error handling.
impl IntoResponse for Error {
    fn into_response(self) -> Response {
        warn!("error result: {:?}", self);
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{}", self)).into_response()
    }
}

/// Given a BSN, determine the attributes to be issued. Contract for the BRP query.
pub trait AttributesLookup {
    fn attributes(&self, bsn: &str) -> Vec<UnsignedMdoc>;
}

/// Given an access token, lookup a BSN: a trait modeling the OIDC [`Client`](crate::openid::Client).
/// Contract for the DigiD bridge.
#[async_trait]
pub trait BsnLookup {
    async fn bsn(&self, access_token: &str) -> Result<String, digid::Error>;
}

struct ApplicationState<A, B> {
    attributes_lookup: A,
    openid_client: B,
    issuer: issuer::Server<SingleKeyRing, MemorySessionStore>,
}

pub async fn create_router<A, B>(settings: Settings, attributes_lookup: A, openid_client: B) -> anyhow::Result<Router>
where
    A: AttributesLookup + Send + Sync + 'static,
    B: BsnLookup + Send + Sync + 'static,
{
    debug!("DigiD issuer discovered, starting HTTP server");

    let key = SingleKeyRing(PrivateKey::from_der(
        &BASE64_STANDARD.decode(&settings.issuer_key.private_key)?,
        &BASE64_STANDARD.decode(&settings.issuer_key.certificate)?,
    )?);

    let mut public_url = settings.public_url;
    if !public_url.as_str().ends_with('/') {
        // If the url does not have a trailing slash then .join() will remove its last path segment
        // before appending its argument (which is also why we can't use .join() for appending this slash).
        // We can use .unwrap() because this errors only happens "if the scheme and `:` delimiter
        // are not followed by a `/` slash".
        public_url.path_segments_mut().unwrap().push("/");
    }
    let public_url = public_url.join("mdoc/")?;

    let application_state = Arc::new(ApplicationState {
        attributes_lookup,
        openid_client,
        issuer: issuer::Server::new(public_url, key, MemorySessionStore::new()),
    });

    let app = Router::new()
        .route("/mdoc/:session_token", post(mdoc_route))
        .route("/start", post(start_route))
        .layer(TraceLayer::new_for_http())
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
        .await
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
    // Using the access_token that the user specified, lookup the user's BSN at the OIDC IdP (DigiD bridge)
    let access_token = authorization_header.token();
    let bsn: String = state
        .openid_client
        .bsn(access_token)
        .inspect_err(|error| error!("error while looking up BSN: {}", error))
        .await?;

    // Start the session, and return the initial mdoc protocol message (containing the URL at which the wallet can
    // find us) to the wallet
    let attributes = state.attributes_lookup.attributes(&bsn);
    let service_engagement = state.issuer.new_session(attributes).map_err(Error::StartMdoc)?;

    Ok(Json(service_engagement))
}

/// Mock implementations of the two traits abstracting other components.
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

    use crate::digid;

    use super::{AttributesLookup, BsnLookup};

    const MOCK_BSN: &str = "999991772";

    pub struct MockBsnLookup;

    #[async_trait]
    impl BsnLookup for MockBsnLookup {
        async fn bsn(&self, _: &str) -> Result<String, digid::Error> {
            Ok(MOCK_BSN.to_string())
        }
    }

    pub struct MockAttributesLookup;

    const PID_FAMILY_NAME: &str = "family_name";
    const PID_GIVEN_NAME: &str = "given_name";
    const PID_BIRTH_DATE: &str = "birth_date";
    const PID_AGE_OVER_18: &str = "age_over_18";
    const PID_AGE_OVER_NN: &str = "age_over_NN";
    const PID_AGE_IN_YEARS: &str = "age_in_years";
    const PID_AGE_BIRTH_YEAR: &str = "age_birth_year";
    const PID_UNIQUE_ID: &str = "unique_id";
    const PID_FAMILY_NAME_BIRTH: &str = "family_name_birth";
    const PID_GIVEN_NAME_BIRTH: &str = "given_name_birth";
    const PID_BIRTH_PLACE: &str = "birth_place";
    const PID_BIRTH_COUNTRY: &str = "birth_country";
    const PID_BIRTH_STATE: &str = "birth_state";
    const PID_BIRTH_CITY: &str = "birth_city";
    const PID_RESIDENT_ADDRESS: &str = "resident_address";
    const PID_RESIDENT_COUNTRY: &str = "resident_country";
    const PID_RESIDENT_STATE: &str = "resident_state";
    const PID_RESIDENT_CITY: &str = "resident_city";
    const PID_RESIDENT_POSTAL_CODE: &str = "resident_postal_code";
    const PID_RESIDENT_STREET: &str = "resident_street";
    const PID_RESIDENT_HOUSE_NUMBER: &str = "resident_house_number";
    const PID_GENDER: &str = "gender";
    const PID_NATIONALITY: &str = "nationality";

    const PID_ISSUANCE_DATE: &str = "issuance_date";
    const PID_EXPIRY_DATE: &str = "expiry_date";
    const PID_ISSUING_AUTHORITY: &str = "issuing_authority";
    const PID_DOCUMENT_NUMBER: &str = "document_number";
    const PID_ADMINISTRATIVE_NUMBER: &str = "administrative_number";
    const PID_ISSUING_COUNTRY: &str = "issuing_country";
    const PID_ISSUING_JURISDICTION: &str = "issuing_jurisdiction";
    const PID_PORTRAIT: &str = "portrait";
    const PID_PORTRAIT_CAPTURE_DATE: &str = "portrait_capture_date";

    const MOCK_PID_DOCTYPE: &str = "com.example.pid";
    const MOCK_ADDRESS_DOCTYPE: &str = "com.example.address";

    const NL_PID_NAMESPACE_PREFIX: &str = "eu.europa.ec.eudiw.pid.nl";

    impl AttributesLookup for MockAttributesLookup {
        fn attributes(&self, bsn: &str) -> Vec<UnsignedMdoc> {
            let pid = UnsignedMdoc {
                doc_type: MOCK_PID_DOCTYPE.to_string(),
                copy_count: 1,
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([
                    (
                        MOCK_PID_DOCTYPE.to_string(),
                        vec![
                            Entry {
                                name: PID_GIVEN_NAME.to_string(),
                                value: Value::Text("Willeke Liselotte".to_string()),
                            },
                            Entry {
                                name: PID_FAMILY_NAME.to_string(),
                                value: Value::Text("De Bruijn".to_string()),
                            },
                            Entry {
                                name: PID_FAMILY_NAME_BIRTH.to_string(),
                                value: Value::Text("Molenaar".to_string()),
                            },
                            Entry {
                                name: PID_GENDER.to_string(),
                                // TODO must be int according to ISO/IEC 5218
                                value: Value::Text("Vrouw".to_string()),
                            },
                            Entry {
                                name: PID_BIRTH_DATE.to_string(),
                                value: Value::Integer(
                                    chrono::DateTime::parse_from_rfc3339("1997-05-10")
                                        .unwrap()
                                        .timestamp()
                                        .into(),
                                ),
                            },
                            Entry {
                                name: PID_AGE_OVER_18.to_string(),
                                value: Value::Bool(true),
                            },
                            Entry {
                                name: PID_BIRTH_PLACE.to_string(),
                                value: Value::Text("Delft NL".to_string()),
                            },
                            Entry {
                                name: PID_BIRTH_COUNTRY.to_string(),
                                // TODO according to ISO 3166-1
                                value: Value::Text("NL".to_string()),
                            },
                            Entry {
                                name: PID_BIRTH_CITY.to_string(),
                                value: Value::Text("Delft".to_string()),
                            },
                            Entry {
                                name: PID_NATIONALITY.to_string(),
                                // TODO according to ISO 3166-1
                                value: Value::Text("NL".to_string()),
                            },
                        ],
                    ),
                    (
                        format!("{}.{}", NL_PID_NAMESPACE_PREFIX, "1"),
                        vec![Entry {
                            name: "bsn".to_string(),
                            value: Value::Text(bsn.to_string()),
                        }],
                    ),
                ]),
            };
            let address_card = UnsignedMdoc {
                doc_type: MOCK_ADDRESS_DOCTYPE.to_string(),
                copy_count: 1,
                valid_from: Tdate::now(),
                valid_until: Utc::now().add(Days::new(365)).into(),
                attributes: IndexMap::from([(
                    MOCK_ADDRESS_DOCTYPE.to_string(),
                    vec![
                        Entry {
                            name: PID_RESIDENT_STREET.to_string(),
                            value: Value::Text("Turfmarkt".to_string()),
                        },
                        Entry {
                            name: PID_RESIDENT_HOUSE_NUMBER.to_string(),
                            value: Value::Text("147".to_string()),
                        },
                        Entry {
                            name: PID_RESIDENT_POSTAL_CODE.to_string(),
                            value: Value::Text("2511 DP".to_string()),
                        },
                        Entry {
                            name: PID_RESIDENT_CITY.to_string(),
                            value: Value::Text("Den Haag".to_string()),
                        },
                    ],
                )]),
            };
            vec![pid, address_card]
        }
    }
}
