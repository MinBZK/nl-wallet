use std::collections::HashMap;
use std::io::ErrorKind;
use std::string::ToString;
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
use mediatype::MediaType;
use mediatype::MediaTypeList;
use mediatype::Name;
use tower_http::compression::CompressionLayer;

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
    publish_dir: PublishDir,
    cache_control: String,
}

#[derive(Debug, thiserror::Error)]
#[cfg_attr(test, derive(PartialEq))]
pub enum RouterError {
    #[error("empty path")]
    EmptyPath,
    #[error("duplicate context path `{0}` for same publish dir: {1} vs {2}")]
    DuplicatePath(String, PublishDir, PublishDir),
}

fn check_serve_directories<'a>(
    serve_dirs: impl IntoIterator<Item = (&'a str, PublishDir)>,
) -> Result<HashMap<&'a str, PublishDir>, RouterError> {
    let iter = serve_dirs.into_iter();
    let mut result = HashMap::with_capacity(iter.size_hint().0);
    for (path, publish_dir) in iter {
        if path.is_empty() {
            return Err(RouterError::EmptyPath);
        }
        if let Some(inserted_dir) = result.remove(&path)
            && inserted_dir != publish_dir
        {
            return Err(RouterError::DuplicatePath(path.to_string(), inserted_dir, publish_dir));
        }
        result.insert(path, publish_dir);
    }
    Ok(result)
}

pub fn create_serve_router<'a>(
    serve_dirs: impl IntoIterator<Item = (&'a str, PublishDir)>,
    ttl: Option<Duration>,
) -> Result<Router, RouterError> {
    let cache_control = match ttl {
        None => "no-cache".to_string(),
        Some(ttl) => format!("max-age={}; must-revalidate", ttl.as_secs()),
    };

    let serve_dirs = check_serve_directories(serve_dirs)?;
    Ok(serve_dirs
        .into_iter()
        .fold(Router::new(), |router, (path, publish_dir)| {
            let state = RouterState {
                publish_dir,
                cache_control: cache_control.clone(),
            };
            router.nest(
                path,
                Router::new().route("/{id}", get(serve_status_list)).with_state(state),
            )
        })
        .layer(middleware::from_fn(add_vary_header))
        // The HTTP response SHOULD use gzip Content-Encoding as defined in [RFC9110].
        .layer(CompressionLayer::new()))
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

    let response = (
        [
            (header::CONTENT_TYPE, STATUSLIST_JWT_MEDIA_TYPE.to_string()),
            (header::CACHE_CONTROL, state.cache_control),
            (header::ETAG, etag.to_string()),
        ],
        bytes,
    )
        .into_response();
    Ok(response)
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
    use assert_matches::assert_matches;

    use crate::serve::check_accept;

    use super::*;

    #[test]
    fn check_serve_dir_errors_on_empty_path() {
        let dir = PublishDir::try_new(std::env::temp_dir()).unwrap();

        let result = check_serve_directories([("", dir.clone())]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), RouterError::EmptyPath);
    }

    #[test]
    fn check_serve_dir_allow_duplicate_path_for_same_publish_dir() {
        let path = "path";
        let dir = PublishDir::try_new(std::env::temp_dir()).unwrap();

        let result = check_serve_directories([(path, dir.clone()), (path, dir.clone())]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), [(path, dir)].into());
    }

    #[test]
    fn check_serve_dir_err_on_duplicate_path_for_different_publish_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let path = "path";
        let dir_a = PublishDir::try_new(std::env::temp_dir()).unwrap();
        let dir_b = PublishDir::try_new(tmp.path().to_path_buf()).unwrap();

        let result = check_serve_directories([(path, dir_a.clone()), (path, dir_b.clone())]);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            RouterError::DuplicatePath(path.to_string(), dir_a, dir_b)
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
