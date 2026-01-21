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
use itertools::Itertools;
use serde::Deserialize;
use server_utils::log_requests::log_request_response;
use strum::IntoEnumIterator;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;
use tracing::warn;

use http_utils::health::create_health_router;
use readable_identifier::ReadableIdentifierParseError;
use utils::path::prefix_local_path;
use web_utils::LANGUAGE_JS_SHA256;
use web_utils::headers::set_content_security_policy;
use web_utils::headers::set_static_cache_control;
use web_utils::language::Language;

use crate::DeletionCode;
use crate::revocation_client::RevocationClient;
use crate::settings::Settings;
use crate::translations::TRANSLATIONS;
use crate::translations::Words;

struct ApplicationState<C> {
    revocation_client: C,
}

static CSP_HEADER: LazyLock<String> = LazyLock::new(|| {
    let script_src = format!("'sha256-{}'", *LANGUAGE_JS_SHA256);

    format!(
        "default-src 'self'; script-src 'self' {script_src}; img-src 'self' data:; font-src 'self' data:; form-action \
         'self'; frame-ancestors 'none'; object-src 'none'; base-uri 'none';"
    )
});

#[derive(Deserialize)]
struct DeleteForm {
    deletion_code: String,
}

pub fn create_router<C>(settings: &Settings, revocation_client: C) -> Router
where
    C: RevocationClient + Clone + Sync + 'static,
{
    let application_state = Arc::new(ApplicationState { revocation_client });

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
        .layer(TraceLayer::new_for_http())
        .layer(middleware::from_fn(|req, next| {
            set_content_security_policy(req, next, &CSP_HEADER)
        }));

    if settings.log_requests {
        app = app.layer(axum::middleware::from_fn(log_request_response));
    }

    app.merge(create_health_router([]))
}

struct BaseTemplate<'a> {
    selected_lang: Language,
    trans: &'a Words<'a>,
    available_languages: &'a [Language],
    language_js_sha256: &'a str,
}

#[derive(Template, WebTemplate)]
#[template(path = "index.askama", escape = "html", ext = "html")]
struct IndexTemplate<'a> {
    base: BaseTemplate<'a>,
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
}

async fn index<C: RevocationClient>(State(_state): State<Arc<ApplicationState<C>>>, language: Language) -> Response {
    IndexTemplate {
        base: BaseTemplate {
            selected_lang: language,
            trans: &TRANSLATIONS[language],
            available_languages: &Language::iter().collect_vec(),
            language_js_sha256: &LANGUAGE_JS_SHA256,
        },
        failed_reason: None,
    }
    .into_response()
}

async fn delete_wallet<C: RevocationClient>(
    State(state): State<Arc<ApplicationState<C>>>,
    language: Language,
    Form(delete_form): Form<DeleteForm>,
) -> Response {
    let base = BaseTemplate {
        selected_lang: language,
        trans: &TRANSLATIONS[language],
        available_languages: &Language::iter().collect_vec(),
        language_js_sha256: &LANGUAGE_JS_SHA256,
    };

    let parse_result: Result<DeletionCode, ReadableIdentifierParseError> = delete_form.deletion_code.parse();

    match parse_result {
        Ok(deletion_code) => match state.revocation_client.revoke(deletion_code).await {
            Ok(()) => SuccessTemplate { base }.into_response(),
            Err(err) => {
                warn!("Error revoking wallet: {}", err);

                ErrorTemplate { base }.into_response()
            }
        },
        Err(err) => {
            warn!("Error parsing deletion code: {}", err);

            IndexTemplate {
                base,
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
    use rstest::rstest;
    use scraper::Html;
    use scraper::Selector;
    use tower::ServiceExt;

    use web_utils::language::Language;

    use crate::revocation_client::tests::MockRevocationClient;
    use crate::settings::Settings;
    use crate::translations::TRANSLATIONS;

    use super::*;

    #[rstest]
    #[case(Language::En, "en")]
    #[case(Language::Nl, "nl")]
    #[tokio::test]
    async fn test_index_with_language_param(#[case] language: Language, #[case] lang_param: &str) {
        let settings = Settings::new().unwrap();
        let client = MockRevocationClient::default();
        let app = create_router(&settings, client);

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
    async fn test_delete_wallet_invalid_code_shows_error_on_index() {
        let settings = Settings::new().unwrap();
        let client = MockRevocationClient::default();
        let app = create_router(&settings, client);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/support/delete")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("deletion_code=invalid"))
                    .unwrap(),
            )
            .await
            .unwrap();

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
        let settings = Settings::new().unwrap();
        let client = MockRevocationClient::new_failing();
        let app = create_router(&settings, client);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/support/delete")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("deletion_code=C20C-KF0R-D32B-A5E3-2X"))
                    .unwrap(),
            )
            .await
            .unwrap();

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
        let settings = Settings::new().unwrap();
        let client = MockRevocationClient::default();
        let app = create_router(&settings, client);

        let response = app
            .oneshot(
                Request::builder()
                    .method("POST")
                    .uri("/support/delete")
                    .header("content-type", "application/x-www-form-urlencoded")
                    .body(Body::from("deletion_code=C20C-KF0R-D32B-A5E3-2X"))
                    .unwrap(),
            )
            .await
            .unwrap();

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

    async fn axum_body_bytes(body: axum::body::Body) -> Vec<u8> {
        axum::body::to_bytes(body, usize::MAX).await.unwrap().to_vec()
    }
}
