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
use jsonwebtoken::Algorithm;
use jsonwebtoken::Header;
use serde::Deserialize;
use serde::Serialize;
use serde_with::DurationSeconds;
use serde_with::serde_as;

use crypto::EcdsaKey;
use http_utils::urls::HttpsUri;
use jwt::Jwt;
use jwt::error::JwtError;

use crate::status_list::StatusList;

static TOKEN_STATUS_LIST_JWT_TYP: &str = "statuslist+jwt";
static TOKEN_STATUS_LIST_JWT_HEADER: &str = "application/statuslist+jwt";

/// A Status List Token embeds a Status List into a token that is cryptographically signed and protects the integrity of
/// the Status List.
///
/// <https://www.ietf.org/archive/id/draft-ietf-oauth-status-list-12.html#name-status-list-token>
#[derive(Debug, Clone, FromStr, Serialize, Deserialize)]
pub struct StatusListToken(Jwt<StatusListClaims>);

impl StatusListToken {
    pub async fn try_new(
        iat: DateTime<Utc>,
        exp: Option<DateTime<Utc>>,
        sub: HttpsUri,
        ttl: Option<Duration>,
        status_list: StatusList,
        key: &impl EcdsaKey,
    ) -> Result<Self, JwtError> {
        let claims = StatusListClaims {
            iat,
            exp,
            sub,
            ttl,
            status_list,
        };
        let header = Header {
            typ: Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()),
            ..Header::new(Algorithm::ES256)
        };

        let jwt = Jwt::sign(&claims, &header, key).await?;
        Ok(StatusListToken(jwt))
    }
}

#[cfg(feature = "axum")]
impl IntoResponse for StatusListToken {
    fn into_response(self) -> Response {
        ([(CONTENT_TYPE, TOKEN_STATUS_LIST_JWT_HEADER)], self.0.0.to_string()).into_response()
    }
}

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
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

    status_list: StatusList,
}

#[cfg(test)]
mod test {
    use axum::Router;
    use axum::routing::get;
    use p256::ecdsa::SigningKey;
    use p256::elliptic_curve::rand_core::OsRng;
    use serde_json::json;
    use tokio::net::TcpListener;

    use http_utils::urls::BaseUrl;
    use tests_integration::common::wait_for_server;

    use super::*;

    #[tokio::test]
    async fn test_status_list_token() {
        let example_header = json!({
            "alg": "ES256",
            "kid": "12",
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

        let header: Header = serde_json::from_value(example_header).unwrap();
        assert_eq!(header.typ, Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()));

        let claims: StatusListClaims = serde_json::from_value(example_payload).unwrap();

        let key = SigningKey::random(&mut OsRng);
        let signed = StatusListToken::try_new(claims.iat, claims.exp, claims.sub, claims.ttl, claims.status_list, &key)
            .await
            .unwrap();

        let (header, _) = signed
            .0
            .parse_and_verify_with_header(&key.verifying_key().into(), &jwt::validations())
            .unwrap();
        assert_eq!(header.typ, Some(TOKEN_STATUS_LIST_JWT_TYP.to_string()));
    }

    #[cfg(feature = "axum")]
    async fn start_mock_server() -> BaseUrl {
        use crate::status_list::test::EXAMPLE_STATUS_LIST_ONE;

        let listener = TcpListener::bind("localhost:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();

        let token_status_list = StatusListToken::try_new(
            Utc::now(),
            Some(Utc::now() + Duration::from_secs(3600)),
            "https://example.com/statuslists/1".parse().unwrap(),
            Some(Duration::from_secs(43200)),
            EXAMPLE_STATUS_LIST_ONE.to_owned(),
            &SigningKey::random(&mut OsRng),
        )
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
        assert!(!payload.status_list.as_ref().is_empty());
    }
}
