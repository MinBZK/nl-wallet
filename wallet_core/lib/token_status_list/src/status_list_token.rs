use std::time::Duration;

#[cfg(feature = "axum")]
use axum::http::header::CONTENT_TYPE;
#[cfg(feature = "axum")]
use axum::response::IntoResponse;
#[cfg(feature = "axum")]
use axum::response::Response;
use chrono::DateTime;
use chrono::Utc;
use chrono::serde::ts_seconds;
use chrono::serde::ts_seconds_option;
use derive_more::FromStr;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;

use crypto::EcdsaKey;
use http_utils::urls::HttpsUri;
use jwt::Algorithm;
use jwt::Header;
use jwt::UnverifiedJwt;
use jwt::error::JwtError;

use crate::status_list::PackedStatusList;

static TOKEN_STATUS_LIST_JWT_TYP: &str = "statuslist+jwt";
static TOKEN_STATUS_LIST_JWT_HEADER: &str = "application/statuslist+jwt";

/// A Status List Token embeds a Status List into a token that is cryptographically signed and protects the integrity of
/// the Status List.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-list-token>
#[derive(Debug, Clone, FromStr, Serialize, Deserialize)]
pub struct StatusListToken(UnverifiedJwt<StatusListClaims>);

impl StatusListToken {
    pub fn builder(sub: HttpsUri, status_list: PackedStatusList) -> StatusListTokenBuilder {
        StatusListTokenBuilder {
            exp: None,
            sub,
            ttl: None,
            status_list,
        }
    }
}

pub struct StatusListTokenBuilder {
    exp: Option<DateTime<Utc>>,
    sub: HttpsUri,
    ttl: Option<Duration>,
    status_list: PackedStatusList,
}

impl StatusListTokenBuilder {
    pub fn exp(mut self, exp: DateTime<Utc>) -> Self {
        self.exp = Some(exp);
        self
    }

    pub fn ttl(mut self, ttl: Duration) -> Self {
        self.ttl = Some(ttl);
        self
    }

    pub async fn sign(self, key: &impl EcdsaKey) -> Result<StatusListToken, JwtError> {
        let header = Header {
            typ: Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()),
            ..Header::new(Algorithm::ES256)
        };

        let claims = StatusListClaims {
            iat: Utc::now(),
            exp: self.exp,
            sub: self.sub,
            ttl: self.ttl,
            status_list: self.status_list,
        };

        let jwt = UnverifiedJwt::sign(&claims, &header, key).await?;
        Ok(StatusListToken(jwt))
    }
}

#[cfg(feature = "axum")]
impl IntoResponse for StatusListToken {
    fn into_response(self) -> Response {
        ([(CONTENT_TYPE, TOKEN_STATUS_LIST_JWT_HEADER)], self.0.to_string()).into_response()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct StatusListClaims {
    #[serde(with = "ts_seconds")]
    iat: DateTime<Utc>,

    #[serde(with = "ts_seconds_option")]
    exp: Option<DateTime<Utc>>,

    /// The sub (subject) claim MUST specify the URI of the Status List Token. The value MUST be equal to that of the
    /// `uri` claim contained in the `status_list` claim of the Referenced Token
    sub: HttpsUri,

    /// If present, MUST specify the maximum amount of time, in seconds, that the Status List Token can be cached by a
    /// consumer before a fresh copy SHOULD be retrieved.
    #[serde_as(as = "Option<DurationSeconds<u64>>")]
    ttl: Option<Duration>,

    status_list: PackedStatusList,
}

#[cfg(test)]
mod test {
    use jwt::DEFAULT_VALIDATIONS;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::rand_core::OsRng;
    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn test_status_list_token() {
        let example_header = json!({
            "alg": "ES256",
            "typ": "statuslist+jwt"
        });
        let example_payload = json!({
            "exp": 2291720170_i64,
            "iat": 1686920170,
            "status_list": {
                "bits": 1,
                "lst": "eNrbuRgAAhcBXQ"
            },
            "sub": "https://example.com/statuslists/1",
            "ttl": 43200
        });

        let expected_header: Header = serde_json::from_value(example_header).unwrap();
        assert_eq!(expected_header.typ, Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()));

        let expected_claims: StatusListClaims = serde_json::from_value(example_payload).unwrap();

        let key = SigningKey::random(&mut OsRng);
        let signed = StatusListToken::builder(expected_claims.sub.clone(), expected_claims.status_list.clone())
            .exp(expected_claims.exp.unwrap())
            .ttl(expected_claims.ttl.unwrap())
            .sign(&key)
            .await
            .unwrap();

        let (header, claims) = signed
            .0
            .parse_and_verify_with_header(&key.verifying_key().into(), &DEFAULT_VALIDATIONS)
            .unwrap();
        assert_eq!(header, expected_header);
        // the `iat` claim is set when signing the token
        assert_eq!(claims.status_list, expected_claims.status_list);
        assert_eq!(claims.sub, expected_claims.sub);
        assert_eq!(claims.ttl, expected_claims.ttl);
        assert_eq!(claims.exp, expected_claims.exp);
    }

    #[cfg(feature = "axum")]
    async fn start_mock_server() -> http_utils::urls::BaseUrl {
        use axum::Router;
        use axum::routing::get;
        use tokio::net::TcpListener;

        use http_utils::urls::BaseUrl;
        use tests_integration::common::wait_for_server;

        use crate::status_list::test::EXAMPLE_STATUS_LIST_ONE;

        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let token_status_list = StatusListToken::builder(
            "https://example.com/statuslists/1".parse().unwrap(),
            EXAMPLE_STATUS_LIST_ONE.to_owned().pack(),
        )
        .exp(Utc::now() + Duration::from_secs(3600))
        .ttl(Duration::from_secs(43200))
        .sign(&SigningKey::random(&mut OsRng))
        .await
        .unwrap();

        let app = Router::new()
            .route("/", get(move || async { token_status_list }))
            .into_make_service();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        let url: BaseUrl = format!("http://localhost:{port}/").as_str().parse().unwrap();
        wait_for_server(url.clone(), std::iter::empty()).await;
        url
    }

    #[tokio::test]
    #[cfg(feature = "axum")]
    async fn test_token_status_list_into_response() {
        let url = start_mock_server().await;

        let response = reqwest::Client::new()
            .get(url.into_inner())
            .send()
            .await
            .expect("Failed to send request");

        assert_eq!(
            response.headers().get("Content-Type").unwrap(),
            TOKEN_STATUS_LIST_JWT_HEADER
        );
        let status_list_token: StatusListToken = response.text().await.unwrap().parse().unwrap();
        let (header, payload) = status_list_token.0.dangerous_parse_unverified().unwrap();
        assert_eq!(header.typ.unwrap(), TOKEN_STATUS_LIST_JWT_TYP);
        assert!(!payload.status_list.is_empty());
    }
}
