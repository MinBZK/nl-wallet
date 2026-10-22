use std::collections::HashSet;

use audit_log::audited;
use audit_log::model::AuditLog;
use chrono::DateTime;
use chrono::Utc;
use crypto::keys::SecureEcdsaKey;
use token_status_list::status_list_service::StatusListService;
use utils::generator::Generator;
use uuid::Uuid;
use wallet_provider_domain::model::wallet_user::WalletId;
use wallet_provider_domain::repository::Committable;
use wallet_provider_domain::repository::TransactionStarter;
use wallet_provider_domain::repository::WalletUserRepository;

use crate::account_server::UserState;
use crate::admin_portal::LoggedInUser;
use crate::revocation::RevocationError;
use crate::revocation::system_revoke_wallets_by_wallet_id;

/// Revokes the wallets of an approved admin task. Like the existing revoke wallets endpoint, this fails without
/// revoking anything if any of the wallet IDs do not exist. Returns the number of wallets revoked by this call,
/// excluding those that were already revoked.
#[audited]
pub async fn revoke_wallets<T, R, F, H>(
    #[audit] task_id: Uuid,
    #[audit] wallet_ids: &HashSet<WalletId>,
    #[audit_user] approved_by: &LoggedInUser,
    user_state: &UserState<R, F, H, impl SecureEcdsaKey, impl StatusListService>,
    time: &impl Generator<DateTime<Utc>>,
    #[auditor] audit_log: &impl AuditLog,
) -> Result<u64, RevocationError>
where
    T: Committable,
    R: TransactionStarter<TransactionType = T> + WalletUserRepository<TransactionType = T>,
{
    system_revoke_wallets_by_wallet_id(wallet_ids, user_state, time).await
}
