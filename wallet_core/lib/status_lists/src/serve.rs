use std::collections::HashMap;
use std::io::ErrorKind;
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
use itertools::Itertools;
use mediatype::MediaType;
use mediatype::MediaTypeList;
use mediatype::Name;
use tower_http::compression::CompressionLayer;

use crate::config::StatusListConfig;
use crate::publish::PublishDir;

const ALL_MEDIA_TYPE: MediaType = MediaType::new(Name::new_unchecked("*"), Name::new_unchecked("*"));
const STATUSLIST_JWT_MEDIA_TYPE: MediaType = MediaType::from_parts(
    Name::new_unchecked("application"),
    Name::new_unchecked("statuslist"),
    Some(Name::new_unchecked("jwt")),
    &[],
);

#[derive(Debug, Clone)]
struct RouterState {
    publish_dir: Arc<PublishDir>,
    cache_control: HeaderValue,
    content_type: HeaderValue,
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
    // Convert to HeaderValue, unwrap is safe since string is ASCII.
    let content_type = HeaderValue::from_str(&STATUSLIST_JWT_MEDIA_TYPE.to_string()).unwrap();

    let route_sources = check_serve_directories(route_sources)?;
    let route_count = route_sources.len();

    let router = route_sources
        .into_iter()
        .zip_eq(itertools::repeat_n(content_type, route_count))
        .fold(Router::new(), |router, ((path, route_source), content_type)| {
            let state = RouterState {
                publish_dir: Arc::new(route_source.publish_dir),
                cache_control: cache_control_header_for_ttl(route_source.ttl),
                content_type,
            };

            router.nest(
                path,
                Router::new().route("/{id}", get(serve_status_list)).with_state(state),
            )
        })
        .layer(middleware::from_fn(add_vary_header))
        // The HTTP response SHOULD use gzip Content-Encoding as defined in [RFC9110].
        .layer(CompressionLayer::new());

    Ok(router)
}

fn cache_control_header_for_ttl(ttl: Option<Duration>) -> HeaderValue {
    let cache_control = match ttl {
        None => "no-cache",
        Some(ttl) => &format!("max-age={}; must-revalidate", ttl.as_secs()),
    };

    // Convert to HeaderValue, unwrap is safe since string is ASCII.
    HeaderValue::from_str(cache_control).unwrap()
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
    Path(id): Path<String>,
    State(state): State<RouterState>,
) -> Result<Response, StatusCode> {
    if let Some(accept) = headers.get(header::ACCEPT) {
        check_accept(accept)?
    }

    let path = state.publish_dir.jwt_path(id.as_str());
    let bytes = tokio::fs::read(&path).await.map_err(|err| map_io_error(&path, &err))?;

    let etag = EntityTag::from_data(&bytes);
    if let Some(request_etag) = headers.get(header::IF_NONE_MATCH)
        && check_if_none_match(request_etag, &etag)?
    {
        return Err(StatusCode::NOT_MODIFIED);
    };

    Ok((
        [
            (header::CONTENT_TYPE, state.content_type),
            (header::CACHE_CONTROL, state.cache_control),
            // Unwrap is safe since etag is ASCII
            (header::ETAG, HeaderValue::from_str(&etag.to_string()).unwrap()),
        ],
        bytes,
    )
        .into_response())
}

fn ascii_header<'a>(header: &'a HeaderValue, name: &str) -> Result<&'a str, StatusCode> {
    header.to_str().map_err(|err| {
        tracing::info!("non-ascii header for {}: {}", name, err);
        StatusCode::BAD_REQUEST
    })
}

/// Check accept header for valid content types
///
/// The spec says that a verifier SHOULD send a request with an Accept header
/// unless the Content-Type is known in the ecosystem or the verifier supports
/// both. For the moment only the JWT format is supported in this code base.
fn check_accept(header: &HeaderValue) -> Result<(), StatusCode> {
    let header = ascii_header(header, "accept")?;
    let content_types = MediaTypeList::new(header)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| {
            tracing::info!("invalid accept header `{}`: {}", header, err);
            StatusCode::BAD_REQUEST
        })?;
    if content_types
        .into_iter()
        // */* is pure for convenience of using curl and friends to test
        .any(|media_type| media_type == ALL_MEDIA_TYPE || media_type == STATUSLIST_JWT_MEDIA_TYPE)
    {
        return Ok(());
    }
    Err(StatusCode::UNSUPPORTED_MEDIA_TYPE)
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

    use super::*;
    use crate::serve::check_accept;

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
    fn check_accept_invalid_type() {
        let header = HeaderValue::from_str("application/statuslist/jwt").unwrap();
        assert_matches!(check_accept(&header), Err(StatusCode::BAD_REQUEST));
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
