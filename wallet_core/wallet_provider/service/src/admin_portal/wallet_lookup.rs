use std::error::Error;

use audit_log::audited;
use audit_log::model::AuditLog;
use audit_log::model::FromAuditLogError;
use crypto::keys::SecureEcdsaKey;
use serde::Deserialize;
use serde::Serialize;
use token_status_list::status_list_service::StatusListService;
use wallet_provider_domain::model::QueryResult;
use wallet_provider_domain::model::admin_task::TaskType;
use wallet_provider_domain::model::wallet_user::RecoveryCode;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::model::wallet_user::WalletUserState;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::PersistenceError;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use super::AdminPortalService;
use super::LoggedInUser;
use crate::account_server::UserState;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum WalletRevocationState {
    Revoked,
    Revokable,
}

impl From<WalletUserState> for WalletRevocationState {
    fn from(source: WalletUserState) -> Self {
        // All wallets that have not yet been revoked can be revoked, see
        // [`wallet_provider_persistence::wallet_user::revoke_wallets`]
        if source == WalletUserState::Revoked {
            Self::Revoked
        } else {
            Self::Revokable
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetWalletRevocationStateError {
    #[error("missing privilege for any of task types {0:?}")]
    MissingPrivilege(Vec<TaskType>),
    #[error("wallet not found: {0}")]
    NotFound(WalletId),
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for GetWalletRevocationStateError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

/// Number of wallets associated with a recovery code, split by whether they are still active, and whether the
/// recovery code is denied.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryCodeSummary {
    pub active_wallets: usize,
    pub inactive_wallets: usize,
    pub is_denied: bool,
}

impl RecoveryCodeSummary {
    pub fn wallet_count(&self) -> usize {
        self.active_wallets + self.inactive_wallets
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetRecoveryCodeSummaryError {
    #[error("missing privilege for any of task types {0:?}")]
    MissingPrivilege(Vec<TaskType>),
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for GetRecoveryCodeSummaryError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

impl<R, T, U> AdminPortalService<R, T, U> {
    /// Summarizes a recovery code for `block_user` tasks: the number of active and inactive wallets associated with
    /// it, and whether it is denied.
    ///
    /// A wallet is active unless it is revoked or transferred, consistent with the check in
    /// `has_multiple_active_accounts_by_recovery_code` in the persistence layer. A recovery code without wallets and
    /// without a denial record yields zero counts and `is_denied: false`; it is not an error.
    ///
    /// Requires the `BlockUser` or `UnblockUser` task privilege, otherwise returns `MissingPrivilege`. The wallet
    /// states and the denial flag are read in one transaction. Returns `Storage` when a persistence call fails. The
    /// call is audited as `get_recovery_code_summary`.
    #[audited]
    pub async fn get_recovery_code_summary<TX, UR, F, H>(
        &self,
        #[audit] recovery_code: &RecoveryCode,
        #[audit_user] user: &LoggedInUser,
        user_state: &UserState<UR, F, H, impl SecureEcdsaKey, impl StatusListService>,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<RecoveryCodeSummary, GetRecoveryCodeSummaryError>
    where
        TX: Committable,
        UR: TransactionStarter<TransactionType = TX> + WalletUserRepository<TransactionType = TX>,
    {
        let required = [TaskType::BlockUser, TaskType::UnblockUser];
        if !user.is_authorized_to(required) {
            return Err(GetRecoveryCodeSummaryError::MissingPrivilege(required.into()));
        }

        let transaction = user_state
            .repositories
            .begin_transaction()
            .await
            .map_err(GetRecoveryCodeSummaryError::Storage)?;

        let wallet_states = user_state
            .repositories
            .find_wallet_user_states_by_recovery_code(&transaction, recovery_code)
            .await
            .map_err(GetRecoveryCodeSummaryError::Storage)?;

        let is_denied = user_state
            .repositories
            .find_recovery_code_is_denied(&transaction, recovery_code)
            .await
            .map_err(GetRecoveryCodeSummaryError::Storage)?
            .unwrap_or(false);

        transaction
            .commit()
            .await
            .map_err(GetRecoveryCodeSummaryError::Storage)?;

        let active_wallets = wallet_states
            .iter()
            .filter(|state| !matches!(state, WalletUserState::Revoked | WalletUserState::Transferred))
            .count();

        Ok(RecoveryCodeSummary {
            active_wallets,
            inactive_wallets: wallet_states.len() - active_wallets,
            is_denied,
        })
    }

    #[audited]
    pub async fn get_wallet_revocation_state<TX, UR, F, H>(
        &self,
        #[audit] wallet_id: &WalletId,
        #[audit_user] user: &LoggedInUser,
        user_state: &UserState<UR, F, H, impl SecureEcdsaKey, impl StatusListService>,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<WalletRevocationState, GetWalletRevocationStateError>
    where
        TX: Committable,
        UR: TransactionStarter<TransactionType = TX> + WalletUserRepository<TransactionType = TX>,
    {
        // Only authorized users with `privilege_revoke_wallet` may check wallet IDs.
        let required = [TaskType::RevokeWallet];
        if !user.is_authorized_to(required) {
            return Err(GetWalletRevocationStateError::MissingPrivilege(required.into()));
        }

        let transaction = user_state
            .repositories
            .begin_transaction()
            .await
            .map_err(GetWalletRevocationStateError::Storage)?;

        let result = user_state
            .repositories
            .find_wallet_user_state_by_wallet_id(&transaction, wallet_id)
            .await
            .map_err(GetWalletRevocationStateError::Storage)?;

        transaction
            .commit()
            .await
            .map_err(GetWalletRevocationStateError::Storage)?;

        match result {
            QueryResult::Found(state) => Ok((*state).into()),
            QueryResult::NotFound => Err(GetWalletRevocationStateError::NotFound(wallet_id.clone())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::assert_matches;

    use audit_log::model::mock::MockAuditLog;
    use crypto::kid::Kid;
    use crypto::trust_anchor::TrustAnchors;
    use hsm::model::mock::MockPkcs11Client;
    use hsm::service::HsmError;
    use p256::ecdsa::SigningKey;
    use rstest::fixture;
    use rstest::rstest;
    use token_status_list::status_list_service::mock::MockStatusListService;
    use wallet_provider_domain::model::admin_task::TaskType;
    use wallet_provider_domain::model::wallet_user::WalletUserState;
    use wallet_provider_domain::repository::MockTransaction;
    use wallet_provider_persistence::repositories::mock::MockAdminPortalRepository;
    use wallet_provider_persistence::repositories::mock::MockTransactionalWalletUserRepository;

    use super::super::test_support::user;
    use super::*;
    use crate::account_server::mock::mock_status_list_service;
    use crate::account_server::mock::user_state;
    use crate::flags::mock::StubWalletFlags;
    use crate::wallet_certificate::mock::setup_hsm;

    #[fixture]
    fn wallet_id() -> WalletId {
        WalletId::from("wallet-1".to_string())
    }

    #[fixture]
    fn recovery_code() -> RecoveryCode {
        RecoveryCode::from("recovery-code".to_string())
    }

    #[fixture]
    fn revoke_wallet_user() -> LoggedInUser {
        user("user-1", &[TaskType::RevokeWallet])
    }

    #[fixture]
    fn block_user() -> LoggedInUser {
        user("user-1", &[TaskType::BlockUser])
    }

    type TestUserState = UserState<
        MockTransactionalWalletUserRepository,
        StubWalletFlags,
        MockPkcs11Client<HsmError>,
        SigningKey,
        MockStatusListService,
    >;

    async fn lookup_user_state(repository: MockTransactionalWalletUserRepository) -> TestUserState {
        user_state(
            repository,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        )
    }

    /// The lookup methods do not touch `self.repository`, so an empty mock suffices.
    fn service() -> AdminPortalService<MockAdminPortalRepository, (), ()> {
        AdminPortalService::new(
            super::super::AdminPortalConfig {
                keycloak_url: "https://keycloak.invalid/".parse().unwrap(),
                keycloak_realm: "test".to_string(),
                keycloak_client_id: "test".to_string(),
                public_url: "https://portal.example.org/".parse().unwrap(),
                session_ttl: std::time::Duration::from_mins(15),
                login_attempt_ttl: std::time::Duration::from_mins(10),
            },
            reqwest::Client::new(),
            MockAdminPortalRepository::new(),
            (),
            (),
        )
    }

    #[rstest]
    #[tokio::test]
    async fn check_wallet_rejects_missing_privilege(wallet_id: WalletId, block_user: LoggedInUser) {
        let user_state = lookup_user_state(MockTransactionalWalletUserRepository::new()).await;

        let result = service()
            .get_wallet_revocation_state(&wallet_id, &block_user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(
            result,
            Err(GetWalletRevocationStateError::MissingPrivilege(task_types)) if task_types == [TaskType::RevokeWallet]
        );
    }

    #[rstest]
    #[case::active(Some(WalletUserState::Active), Some(WalletRevocationState::Revokable))]
    #[case::revoked(Some(WalletUserState::Revoked), Some(WalletRevocationState::Revoked))]
    #[case::not_found(None, None)]
    #[tokio::test]
    async fn check_wallet_returns_lookup_result(
        wallet_id: WalletId,
        revoke_wallet_user: LoggedInUser,
        #[case] state: Option<WalletUserState>,
        #[case] expected: Option<WalletRevocationState>,
    ) {
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        let returned_wallet_id = wallet_id.clone();
        wallet_user_repo
            .expect_find_wallet_user_state_by_wallet_id()
            .times(1)
            .return_once(move |_, id| {
                assert_eq!(*id, returned_wallet_id);
                Ok(match state {
                    Some(state) => QueryResult::Found(Box::new(state)),
                    None => QueryResult::NotFound,
                })
            });
        let user_state = lookup_user_state(wallet_user_repo).await;

        let result = service()
            .get_wallet_revocation_state(&wallet_id, &revoke_wallet_user, &user_state, &MockAuditLog)
            .await;

        match expected {
            Some(expected) => assert_matches!(result, Ok(found_state) if found_state == expected),
            None => assert_matches!(result, Err(GetWalletRevocationStateError::NotFound(id)) if id == wallet_id),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn recovery_code_summary_reject_missing_privilege(
        recovery_code: RecoveryCode,
        revoke_wallet_user: LoggedInUser,
    ) {
        let user_state = lookup_user_state(MockTransactionalWalletUserRepository::new()).await;

        let result = service()
            .get_recovery_code_summary(&recovery_code, &revoke_wallet_user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(
            result,
            Err(GetRecoveryCodeSummaryError::MissingPrivilege(task_types))
                if task_types == [TaskType::BlockUser, TaskType::UnblockUser]
        );
    }

    #[rstest]
    #[case::counts(
        vec![WalletUserState::Active, WalletUserState::Blocked, WalletUserState::Revoked, WalletUserState::Transferred],
        None,
        RecoveryCodeSummary { active_wallets: 2, inactive_wallets: 2, is_denied: false }
    )]
    #[case::unknown(vec![], None, RecoveryCodeSummary { active_wallets: 0, inactive_wallets: 0, is_denied: false })]
    #[case::denied(
        vec![WalletUserState::Revoked],
        Some(true),
        RecoveryCodeSummary { active_wallets: 0, inactive_wallets: 1, is_denied: true }
    )]
    #[tokio::test]
    async fn recovery_code_summary_return_lookup_result(
        recovery_code: RecoveryCode,
        #[values(TaskType::BlockUser, TaskType::UnblockUser)] task_type: TaskType,
        #[case] states: Vec<WalletUserState>,
        #[case] is_denied: Option<bool>,
        #[case] expected: RecoveryCodeSummary,
    ) {
        let block_user = user("user-1", &[task_type]);
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_wallet_user_states_by_recovery_code()
            .times(1)
            .return_once(move |_, _| Ok(states));
        wallet_user_repo
            .expect_find_recovery_code_is_denied()
            .times(1)
            .return_once(move |_, _| Ok(is_denied));
        let user_state = lookup_user_state(wallet_user_repo).await;

        let result = service()
            .get_recovery_code_summary(&recovery_code, &block_user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(result, Ok(summary) if summary == expected);
    }
}
