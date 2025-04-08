use std::sync::Arc;

use parking_lot::Mutex;
use tokio::task::JoinHandle;
use tracing::error;

use update_policy_model::update_policy::VersionState;

use crate::repository::BackgroundUpdateableRepository;
use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryCallback;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::HttpUpdatePolicyRepository;
use super::UpdatePolicyError;
use super::UpdatePolicyRepository;

type UpdateTask = JoinHandle<Result<RepositoryUpdateState<VersionState>, UpdatePolicyError>>;

pub struct BackgroundUpdateableUpdatePolicyRepository<T, B> {
    wrapped: Arc<T>,
    callback: Arc<Mutex<Option<RepositoryCallback<VersionState>>>>,
    background_task: Arc<Mutex<Option<UpdateTask>>>,
    _phantom: std::marker::PhantomData<B>,
}

impl<T, B> BackgroundUpdateableUpdatePolicyRepository<T, B>
where
    T: UpdateableRepository<VersionState, B, Error = UpdatePolicyError> + Send + Sync + 'static,
{
    fn from_arc(wrapped: Arc<T>) -> Self {
        Self {
            wrapped,
            callback: Arc::new(Mutex::new(None)),
            background_task: Arc::new(Mutex::new(None)),
            _phantom: std::marker::PhantomData,
        }
    }

    async fn fetch_and_callback(
        wrapped: &Arc<T>,
        config: &B,
        callback: &Arc<Mutex<Option<RepositoryCallback<VersionState>>>>,
    ) -> Result<RepositoryUpdateState<VersionState>, UpdatePolicyError> {
        match wrapped.fetch(config).await {
            Ok(state) => {
                if let RepositoryUpdateState::Updated { .. } = state {
                    let new_state = wrapped.get();

                    if let Some(callback) = callback.lock().as_deref_mut() {
                        callback(new_state);
                    }
                }
                Ok(state)
            }
            Err(e) => {
                error!("fetch update policy error: {}", e);
                Err(e)
            }
        }
    }
}

impl UpdatePolicyRepository {
    pub fn init() -> Self {
        let wrapped = HttpUpdatePolicyRepository::new();
        Self::from_arc(Arc::new(wrapped))
    }
}

impl<T, B> BackgroundUpdateableRepository<VersionState, B> for BackgroundUpdateableUpdatePolicyRepository<T, B>
where
    T: UpdateableRepository<VersionState, B, Error = UpdatePolicyError> + Send + Sync + 'static,
    B: Send + Sync + 'static,
{
    fn fetch_in_background(&self, config: B) {
        if self
            .background_task
            .lock()
            .as_ref()
            .is_none_or(|task| !task.is_finished())
        {
            let wrapped = Arc::clone(&self.wrapped);
            let callback = Arc::clone(&self.callback);
            let background_task =
                tokio::spawn(async move { Self::fetch_and_callback(&wrapped, &config, &callback).await });

            self.background_task.lock().replace(background_task);
        }
    }
}

impl<T, B> UpdateableRepository<VersionState, B> for BackgroundUpdateableUpdatePolicyRepository<T, B>
where
    T: UpdateableRepository<VersionState, B, Error = UpdatePolicyError> + Send + Sync + 'static,
    B: Send + Sync,
{
    type Error = UpdatePolicyError;

    async fn fetch(&self, config: &B) -> Result<RepositoryUpdateState<VersionState>, Self::Error> {
        let background_task = { self.background_task.lock().take() };
        match background_task {
            Some(task) if !task.is_finished() => task.await?,
            _ => Self::fetch_and_callback(&self.wrapped, config, &self.callback).await,
        }
    }
}

impl<T, B> Repository<VersionState> for BackgroundUpdateableUpdatePolicyRepository<T, B>
where
    T: Repository<VersionState>,
{
    fn get(&self) -> VersionState {
        self.wrapped.get()
    }
}

impl<T, B> ObservableRepository<VersionState> for BackgroundUpdateableUpdatePolicyRepository<T, B>
where
    T: Repository<VersionState>,
{
    fn register_callback_on_update(
        &self,
        callback: RepositoryCallback<VersionState>,
    ) -> Option<RepositoryCallback<VersionState>> {
        self.callback.lock().replace(callback)
    }

    fn clear_callback(&self) -> Option<RepositoryCallback<VersionState>> {
        self.callback.lock().take()
    }
}

impl<T, B> Drop for BackgroundUpdateableUpdatePolicyRepository<T, B> {
    fn drop(&mut self) {
        if let Some(t) = self.background_task.lock().as_ref() {
            t.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::time::Duration;

    use parking_lot::RwLock;
    use tokio::sync::Notify;

    use http_utils::http::client::TlsPinningConfig;
    use update_policy_model::update_policy::VersionState;

    use crate::repository::Repository;
    use crate::repository::RepositoryUpdateState;
    use crate::repository::UpdateableRepository;

    use super::*;

    struct TestRepo(RwLock<VersionState>);

    impl Repository<VersionState> for TestRepo {
        fn get(&self) -> VersionState {
            *self.0.read()
        }
    }

    impl<B> UpdateableRepository<VersionState, B> for TestRepo
    where
        B: Send + Sync,
    {
        type Error = UpdatePolicyError;

        async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<VersionState>, UpdatePolicyError> {
            let mut config = self.0.write();
            let from = *config;
            *config = VersionState::Block;

            Ok(RepositoryUpdateState::Updated { from, to: *config })
        }
    }

    #[tokio::test]
    async fn should_update_config() {
        let repository =
            BackgroundUpdateableUpdatePolicyRepository::from_arc(Arc::new(TestRepo(RwLock::new(VersionState::Ok))));
        assert_eq!(repository.get(), VersionState::Ok);

        let notifier = Arc::new(Notify::new());
        let callback_notifier = notifier.clone();

        repository.register_callback_on_update(Box::new(move |state| {
            callback_notifier.notify_one();
            assert_eq!(state, VersionState::Block);
        }));

        // No awaits, so the task will not be executed
        assert_eq!(repository.get(), VersionState::Ok);

        repository.fetch_in_background(TlsPinningConfig {
            base_url: "https://example.com".parse().unwrap(),
            trust_anchors: vec![],
        });
        notifier.notified().await;
        assert_eq!(repository.get(), VersionState::Block);
    }

    #[tokio::test]
    async fn drop_should_abort_background_task() {
        let test_repo = Arc::new(TestRepo(RwLock::new(VersionState::Ok)));
        let notifier = Arc::new(Notify::new());

        {
            let repository = BackgroundUpdateableUpdatePolicyRepository::from_arc(Arc::clone(&test_repo));
            assert_eq!(repository.get(), VersionState::Ok);

            let callback_notifier = notifier.clone();

            repository.register_callback_on_update(Box::new(move |state| {
                callback_notifier.notify_one();
                assert_eq!(state, VersionState::Block);
            }));

            // No awaits, so the task will not be executed
            assert_eq!(repository.get(), VersionState::Ok);

            repository.fetch_in_background(TlsPinningConfig {
                base_url: "https://example.com".parse().unwrap(),
                trust_anchors: vec![],
            });

            // Drop the background repository
        }

        // Enforce a possible switch to the next task
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Verify that repo didn't get updated
        assert_eq!(test_repo.get(), VersionState::Ok);
    }
}
