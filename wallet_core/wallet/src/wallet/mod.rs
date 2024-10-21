mod change_pin;
mod config;
mod disclosure;
mod documents;
mod history;
mod init;
mod instruction_client;
mod issuance;
mod lock;
mod registration;
mod reset;
mod uri;

#[cfg(test)]
mod test;

use tokio::sync::RwLock;
use uuid::Uuid;

use openid4vc::{
    disclosure_session::{DisclosureSession, HttpVpMessageClient},
    issuance_session::HttpIssuanceSession,
};
use platform_support::hw_keystore::hardware::{HardwareEcdsaKey, HardwareEncryptionKey};

use crate::{
    account_provider::HttpAccountProviderClient,
    config::UpdatingFileHttpConfigurationRepository,
    issuance::HttpDigidSession,
    lock::WalletLock,
    storage::{DatabaseStorage, RegistrationData},
    wte::WpWteIssuanceClient,
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
    lock::{LockCallback, UnlockMethod, WalletUnlockError},
    registration::WalletRegistrationError,
    reset::ResetError,
    uri::{UriIdentificationError, UriType},
};

use self::issuance::PidIssuanceSession;

struct WalletRegistration<K> {
    hw_privkey: K,
    data: RegistrationData,
}

pub struct Wallet<
    CR = UpdatingFileHttpConfigurationRepository, // ConfigurationRepository
    S = DatabaseStorage<HardwareEncryptionKey>,   // Storage
    PEK = HardwareEcdsaKey,                       // PlatformEcdsaKey
    APC = HttpAccountProviderClient,              // AccountProviderClient
    DS = HttpDigidSession,                        // DigidSession
    IC = HttpIssuanceSession,                     // IssuanceSession
    MDS = DisclosureSession<HttpVpMessageClient, Uuid>, // MdocDisclosureSession
    WIC = WpWteIssuanceClient,                    // WteIssuanceClient
> {
    config_repository: CR,
    storage: RwLock<S>,
    account_provider_client: APC,
    issuance_session: Option<PidIssuanceSession<DS, IC>>,
    disclosure_session: Option<MDS>,
    lock: WalletLock,
    registration: Option<WalletRegistration<PEK>>,
    documents_callback: Option<DocumentsCallback>,
    recent_history_callback: Option<RecentHistoryCallback>,
    wte_issuance_client: WIC,
}
