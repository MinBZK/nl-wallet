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

    const MOCK_BSN_1: &str = "999991771";
    const MOCK_BSN_2: &str = "999991772";

    pub struct MockBsnLookup;

    #[async_trait]
    impl BsnLookup for MockBsnLookup {
        async fn bsn(&self, _access_token: &str) -> Result<String, digid::Error> {
            if rand::random() {
                Ok(MOCK_BSN_1.to_string())
            } else {
                Ok(MOCK_BSN_2.to_string())
            }
        }
    }

    pub struct MockAttributesLookup;

    // ISO/IEC 5218
    #[allow(dead_code)]
    enum Gender {
        Unknown,
        Male,
        Female,
        NotApplicable,
    }

    impl From<Gender> for Value {
        fn from(value: Gender) -> Value {
            use Gender::*;
            let value = match value {
                Unknown => 0,
                Male => 1,
                Female => 2,
                NotApplicable => 9,
            };
            Value::Integer(value.into())
        }
    }

    const PID_BSN: &str = "bsn";

    const PID_FAMILY_NAME: &str = "family_name";
    const PID_GIVEN_NAME: &str = "given_name";
    const PID_BIRTH_DATE: &str = "birth_date";
    const PID_AGE_OVER_18: &str = "age_over_18";
    // const PID_AGE_OVER_NN: &str = "age_over_NN";
    // const PID_AGE_IN_YEARS: &str = "age_in_years";
    // const PID_AGE_BIRTH_YEAR: &str = "age_birth_year";
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

    #[derive(Default)]
    struct PersonAttributes {
        bsn: String,
        family_name: String,
        given_name: String,
        birth_date: chrono::NaiveDate,
        age_over_18: bool,
        // age_over_NN: Option<bool>,
        // age_in_years: Option<u32>,
        // age_birth_year: Option<u32>,
        unique_id: String,
        family_name_birth: Option<String>,
        given_name_birth: Option<String>,
        birth_place: Option<String>,
        birth_country: Option<String>,
        birth_state: Option<String>,
        birth_city: Option<String>,
        gender: Option<Gender>,
        nationality: Option<String>,
    }

    impl From<PersonAttributes> for Vec<Entry> {
        fn from(value: PersonAttributes) -> Vec<Entry> {
            vec![
                Entry {
                    name: PID_BSN.to_string(),
                    value: Value::Text(value.bsn),
                }
                .into(),
                Entry {
                    name: PID_FAMILY_NAME.to_string(),
                    value: Value::Text(value.family_name),
                }
                .into(),
                Entry {
                    name: PID_GIVEN_NAME.to_string(),
                    value: Value::Text(value.given_name),
                }
                .into(),
                Entry {
                    name: PID_BIRTH_DATE.to_string(),
                    value: Value::Text(value.birth_date.format("%Y-%m-%d").to_string()),
                }
                .into(),
                Entry {
                    name: PID_AGE_OVER_18.to_string(),
                    value: Value::Bool(value.age_over_18),
                }
                .into(),
                Entry {
                    name: PID_UNIQUE_ID.to_string(),
                    value: Value::Text(value.unique_id),
                }
                .into(),
                value.family_name_birth.map(|v| Entry {
                    name: PID_FAMILY_NAME_BIRTH.to_string(),
                    value: Value::Text(v),
                }),
                value.given_name_birth.map(|v| Entry {
                    name: PID_GIVEN_NAME_BIRTH.to_string(),
                    value: Value::Text(v),
                }),
                value.birth_place.map(|v| Entry {
                    name: PID_BIRTH_PLACE.to_string(),
                    value: Value::Text(v),
                }),
                value.birth_country.map(|v| Entry {
                    name: PID_BIRTH_COUNTRY.to_string(),
                    // TODO according to ISO 3166-1
                    value: Value::Text(v),
                }),
                value.birth_state.map(|v| Entry {
                    name: PID_BIRTH_STATE.to_string(),
                    value: Value::Text(v),
                }),
                value.birth_city.map(|v| Entry {
                    name: PID_BIRTH_CITY.to_string(),
                    value: Value::Text(v),
                }),
                value.gender.map(|v| Entry {
                    name: PID_GENDER.to_string(),
                    // TODO must be int according to ISO/IEC 5218
                    value: v.into(),
                }),
                value.nationality.map(|v| Entry {
                    name: PID_NATIONALITY.to_string(),
                    // TODO according to ISO 3166-1
                    value: Value::Text(v),
                }),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
    }

    #[derive(Default)]
    struct ResidentAttributes {
        address: Option<String>,
        country: Option<String>,
        state: Option<String>,
        city: Option<String>,
        postal_code: Option<String>,
        street: Option<String>,
        house_number: Option<String>,
    }

    impl From<ResidentAttributes> for Vec<Entry> {
        fn from(value: ResidentAttributes) -> Vec<Entry> {
            vec![
                value.address.map(|v| Entry {
                    name: PID_RESIDENT_ADDRESS.to_string(),
                    value: Value::Text(v),
                }),
                value.country.map(|v| Entry {
                    name: PID_RESIDENT_COUNTRY.to_string(),
                    value: Value::Text(v),
                }),
                value.state.map(|v| Entry {
                    name: PID_RESIDENT_STATE.to_string(),
                    value: Value::Text(v),
                }),
                value.city.map(|v| Entry {
                    name: PID_RESIDENT_CITY.to_string(),
                    value: Value::Text(v),
                }),
                value.postal_code.map(|v| Entry {
                    name: PID_RESIDENT_POSTAL_CODE.to_string(),
                    value: Value::Text(v),
                }),
                value.street.map(|v| Entry {
                    name: PID_RESIDENT_STREET.to_string(),
                    value: Value::Text(v),
                }),
                value.house_number.map(|v| Entry {
                    name: PID_RESIDENT_HOUSE_NUMBER.to_string(),
                    value: Value::Text(v),
                }),
            ]
            .into_iter()
            .flatten()
            .collect()
        }
    }

    const MOCK_PID_DOCTYPE: &str = "com.example.pid";
    const MOCK_ADDRESS_DOCTYPE: &str = "com.example.address";

    impl AttributesLookup for MockAttributesLookup {
        fn attributes(&self, bsn: &str) -> Vec<UnsignedMdoc> {
            match bsn {
                MOCK_BSN_1 => vec![
                    UnsignedMdoc {
                        doc_type: MOCK_PID_DOCTYPE.to_string(),
                        copy_count: 1,
                        valid_from: Tdate::now(),
                        valid_until: Utc::now().add(Days::new(365)).into(),
                        attributes: IndexMap::from([(
                            MOCK_PID_DOCTYPE.to_string(),
                            PersonAttributes {
                                unique_id: "1".to_string(),
                                bsn: bsn.to_owned(),
                                given_name: "Johannes Frederik".to_string(),
                                family_name: "Van Waarde".to_string(),
                                gender: Some(Gender::Male),
                                birth_date: chrono::NaiveDate::parse_from_str("1995-09-21", "%Y-%m-%d").unwrap(),
                                age_over_18: true,
                                birth_country: Some("NL".to_string()),
                                birth_city: Some("Leiden".to_string()),
                                nationality: Some("NL".to_string()),
                                ..PersonAttributes::default()
                            }
                            .into(),
                        )]),
                    },
                    UnsignedMdoc {
                        doc_type: MOCK_ADDRESS_DOCTYPE.to_string(),
                        copy_count: 1,
                        valid_from: Tdate::now(),
                        valid_until: Utc::now().add(Days::new(365)).into(),
                        attributes: IndexMap::from([(
                            MOCK_ADDRESS_DOCTYPE.to_string(),
                            ResidentAttributes {
                                street: Some("Weena".to_string()),
                                house_number: Some("10".to_string()),
                                postal_code: Some("3012 CM".to_string()),
                                city: Some("Rotterdam".to_string()),
                                ..ResidentAttributes::default()
                            }
                            .into(),
                        )]),
                    },
                ],
                MOCK_BSN_2 => vec![
                    UnsignedMdoc {
                        doc_type: MOCK_PID_DOCTYPE.to_string(),
                        copy_count: 1,
                        valid_from: Tdate::now(),
                        valid_until: Utc::now().add(Days::new(365)).into(),
                        attributes: IndexMap::from([(
                            MOCK_PID_DOCTYPE.to_string(),
                            PersonAttributes {
                                unique_id: "2".to_string(),
                                bsn: bsn.to_owned(),
                                given_name: "Willeke Liselotte".to_string(),
                                family_name: "De Bruijn".to_string(),
                                family_name_birth: Some("Molenaar".to_string()),
                                gender: Some(Gender::Female),
                                birth_date: chrono::NaiveDate::parse_from_str("1997-05-10", "%Y-%m-%d").unwrap(),
                                age_over_18: true,
                                birth_country: Some("NL".to_string()),
                                birth_city: Some("Delft".to_string()),
                                nationality: Some("NL".to_string()),
                                ..PersonAttributes::default()
                            }
                            .into(),
                        )]),
                    },
                    UnsignedMdoc {
                        doc_type: MOCK_ADDRESS_DOCTYPE.to_string(),
                        copy_count: 1,
                        valid_from: Tdate::now(),
                        valid_until: Utc::now().add(Days::new(365)).into(),
                        attributes: IndexMap::from([(
                            MOCK_ADDRESS_DOCTYPE.to_string(),
                            ResidentAttributes {
                                street: Some("Turfmarkt".to_string()),
                                house_number: Some("147".to_string()),
                                postal_code: Some("2511 DP".to_string()),
                                city: Some("Den Haag".to_string()),
                                ..ResidentAttributes::default()
                            }
                            .into(),
                        )]),
                    },
                ],
                &_ => unreachable!(),
            }
        }
    }
}
