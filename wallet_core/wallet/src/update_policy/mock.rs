use derive_more::Constructor;
use wallet_common::update_policy::VersionState;

use crate::repository::BackgroundUpdateableRepository;
use crate::repository::ObservableRepository;
use crate::repository::Repository;
use crate::repository::RepositoryUpdateState;
use crate::repository::UpdateableRepository;

use super::UpdatePolicyError;

#[derive(Default, Constructor)]
pub struct MockUpdatePolicyRepository {
    state: VersionState,
}

impl Repository<VersionState> for MockUpdatePolicyRepository {
    fn get(&self) -> VersionState {
        self.state
    }
}

impl ObservableRepository<VersionState> for MockUpdatePolicyRepository {
    fn register_callback_on_update(
        &self,
        _callback: crate::repository::RepositoryCallback<VersionState>,
    ) -> Option<crate::repository::RepositoryCallback<VersionState>> {
        None
    }

    fn clear_callback(&self) -> Option<crate::repository::RepositoryCallback<VersionState>> {
        None
    }
}

impl<B> UpdateableRepository<VersionState, B> for MockUpdatePolicyRepository
where
    B: Send + Sync,
{
    type Error = UpdatePolicyError;

    async fn fetch(&self, _: &B) -> Result<RepositoryUpdateState<VersionState>, Self::Error> {
        Ok(RepositoryUpdateState::Unmodified(self.state))
    }
}

impl<B> BackgroundUpdateableRepository<VersionState, B> for MockUpdatePolicyRepository {
    fn fetch_in_background(&self, _: B) {
        let state = self.state;
        tokio::spawn(async move {
            Result::<RepositoryUpdateState<VersionState>, UpdatePolicyError>::Ok(RepositoryUpdateState::Unmodified(
                state,
            ))
        });
    }
}
