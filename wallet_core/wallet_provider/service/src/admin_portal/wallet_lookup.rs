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
    #[error("missing privilege for task type {0}")]
    MissingPrivilege(TaskType),
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

/// Number of wallets associated with a recovery code, split by whether they are still active.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecoveryCodeWalletCounts {
    pub active: usize,
    pub inactive: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum GetRecoveryCodeWalletCountsError {
    #[error("missing privilege for task type {0}")]
    MissingPrivilege(TaskType),
    #[error("recovery code not found")]
    NotFound,
    #[error("persistence error: {0}")]
    Storage(#[source] PersistenceError),
    #[error("error while auditing: {0}")]
    AuditLog(#[source] Box<dyn Error + Send + Sync>),
}

impl FromAuditLogError for GetRecoveryCodeWalletCountsError {
    fn from_audit_log_error(audit_log_error: Box<dyn Error + Send + Sync>) -> Self {
        Self::AuditLog(audit_log_error)
    }
}

impl<R, T, U> AdminPortalService<R, T, U> {
    /// Counts the wallets associated with the recovery code, for `block_user` tasks. A wallet is active unless it
    /// is revoked or transferred, consistent with the check in `has_multiple_active_accounts_by_recovery_code` in the
    /// persistence layer.
    #[audited]
    pub async fn get_recovery_code_wallet_counts<TX, UR, F, H>(
        &self,
        #[audit] recovery_code: &RecoveryCode,
        #[audit_user] user: &LoggedInUser,
        user_state: &UserState<UR, F, H, impl SecureEcdsaKey, impl StatusListService>,
        #[auditor] audit_log: &impl AuditLog,
    ) -> Result<RecoveryCodeWalletCounts, GetRecoveryCodeWalletCountsError>
    where
        TX: Committable,
        UR: TransactionStarter<TransactionType = TX> + WalletUserRepository<TransactionType = TX>,
    {
        if !user.is_authorized_to(&TaskType::BlockUser) {
            return Err(GetRecoveryCodeWalletCountsError::MissingPrivilege(TaskType::BlockUser));
        }

        let transaction = user_state
            .repositories
            .begin_transaction()
            .await
            .map_err(GetRecoveryCodeWalletCountsError::Storage)?;

        let states = user_state
            .repositories
            .find_wallet_user_states_by_recovery_code(&transaction, recovery_code)
            .await
            .map_err(GetRecoveryCodeWalletCountsError::Storage)?;

        transaction
            .commit()
            .await
            .map_err(GetRecoveryCodeWalletCountsError::Storage)?;

        if states.is_empty() {
            return Err(GetRecoveryCodeWalletCountsError::NotFound);
        }

        let active = states
            .iter()
            .filter(|state| !matches!(state, WalletUserState::Revoked | WalletUserState::Transferred))
            .count();

        Ok(RecoveryCodeWalletCounts {
            active,
            inactive: states.len() - active,
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
        if !user.is_authorized_to(&TaskType::RevokeWallet) {
            return Err(GetWalletRevocationStateError::MissingPrivilege(TaskType::RevokeWallet));
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
    use rstest::fixture;
    use rstest::rstest;
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
    fn privileged_user() -> LoggedInUser {
        user("user-1", &[TaskType::RevokeWallet])
    }

    #[fixture]
    fn unprivileged_user() -> LoggedInUser {
        user("user-1", &[TaskType::BlockUser])
    }

    /// Creates the service under test; `check_wallet` does not touch `self.repository`, so an empty mock suffices.
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
    async fn check_wallet_rejects_missing_privilege(wallet_id: WalletId, unprivileged_user: LoggedInUser) {
        let wallet_user_repo = MockTransactionalWalletUserRepository::new();
        let user_state = user_state(
            wallet_user_repo,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        );

        let result = service()
            .get_wallet_revocation_state(&wallet_id, &unprivileged_user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(
            result,
            Err(GetWalletRevocationStateError::MissingPrivilege(TaskType::RevokeWallet))
        );
    }

    #[rstest]
    #[tokio::test]
    async fn check_wallet_returns_state_when_found(wallet_id: WalletId, privileged_user: LoggedInUser) {
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
                Ok(QueryResult::Found(Box::new(WalletUserState::Active)))
            });
        let user_state = user_state(
            wallet_user_repo,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        );

        let result = service()
            .get_wallet_revocation_state(&wallet_id, &privileged_user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(result, Ok(found_state) if found_state == WalletRevocationState::Revokable);
    }

    #[rstest]
    #[tokio::test]
    async fn check_wallet_returns_not_found_when_wallet_does_not_exist(
        wallet_id: WalletId,
        privileged_user: LoggedInUser,
    ) {
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_wallet_user_state_by_wallet_id()
            .times(1)
            .return_once(|_, _| Ok(QueryResult::NotFound));
        let user_state = user_state(
            wallet_user_repo,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        );

        let result = service()
            .get_wallet_revocation_state(&wallet_id, &privileged_user, &user_state, &MockAuditLog)
            .await;

        assert_matches!(result, Err(GetWalletRevocationStateError::NotFound(id)) if id == wallet_id);
    }

    #[rstest]
    #[tokio::test]
    async fn recovery_code_counts_split_active_and_inactive_wallets() {
        let recovery_code = RecoveryCode::from("recovery-code".to_string());
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_wallet_user_states_by_recovery_code()
            .times(1)
            .return_once(|_, _| {
                Ok(vec![
                    WalletUserState::Active,
                    WalletUserState::Blocked,
                    WalletUserState::Revoked,
                    WalletUserState::Transferred,
                ])
            });
        let user_state = user_state(
            wallet_user_repo,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        );

        let result = service()
            .get_recovery_code_wallet_counts(
                &recovery_code,
                &user("user-1", &[TaskType::BlockUser]),
                &user_state,
                &MockAuditLog,
            )
            .await;

        assert_matches!(result, Ok(counts) if counts == RecoveryCodeWalletCounts { active: 2, inactive: 2 });
    }

    #[rstest]
    #[tokio::test]
    async fn recovery_code_counts_report_not_found_without_wallets() {
        let recovery_code = RecoveryCode::from("recovery-code".to_string());
        let mut wallet_user_repo = MockTransactionalWalletUserRepository::new();
        wallet_user_repo
            .expect_begin_transaction()
            .returning(|| Ok(MockTransaction));
        wallet_user_repo
            .expect_find_wallet_user_states_by_recovery_code()
            .times(1)
            .return_once(|_, _| Ok(vec![]));
        let user_state = user_state(
            wallet_user_repo,
            StubWalletFlags::default(),
            setup_hsm().await,
            Kid::try_from("0").unwrap(),
            TrustAnchors::empty(),
            mock_status_list_service(),
        );

        let result = service()
            .get_recovery_code_wallet_counts(
                &recovery_code,
                &user("user-1", &[TaskType::BlockUser]),
                &user_state,
                &MockAuditLog,
            )
            .await;

        assert_matches!(result, Err(GetRecoveryCodeWalletCountsError::NotFound));
    }
}
