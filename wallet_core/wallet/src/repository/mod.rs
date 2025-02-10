mod etag_client;
mod reqwest_client;

use std::error::Error;
use std::path::PathBuf;

use error_category::ErrorCategory;
pub use etag_client::EtagHttpClient;
use nutype::nutype;
pub use reqwest_client::ReqwestHttpClient;

#[derive(Debug)]
pub enum RepositoryUpdateState<T> {
    Updated { from: T, to: T },
    Unmodified(T),
    Cached(T),
}

impl<T> RepositoryUpdateState<T> {
    pub fn get(&self) -> &T {
        match self {
            RepositoryUpdateState::Updated { to, .. } => to,
            RepositoryUpdateState::Unmodified(val) => val,
            RepositoryUpdateState::Cached(val) => val,
        }
    }
}

pub trait Repository<T> {
    fn get(&self) -> T;
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(defer)]
pub enum HttpClientError {
    #[error("networking error: {0}")]
    #[category(critical)]
    Networking(#[from] reqwest::Error),
    #[error("could not get config from config server: {0} - Response body: {1}")]
    #[category(critical)]
    Response(#[source] reqwest::Error, String),
    #[error("could not store or load configuration: {0}")]
    EtagFile(#[from] FileStorageError),
    #[category(critical)]
    #[error("could not parse body: {0}")]
    Parse(#[source] Box<dyn Error + Send + Sync>),
    #[category(critical)]
    #[error("empty body")]
    EmptyBody,
}

#[derive(Debug, thiserror::Error, ErrorCategory)]
#[category(pd)]
pub enum FileStorageError {
    #[error("config file I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

#[nutype(
    validate(predicate = |s| sanitize_filename::is_sanitized(s.to_string_lossy())),
    derive(Debug, Clone, AsRef, FromStr),
)]
struct Filename(PathBuf);

pub enum HttpResponse<T> {
    Parsed(T),
    NotModified,
}

#[trait_variant::make(Send)]
pub trait HttpClient<T, B> {
    type Error: Error + Send + Sync + 'static;

    async fn fetch(&self, client_builder: &B) -> Result<HttpResponse<T>, Self::Error>;
}

#[trait_variant::make(Send)]
pub trait UpdateableRepository<T, B>: Repository<T> {
    type Error: Error + Send + Sync + 'static;

    async fn fetch(&self, client_builder: &B) -> Result<RepositoryUpdateState<T>, Self::Error>;
}

pub type RepositoryCallback<T> = Box<dyn FnMut(T) + Send + Sync>;

pub trait ObservableRepository<T>: Repository<T> {
    fn register_callback_on_update(&self, callback: RepositoryCallback<T>) -> Option<RepositoryCallback<T>>;

    fn clear_callback(&self) -> Option<RepositoryCallback<T>>;
}

pub trait BackgroundUpdateableRepository<T, B>: Repository<T> {
    fn fetch_in_background(&self, client_builder: B);
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::Filename;

    #[rstest]
    #[case(r#"valid"#, true)]
    #[case(r#"val-id"#, true)]
    #[case(r#"val.id"#, true)]
    #[case(r#"val id"#, true)]
    #[case(r#"VALID"#, true)]
    #[case(
        "invalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinva\
         lidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidinvalidi\
         nvalidinvalidinvalidinvalidinvalidinvalid",
        false
    )]
    #[case(r#"in/valid"#, false)]
    #[case(
        r#"in
        valid"#,
        false
    )]
    fn parse_filename(#[case] input: &str, #[case] is_ok: bool) {
        assert_eq!(input.parse::<Filename>().is_ok(), is_ok);
    }
}
