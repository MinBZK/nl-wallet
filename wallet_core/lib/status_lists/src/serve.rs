use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::string::ToString;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::Path;
use axum::extract::Request;
use axum::extract::State;
use axum::http::HeaderMap;
use axum::http::HeaderValue;
use axum::http::StatusCode;
use axum::http::header;
use axum::middleware;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::response::Response;
use axum::routing::get;
use etag::EntityTag;
use http::Method;
use http_utils::mediatype::MediaType;
use http_utils::mediatype::find_content_type_from_accept;
use mediatype::Name;
use strum::EnumIter;
use strum::IntoEnumIterator;
use tower_http::compression::CompressionLayer;
use tower_http::cors::Any;
use tower_http::cors::CorsLayer;

use crate::ExternalId;
use crate::config::StatusListConfig;
use crate::publish::PublishDir;

const STATUSLIST_JWT_MEDIA_TYPE: MediaType = MediaType::new(
    mediatype::names::APPLICATION,
    Name::new_unchecked("statuslist"),
    Some(Name::new_unchecked("jwt")),
);
const STATUSLIST_JWT_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/statuslist+jwt");

const STATUSLIST_CWT_MEDIA_TYPE: MediaType = MediaType::new(
    mediatype::names::APPLICATION,
    Name::new_unchecked("statuslist"),
    Some(Name::new_unchecked("cwt")),
);
const STATUSLIST_CWT_CONTENT_TYPE: HeaderValue = HeaderValue::from_static("application/statuslist+cwt");

/// The formats in which a Status List Token can be served, if no preference is given the enum variant order decides.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter)]
enum StatusListFormat {
    Jwt,
    Cwt,
}

impl StatusListFormat {
    fn media_type(self) -> MediaType<'static> {
        match self {
            Self::Jwt => STATUSLIST_JWT_MEDIA_TYPE,
            Self::Cwt => STATUSLIST_CWT_MEDIA_TYPE,
        }
    }

    fn content_type(self) -> HeaderValue {
        match self {
            Self::Jwt => STATUSLIST_JWT_CONTENT_TYPE,
            Self::Cwt => STATUSLIST_CWT_CONTENT_TYPE,
        }
    }

    fn path(self, publish_dir: &PublishDir, id: &ExternalId) -> PathBuf {
        match self {
            Self::Jwt => publish_dir.jwt_path(id),
            Self::Cwt => publish_dir.cwt_path(id),
        }
    }
}

#[derive(Debug, Clone)]
struct RouterState {
    publish_dir: Arc<PublishDir>,
    cache_control: HeaderValue,
}

#[derive(Debug, thiserror::Error)]
pub enum RouterError {
    #[error("empty path")]
    EmptyPath,

    #[error("path does not start with slash: {0}")]
    NoSlashPrefix(String),

    #[error("duplicate context path `{0}` for same publish dir: {1} vs {2}")]
    DuplicatePath(String, PublishDir, PublishDir),
}

#[derive(Debug, Clone)]
#[cfg_attr(test, derive(PartialEq))]
pub struct StatusListRouteSource<'a> {
    pub path: &'a str,
    pub publish_dir: PublishDir,
    pub ttl: Option<Duration>,
}

impl<K> StatusListConfig<K> {
    pub fn to_route_source(&self) -> StatusListRouteSource<'_> {
        StatusListRouteSource {
            path: &self.context_path,
            publish_dir: self.publish_dir.clone(),
            ttl: self.ttl,
        }
    }
}

fn check_serve_directories<'a>(
    route_sources: impl IntoIterator<Item = StatusListRouteSource<'a>>,
) -> Result<HashMap<&'a str, StatusListRouteSource<'a>>, RouterError> {
    let iter = route_sources.into_iter();
    let (size_hint_lower, _) = iter.size_hint();

    let mut result = HashMap::<_, StatusListRouteSource<'_>>::with_capacity(size_hint_lower);

    for route_source in iter {
        if route_source.path.is_empty() {
            return Err(RouterError::EmptyPath);
        }

        if !route_source.path.starts_with('/') {
            return Err(RouterError::NoSlashPrefix(route_source.path.to_string()));
        }

        if let Some(existing_dir) = result.remove(route_source.path)
            && existing_dir.publish_dir != route_source.publish_dir
        {
            return Err(RouterError::DuplicatePath(
                route_source.path.to_string(),
                existing_dir.publish_dir,
                route_source.publish_dir,
            ));
        }

        result.insert(route_source.path, route_source);
    }

    Ok(result)
}

pub fn create_serve_router<'a>(
    route_sources: impl IntoIterator<Item = StatusListRouteSource<'a>>,
) -> Result<Router, RouterError> {
    let route_sources = check_serve_directories(route_sources)?;

    let router = route_sources
        .into_iter()
        .fold(Router::new(), |router, (path, route_source)| {
            let state = RouterState {
                publish_dir: Arc::new(route_source.publish_dir),
                cache_control: cache_control_header_for_ttl(route_source.ttl),
            };

            router.nest(
                path,
                Router::new().route("/{id}", get(serve_status_list)).with_state(state),
            )
        });
    let router = add_status_list_middleware(router);

    Ok(router)
}

fn cache_control_header_for_ttl(ttl: Option<Duration>) -> HeaderValue {
    match ttl {
        None => HeaderValue::from_static("no-cache"),
        Some(ttl) => {
            // Convert to HeaderValue, unwrap is safe since string is ASCII.
            HeaderValue::from_str(&format!("max-age={}; must-revalidate", ttl.as_secs())).unwrap()
        }
    }
}

pub fn add_status_list_middleware(router: Router) -> Router {
    router
        .layer(middleware::from_fn(add_vary_header))
        // The HTTP endpoint SHOULD support the use of Cross-Origin Resource Sharing (CORS) [...] to enable
        // Browser-based clients to access it
        .layer(
            CorsLayer::new()
                .allow_methods([Method::GET, Method::HEAD, Method::OPTIONS])
                .allow_origin(Any),
        )
        // The HTTP response SHOULD use gzip Content-Encoding as defined in [RFC9110].
        .layer(CompressionLayer::new())
}

async fn add_vary_header(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    response
        .headers_mut()
        .insert(header::VARY, HeaderValue::from_static("accept"));
    response
}

async fn serve_status_list(
    headers: HeaderMap,
    Path(id): Path<ExternalId>,
    State(state): State<RouterState>,
) -> Result<Response, StatusCode> {
    let format = requested_format(headers.get(header::ACCEPT))?;

    let path = format.path(&state.publish_dir, &id);
    let bytes = tokio::fs::read(&path).await.map_err(|err| map_io_error(&path, &err))?;

    let etag = EntityTag::from_data(&bytes);
    if let Some(request_etag) = headers.get(header::IF_NONE_MATCH)
        && check_if_none_match(request_etag, &etag)?
    {
        return Err(StatusCode::NOT_MODIFIED);
    };

    let response_headers = [
        (header::CONTENT_TYPE, format.content_type()),
        (header::CACHE_CONTROL, state.cache_control),
        // Unwrap is safe since etag is ASCII
        (header::ETAG, HeaderValue::from_str(&etag.to_string()).unwrap()),
    ];
    let response = (response_headers, bytes).into_response();

    Ok(response)
}

pub fn http_response_for_status_list_body(
    request_headers: &HeaderMap,
    status_list_body: Vec<u8>,
    ttl: Option<Duration>,
) -> Result<Response, StatusCode> {
    // The status list body is only available as JWT
    if requested_format(request_headers.get(header::ACCEPT))? != StatusListFormat::Jwt {
        return Err(StatusCode::NOT_ACCEPTABLE);
    }

    let response_headers = [
        (header::CONTENT_TYPE, STATUSLIST_JWT_CONTENT_TYPE),
        (header::CACHE_CONTROL, cache_control_header_for_ttl(ttl)),
    ];
    let response = (response_headers, status_list_body).into_response();

    Ok(response)
}

fn ascii_header<'a>(header: &'a HeaderValue, name: &str) -> Result<&'a str, StatusCode> {
    header.to_str().map_err(|err| {
        tracing::info!("non-ascii header for {name}: {err}");
        StatusCode::BAD_REQUEST
    })
}

/// Determine the format to serve from the accept header. Without an accept header, or for `*/*`, JWT is served.
/// Returns `406 Not Acceptable` if no format is acceptable.
fn requested_format(header: Option<&HeaderValue>) -> Result<StatusListFormat, StatusCode> {
    match find_content_type_from_accept(
        header,
        |media_type| StatusListFormat::iter().find(|format| format.media_type() == media_type),
        StatusListFormat::Jwt,
    ) {
        Ok(Some(format)) => Ok(format),
        Ok(None) => Err(StatusCode::NOT_ACCEPTABLE),
        Err(err) => {
            tracing::info!("invalid accept header: {err}");
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

fn check_if_none_match(header: &HeaderValue, etag: &EntityTag) -> Result<bool, StatusCode> {
    let header = ascii_header(header, "if-none-match")?;
    let request_etag = header.parse::<EntityTag>().map_err(|err| {
        tracing::info!("invalid if-none-match header `{}`: {}", header, err);
        StatusCode::BAD_REQUEST
    })?;
    Ok(request_etag.weak_eq(etag))
}

fn map_io_error(path: &std::path::Path, err: &std::io::Error) -> StatusCode {
    if err.kind() == ErrorKind::NotFound {
        return StatusCode::NOT_FOUND;
    }
    tracing::warn!("could not read `{}`: {}", path.display(), err);
    StatusCode::INTERNAL_SERVER_ERROR
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use rstest::rstest;

    use super::*;

    #[test]
    fn check_serve_dir_errors_on_empty_path() {
        let dir = PublishDir::try_new(std::env::temp_dir()).unwrap();

        let result = check_serve_directories([StatusListRouteSource {
            path: "",
            publish_dir: dir,
            ttl: None,
        }]);
        assert!(result.is_err());
        assert_matches!(result.unwrap_err(), RouterError::EmptyPath);
    }

    #[test]
    fn check_serve_dir_errors_on_path_without_slahs_prefix() {
        let path = "path";
        let dir = PublishDir::try_new(std::env::temp_dir()).unwrap();

        let result = check_serve_directories([StatusListRouteSource {
            path,
            publish_dir: dir,
            ttl: None,
        }]);
        assert!(result.is_err());
        assert_matches!(result.unwrap_err(), RouterError::NoSlashPrefix(error_path) if error_path == path);
    }

    #[test]
    fn check_serve_dir_allow_duplicate_path_for_same_publish_dir() {
        let path = "/path";
        let dir = PublishDir::try_new(std::env::temp_dir()).unwrap();

        let route_source = StatusListRouteSource {
            path,
            publish_dir: dir.clone(),
            ttl: None,
        };
        let result = check_serve_directories([route_source.clone(), route_source.clone()]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [(path, route_source)].into());
    }

    #[test]
    fn check_serve_dir_err_on_duplicate_path_for_different_publish_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let path = "/path";
        let dir_a = PublishDir::try_new(std::env::temp_dir()).unwrap();
        let dir_b = PublishDir::try_new(tmp.path().to_path_buf()).unwrap();

        let route_source_a = StatusListRouteSource {
            path,
            publish_dir: dir_a.clone(),
            ttl: None,
        };
        let route_source_b = StatusListRouteSource {
            path,
            publish_dir: dir_b.clone(),
            ttl: None,
        };

        let result = check_serve_directories([route_source_a, route_source_b]);
        assert!(result.is_err());
        assert_matches!(
            result.unwrap_err(),
            RouterError::DuplicatePath(error_path, error_dir_a, error_dir_b)
                if error_path == path && error_dir_a == dir_a && error_dir_b == dir_b
        );
    }

    #[test]
    fn non_ascii_header() {
        let header = HeaderValue::from_bytes(&[0x80]).unwrap();
        assert_matches!(ascii_header(&header, "test"), Err(StatusCode::BAD_REQUEST));
    }

    #[test]
    fn content_type_matches_media_type() {
        for format in StatusListFormat::iter() {
            assert_eq!(format.content_type(), format.media_type().to_string().as_str());
        }
    }

    #[test]
    fn requested_format_invalid_type() {
        let header = HeaderValue::from_str("application/statuslist/jwt").unwrap();
        assert_matches!(requested_format(Some(&header)), Err(StatusCode::BAD_REQUEST));
    }

    #[rstest]
    #[case::no_accept(None, Ok(StatusListFormat::Jwt))]
    #[case::any(Some("*/*"), Ok(StatusListFormat::Jwt))]
    #[case::jwt(Some("application/statuslist+jwt"), Ok(StatusListFormat::Jwt))]
    #[case::cwt(Some("application/statuslist+cwt"), Ok(StatusListFormat::Cwt))]
    #[case::cwt_preferred(
        Some("application/statuslist+cwt, application/statuslist+jwt"),
        Ok(StatusListFormat::Cwt)
    )]
    #[case::jwt_preferred(
        Some("application/statuslist+jwt, application/statuslist+cwt"),
        Ok(StatusListFormat::Jwt)
    )]
    #[case::text(Some("text/plain"), Err(StatusCode::NOT_ACCEPTABLE))]
    fn requested_format_from_accept(
        #[case] accept: Option<&str>,
        #[case] expected: Result<StatusListFormat, StatusCode>,
    ) {
        let header = accept.map(|accept| HeaderValue::from_str(accept).unwrap());
        assert_eq!(requested_format(header.as_ref()), expected);
    }

    #[test]
    fn check_if_none_match_invalid_etag() {
        let header = HeaderValue::from_str("@").unwrap();
        assert_matches!(
            check_if_none_match(&header, &EntityTag::strong("test")),
            Err(StatusCode::BAD_REQUEST)
        );
    }

    #[test]
    fn map_other_io_error() {
        let result = map_io_error(
            &std::env::temp_dir(),
            &std::io::Error::new(ErrorKind::IsADirectory, "is a directory"),
        );
        assert_matches!(result, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
