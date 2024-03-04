mod config;
mod disclosure;
mod documents;
mod history;
mod init;
mod issuance;
mod lock;
mod registration;
mod reset;
mod uri;

#[cfg(test)]
mod test;

use tokio::sync::RwLock;
use uuid::Uuid;

use nl_wallet_mdoc::holder::{CborHttpClient, DisclosureSession};
use openid4vc::{issuance_session::HttpIssuanceSession, oidc::HttpOidcClient};
use platform_support::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};

use crate::{
    account_provider::HttpAccountProviderClient,
    config::UpdatingFileHttpConfigurationRepository,
    lock::WalletLock,
    storage::{DatabaseStorage, RegistrationData},
};

pub use self::{
    config::ConfigCallback,
    disclosure::{DisclosureError, DisclosureProposal},
    documents::DocumentsCallback,
    history::{
        EventConversionError, EventStatus, EventStorageError, HistoryError, HistoryEvent, RecentHistoryCallback,
    },
    init::WalletInitError,
    issuance::PidIssuanceError,
    lock::{LockCallback, WalletUnlockError},
    registration::WalletRegistrationError,
    reset::ResetError,
    uri::{UriIdentificationError, UriType},
};

#[cfg(test)]
pub(crate) use self::issuance::rvig_registration;

use self::issuance::PidIssuanceSession;

struct WalletRegistration<K> {
    hw_privkey: K,
    data: RegistrationData,
}

pub struct Wallet<
    CR = UpdatingFileHttpConfigurationRepository,  // ConfigurationRepository
    S = DatabaseStorage<HardwareEncryptionKey>,    // Storage
    PEK = HardwareEcdsaKey,                        // PlatformEcdsaKey
    APC = HttpAccountProviderClient,               // AccountProviderClient
    OIC = HttpOidcClient,                          // OidcClient
    IC = HttpIssuanceSession,                      // IssuanceSession
    MDS = DisclosureSession<CborHttpClient, Uuid>, // MdocDisclosureSession
> {
    config_repository: CR,
    storage: RwLock<S>,
    account_provider_client: APC,
    issuance_session: Option<PidIssuanceSession<OIC, IC>>,
    disclosure_session: Option<MDS>,
    lock: WalletLock,
    registration: Option<WalletRegistration<PEK>>,
    documents_callback: Option<DocumentsCallback>,
    recent_history_callback: Option<RecentHistoryCallback>,
}
