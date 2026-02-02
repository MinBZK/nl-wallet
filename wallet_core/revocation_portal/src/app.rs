use std::path::Path;
use std::sync::Arc;
use std::sync::LazyLock;

use askama::Template;
use askama_web::WebTemplate;
use axum::Form;
use axum::Router;
use axum::extract::State;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::middleware;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use axum::routing::post;
use axum_csrf::CsrfConfig;
use axum_csrf::CsrfLayer;
use axum_csrf::CsrfToken;
use axum_csrf::Key;
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use itertools::Itertools;
use serde::Deserialize;
use strfmt::strfmt;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;

use crypto::SymmetricKey;
use crypto::utils::sha256;
use http_utils::health::create_health_router;
use server_utils::log_requests::log_request_response;
use utils::path::prefix_local_path;
use web_utils::LANGUAGE_JS_SHA256;
use web_utils::headers::set_content_security_policy;
use web_utils::headers::set_static_cache_control;
use web_utils::language::Language;

use crate::revocation_client::RevocationClient;
use crate::translations::TRANSLATIONS;
use crate::translations::Words;
use crate::translations::chrono_locale;

struct ApplicationState<C> {
    revocation_client: C,
}

static CSP_HEADER: LazyLock<String> = LazyLock::new(|| {
    let script_src = format!(
        "'sha256-{}' 'sha256-{}' 'sha256-{}'",
        *LANGUAGE_JS_SHA256, *PORTAL_JS_SHA256, *LOKALIZE_JS_SHA256
    );

    format!(
        "default-src 'self'; script-src {script_src}; img-src 'self' data:; font-src 'self' data:; form-action \
         'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none';"
    )
});

pub static PORTAL_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(sha256(include_bytes!("../assets/portal.js"))));

pub static LOKALIZE_JS_SHA256: LazyLock<String> =
    LazyLock::new(|| BASE64_STANDARD.encode(sha256(include_bytes!("../assets/lokalize.js"))));

#[derive(Deserialize)]
struct DeleteForm {
    csrf_token: String,
    deletion_code: String,
    language: String,
}

pub fn create_router<C>(cookie_encryption_key: &SymmetricKey, log_requests: bool, revocation_client: C) -> Router
where
    C: RevocationClient + Clone + Sync + 'static,
{
    let application_state = Arc::new(ApplicationState { revocation_client });

    let csrf_config = CsrfConfig::default().with_key(Some(Key::from(cookie_encryption_key.as_ref())));

    let mut app = Router::new()
        .route("/support/delete", get(index::<C>))
        .route("/support/delete", post(delete_wallet::<C>))
        .fallback_service(
            ServiceBuilder::new()
                .layer(middleware::from_fn(set_static_cache_control))
                .service(
                    ServeDir::new(prefix_local_path(Path::new("assets"))).fallback(
                        ServiceBuilder::new()
                            .service(ServeDir::new(prefix_local_path(Path::new("../lib/web_utils/assets"))))
                            .not_found_service({ StatusCode::NOT_FOUND }.into_service()),
                    ),
                ),
        )
        .with_state(Arc::clone(&application_state))
        .layer(CsrfLayer::new(csrf_config))
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(|req, next| {
            set_content_security_policy(req, next, &CSP_HEADER)
        }));

    if log_requests {
        app = app.layer(axum::middleware::from_fn(log_request_response));
    }

    app.merge(create_health_router([]))
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
    language_js_sha256: &'a str,
    portal_js_sha256: &'a str,
    lokalize_js_sha256: &'a str,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate<'a> {
    base: BaseTemplate<'a>,
    csrf_token: String,
    failed_reason: Option<&'a str>,
}

#[derive(Template, WebTemplate)]
#[template(path = "error.askama", escape = "html", ext = "html")]
struct ErrorTemplate<'a> {
    base: BaseTemplate<'a>,
}

#[derive(Template, WebTemplate)]
#[template(path = "success.askama", escape = "html", ext = "html")]
struct SuccessTemplate<'a> {
    base: BaseTemplate<'a>,
    revoked_at_rfc3339: String,
    success_message: String,
    success_message_template: String,
}

async fn index<C: RevocationClient>(
    State(_state): State<Arc<ApplicationState<C>>>,
    language: Language,
    token: CsrfToken,
) -> Response {
    let base = BaseTemplate {
        selected_lang: language,
        trans: &TRANSLATIONS[language],
        available_languages: &Language::iter().collect_vec(),
        language_js_sha256: &LANGUAGE_JS_SHA256,
        portal_js_sha256: &PORTAL_JS_SHA256,
        lokalize_js_sha256: &LOKALIZE_JS_SHA256,
    };

    let csrf_token = match token.authenticity_token() {
        Ok(csrf_token) => csrf_token,
        Err(err) => {
            warn!("Error getting hashed csrf token: {}", err);
            return (StatusCode::UNPROCESSABLE_ENTITY, ErrorTemplate { base }).into_response();
        }
    };

    let template = IndexTemplate {
        base,
        csrf_token,
        failed_reason: None,
    }
    .into_response();

    (token, template).into_response()
}

async fn delete_wallet<C: RevocationClient>(
    State(state): State<Arc<ApplicationState<C>>>,
    language: Language,
    token: CsrfToken,
    Form(delete_form): Form<DeleteForm>,
) -> Response {
    let language = delete_form.language.parse().unwrap_or(language);

    let base = BaseTemplate {
        selected_lang: language,
        trans: &TRANSLATIONS[language],
        available_languages: &Language::iter().collect_vec(),
        language_js_sha256: &LANGUAGE_JS_SHA256,
        portal_js_sha256: &PORTAL_JS_SHA256,
        lokalize_js_sha256: &LOKALIZE_JS_SHA256,
    };

    if let Err(err) = token.verify(&delete_form.csrf_token) {
        warn!("CSRF error: {}", err);
        return (StatusCode::UNPROCESSABLE_ENTITY, ErrorTemplate { base }).into_response();
    }

    match delete_form.deletion_code.parse() {
        Ok(deletion_code) => match state.revocation_client.revoke(deletion_code).await {
            Ok(result) => {
                let date = result
                    .revoked_at
                    .format_localized(TRANSLATIONS[language].date_format, chrono_locale(language))
                    .to_string();
                let time = result
                    .revoked_at
                    .format_localized(TRANSLATIONS[language].time_format, chrono_locale(language))
                    .to_string();

                SuccessTemplate {
                    base,
                    revoked_at_rfc3339: result.revoked_at.to_rfc3339(),
                    success_message: strfmt!(TRANSLATIONS[language].success_wb_confirmation, date, time)
                        .expect("success message formatting should succeed"),
                    success_message_template: String::from(TRANSLATIONS[language].success_wb_confirmation),
                }
                .into_response()
            }
            Err(err) => {
                warn!("Error revoking wallet: {}", err);
                ErrorTemplate { base }.into_response()
            }
        },
        Err(err) => {
            warn!("Error parsing deletion code: {}", err);

            let csrf_token = match token.authenticity_token() {
                Ok(csrf_token) => csrf_token,
                Err(err) => {
                    warn!("Error getting hashed csrf token: {}", err);
                    return ErrorTemplate { base }.into_response();
                }
            };

            IndexTemplate {
                base,
                csrf_token,
                failed_reason: Some(TRANSLATIONS[language].delete_code_incorrect),
            }
            .into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str;

    use axum::body::Body;
    use axum::http::Request;
    use axum::http::StatusCode;
    use axum::http::header;
    use chrono::DateTime;
    use chrono::TimeZone;
    use chrono::Utc;
    use rstest::rstest;
    use scraper::Html;
    use scraper::Selector;
    use tower::Service;
    use tower::ServiceExt;

    use crypto::utils::random_bytes;
    use web_utils::language::Language;

    use crate::DeletionCode;
    use crate::revocation_client::RevocationError;
    use crate::revocation_client::RevocationResult;
    use crate::revocation_client::tests::MockRevocationClient;
    use crate::translations::TRANSLATIONS;

    use super::*;

    #[derive(Clone)]
    struct FixedRevocationClient {
        revoked_at: DateTime<Utc>,
    }

    impl RevocationClient for FixedRevocationClient {
        async fn revoke(&self, _deletion_code: DeletionCode) -> Result<RevocationResult, RevocationError> {
            Ok(RevocationResult {
                revoked_at: self.revoked_at,
            })
        }
    }

    async fn get_csrf_and_cookie(app: &mut Router) -> (String, String) {
        let response = app
            .call(Request::builder().uri("/support/delete").body(Body::empty()).unwrap())
            .await
            .unwrap();

        let cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let body = axum_body_bytes(response.into_body()).await;
        let html = Html::parse_document(str::from_utf8(&body).unwrap());
        let selector = Selector::parse("input[name='csrf_token']").unwrap();
        let token = html
            .select(&selector)
            .next()
            .unwrap()
            .value()
            .attr("value")
            .unwrap()
            .to_string();

        (token, cookie)
    }

    async fn post_delete_with_lang(app: &mut Router, deletion_code: &str, lang: &str) -> Response {
        let (token, cookie) = get_csrf_and_cookie(app).await;

        let form = [
            ("deletion_code", deletion_code),
            ("csrf_token", &token),
            ("language", lang),
        ];
        let body = serde_urlencoded::to_string(form).unwrap();

        app.oneshot(
            Request::builder()
                .method("POST")
                .uri("/support/delete")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .header(header::COOKIE, cookie)
                .body(Body::from(body))
                .unwrap(),
        )
        .await
        .unwrap()
    }

    #[rstest]
    #[case(Language::En, "en")]
    #[case(Language::Nl, "nl")]
    #[tokio::test]
    async fn test_index_with_language_param(#[case] language: Language, #[case] lang_param: &str) {
        let client = MockRevocationClient::default();
        let app = create_router(&random_bytes(64).into(), false, client);

        let response = app
            .oneshot(
                Request::builder()
                    .uri(format!("/support/delete?lang={lang_param}"))
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum_body_bytes(response.into_body()).await;
        let html = Html::parse_document(str::from_utf8(&body).unwrap());

        // Check the visible H1 tag for the localized title
        let h1_selector = Selector::parse("h1").unwrap();
        let h1_text = html.select(&h1_selector).next().unwrap().inner_html();

        assert!(h1_text.trim().contains(TRANSLATIONS[language].delete_title));
    }

    #[tokio::test]
    async fn test_delete_wallet_fails_for_missing_csrf_token() {
        let client = MockRevocationClient::default();
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let (_token, cookie) = get_csrf_and_cookie(&mut app).await;

        let form = [("deletion_code", "C20C-KF0R-D32B-A5E3-2X"), ("language", "nl")];
        let body = serde_urlencoded::to_string(form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/support/delete")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_delete_wallet_fails_for_missing_csrf_cookie() {
        let client = MockRevocationClient::default();
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let (token, _cookie) = get_csrf_and_cookie(&mut app).await;

        let form = [
            ("deletion_code", "C20C-KF0R-D32B-A5E3-2X"),
            ("csrf_token", &token),
            ("language", "nl"),
        ];
        let body = serde_urlencoded::to_string(form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/support/delete")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_delete_wallet_fails_for_wrong_csrf_values() {
        let client = MockRevocationClient::default();
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let (_token, cookie) = get_csrf_and_cookie(&mut app).await;

        let form = [
            ("deletion_code", "C20C-KF0R-D32B-A5E3-2X"),
            ("csrf_token", "this_csrf_is_wrong"),
            ("language", "nl"),
        ];
        let body = serde_urlencoded::to_string(form).unwrap();

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/support/delete")
                    .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                    .header(header::COOKIE, cookie)
                    .body(Body::from(body))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_delete_wallet_invalid_code_shows_error_on_index() {
        let client = MockRevocationClient::default();
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let response = post_delete_with_lang(&mut app, "invalid", "nl").await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum_body_bytes(response.into_body()).await;
        let html = Html::parse_document(str::from_utf8(&body).unwrap());

        let input_selector = Selector::parse("input[data-test='deletion-code-input'].invalid").unwrap();
        assert!(html.select(&input_selector).next().is_some());

        let error_selector = Selector::parse("[data-test='error-message'].visible").unwrap();
        let error_text = html.select(&error_selector).next().unwrap().inner_html();

        assert!(error_text.contains(TRANSLATIONS[Language::Nl].delete_code_incorrect));
    }

    #[tokio::test]
    async fn test_delete_wallet_revocation_failure_shows_error_template() {
        let client = MockRevocationClient::new_failing();
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let response = post_delete_with_lang(&mut app, "C20C-KF0R-D32B-A5E3-2X", "nl").await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum_body_bytes(response.into_body()).await;
        let html = Html::parse_document(str::from_utf8(&body).unwrap());

        // Verify the ErrorTemplate section is present
        let error_container_selector = Selector::parse("section[data-test='error-container']").unwrap();
        html.select(&error_container_selector)
            .next()
            .expect("Error container missing");

        // Verify the H1 within the error page contains the correct title
        let h1_selector = Selector::parse("h1").unwrap();
        let h1_text = html.select(&h1_selector).next().unwrap().inner_html();
        assert!(h1_text.trim().contains(TRANSLATIONS[Language::Nl].error_title));

        // Ensure the deletion form from IndexTemplate is NOT present
        let form_selector = Selector::parse("form.delete-form").unwrap();
        assert!(html.select(&form_selector).next().is_none());
    }

    #[tokio::test]
    async fn test_delete_wallet_success_shows_success_template() {
        let client = MockRevocationClient::default();
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let response = post_delete_with_lang(&mut app, "C20C-KF0R-D32B-A5E3-2X", "nl").await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum_body_bytes(response.into_body()).await;
        let body_str = str::from_utf8(&body).unwrap();
        let html = Html::parse_document(body_str);

        let h1_selector = Selector::parse("h1").unwrap();
        let h1_text = html.select(&h1_selector).next().unwrap().inner_html();
        assert!(h1_text.trim().contains(TRANSLATIONS[Language::Nl].success_title));

        assert!(body_str.contains(TRANSLATIONS[Language::Nl].success_wb_confirmation));

        let back_link_selector = Selector::parse("a.back-link").unwrap();
        let back_link = html.select(&back_link_selector).next().expect("Back link should exist");
        assert!(
            back_link
                .inner_html()
                .contains(TRANSLATIONS[Language::Nl].back_to_support)
        );
    }

    #[rstest]
    #[case::nl(Language::Nl, "nl")]
    #[case::en(Language::En, "en")]
    #[tokio::test]
    async fn test_delete_wallet_success_includes_expected_date_time_output(
        #[case] language: Language,
        #[case] lang_param: &str,
    ) {
        // Pick a fixed timestamp that is unlikely to be affected by DST surprises.
        // (Still UTC, but a stable value keeps the test deterministic.)
        let revoked_at = Utc.with_ymd_and_hms(2026, 1, 2, 3, 4, 5).single().unwrap();

        let client = FixedRevocationClient { revoked_at };
        let mut app = create_router(&random_bytes(64).into(), false, client);

        let response = post_delete_with_lang(&mut app, "C20C-KF0R-D32B-A5E3-2X", lang_param).await;
        assert_eq!(response.status(), StatusCode::OK);

        let body = axum_body_bytes(response.into_body()).await;
        let body_str = str::from_utf8(&body).unwrap();

        // 1) The raw RFC3339 timestamp should be present in the success page output
        //    (SuccessTemplate.revoked_at_rfc3339).
        let expected_rfc3339 = revoked_at.to_rfc3339();
        assert!(
            body_str.contains(&expected_rfc3339),
            "expected body to contain revoked_at RFC3339 ({expected_rfc3339}), got:\n{body_str}"
        );

        // 2) The handler formats localized date/time and interpolates them into a success message.
        let expected_date = revoked_at
            .format_localized(TRANSLATIONS[language].date_format, chrono_locale(language))
            .to_string();
        let expected_time = revoked_at
            .format_localized(TRANSLATIONS[language].time_format, chrono_locale(language))
            .to_string();

        assert!(
            body_str.contains(&expected_date),
            "expected body to contain formatted date ({expected_date}), got:\n{body_str}"
        );
        assert!(
            body_str.contains(&expected_time),
            "expected body to contain formatted time ({expected_time}), got:\n{body_str}"
        );
    }

    async fn axum_body_bytes(body: axum::body::Body) -> Vec<u8> {
        axum::body::to_bytes(body, usize::MAX).await.unwrap().to_vec()
    }
}
