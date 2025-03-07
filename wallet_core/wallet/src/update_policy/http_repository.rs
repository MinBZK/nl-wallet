use std::str::FromStr;
use std::sync::LazyLock;
use std::time::Duration;
use std::time::Instant;

use parking_lot::RwLock;
use semver::Version;
use serde::de::DeserializeOwned;
use tracing::info;

use configuration::http::TlsPinningConfig;
use configuration::http::TlsPinningConfigHash;
use wallet_common::update_policy::UpdatePolicyResponse;
use wallet_common::update_policy::VersionState;

use crate::repository::HttpClient;
use crate::repository::HttpResponse;
use crate::repository::Repository;
use crate::repository::RepositoryUpdateState;
use crate::repository::ReqwestHttpClient;
use crate::repository::UpdateableRepository;
use crate::update_policy::UpdatePolicyError;

pub struct HttpUpdatePolicyRepository {
    client: ReqwestHttpClient<Json<UpdatePolicyResponse>, TlsPinningConfig>,
    state: RwLock<(VersionState, Option<(Instant, TlsPinningConfigHash)>)>,
}

static CURRENT_VERSION: LazyLock<Version> =
    LazyLock::new(|| Version::parse(env!("CARGO_PKG_VERSION")).expect("should always be a valid semver"));
#[allow(clippy::identity_op)]
static CACHE_DURATION: LazyLock<Duration> = LazyLock::new(|| Duration::from_secs(1 * 60 * 60)); // 1 hour

impl HttpUpdatePolicyRepository {
    #[allow(clippy::new_without_default)] // this will receive some parameters in the future
    pub fn new() -> Self {
        LazyLock::force(&CURRENT_VERSION); // force a failure as early as possible

        Self {
            client: ReqwestHttpClient::new("update-policy".parse().expect("should be a valid filename")),
            state: RwLock::new((VersionState::Ok, None)),
        }
    }
}

impl Repository<VersionState> for HttpUpdatePolicyRepository {
    fn get(&self) -> VersionState {
        self.state.read().0
    }
}

struct Json<T>(T);

impl<T> FromStr for Json<T>
where
    T: DeserializeOwned,
{
    type Err = serde_json::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(serde_json::from_str(s)?))
    }
}

impl UpdateableRepository<VersionState, TlsPinningConfig> for HttpUpdatePolicyRepository {
    type Error = UpdatePolicyError;

    async fn fetch(&self, config: &TlsPinningConfig) -> Result<RepositoryUpdateState<VersionState>, Self::Error> {
        let now = Instant::now();

        {
            let (current_state, last_fetched) = *self.state.read();
            if last_fetched.is_some_and(|(last_fetch, fetched_for)| {
                now.checked_duration_since(last_fetch)
                    .is_some_and(|diff| diff < *CACHE_DURATION)
                    && fetched_for == config.to_hash()
            }) {
                info!("Using cached version state for version {}", *CURRENT_VERSION);
                return Ok(RepositoryUpdateState::Cached(current_state));
            }
        }

        let body = self.client.fetch(config).await?;
        let new_state = match body {
            HttpResponse::Parsed(Json(policy)) => policy.into_version_state(&CURRENT_VERSION),
            HttpResponse::NotModified => {
                info!("Update policy has not changed");
                return Ok(RepositoryUpdateState::Unmodified(self.get()));
            }
        };

        let mut lock = self.state.write();
        lock.1 = Some((now, config.to_hash()));

        if new_state == lock.0 {
            info!("Received new update policy, nothing changed");

            return Ok(RepositoryUpdateState::Unmodified(lock.0));
        }

        info!(
            "Received new update policy, updating the state for version {} to {}",
            *CURRENT_VERSION, new_state
        );

        let from = lock.0;
        lock.0 = new_state;

        Ok(RepositoryUpdateState::Updated { from, to: lock.0 })
    }
}
