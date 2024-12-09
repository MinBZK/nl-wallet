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

use openid4vc::disclosure_session::DisclosureSession;
use openid4vc::disclosure_session::HttpVpMessageClient;
use openid4vc::issuance_session::HttpIssuanceSession;
use platform_support::attested_key::hardware::HardwareAttestedKeyHolder;
use platform_support::attested_key::AttestedKey;
use platform_support::attested_key::AttestedKeyHolder;
use platform_support::hw_keystore::hardware::HardwareEncryptionKey;

use crate::account_provider::HttpAccountProviderClient;
use crate::config::UpdatingFileHttpConfigurationRepository;
use crate::issuance::HttpDigidSession;
use crate::lock::WalletLock;
use crate::storage::DatabaseStorage;
use crate::storage::RegistrationData;
use crate::wte::WpWteIssuanceClient;

pub use self::config::ConfigCallback;
pub use self::disclosure::DisclosureError;
pub use self::disclosure::DisclosureProposal;
pub use self::documents::DocumentsCallback;
pub use self::history::EventConversionError;
pub use self::history::EventStatus;
pub use self::history::EventStorageError;
pub use self::history::HistoryError;
pub use self::history::HistoryEvent;
pub use self::history::RecentHistoryCallback;
pub use self::init::WalletInitError;
pub use self::issuance::PidIssuanceError;
pub use self::lock::LockCallback;
pub use self::lock::UnlockMethod;
pub use self::lock::WalletUnlockError;
pub use self::registration::WalletRegistrationError;
pub use self::reset::ResetError;
pub use self::uri::UriIdentificationError;
pub use self::uri::UriType;

use self::issuance::PidIssuanceSession;

struct WalletRegistration<A, G> {
    attested_key: AttestedKey<A, G>,
    data: RegistrationData,
}

pub struct Wallet<
    CR = UpdatingFileHttpConfigurationRepository, // ConfigurationRepository
    S = DatabaseStorage<HardwareEncryptionKey>,   // Storage
    AKH = HardwareAttestedKeyHolder,              // AttestedKeyHolder
    APC = HttpAccountProviderClient,              // AccountProviderClient
    DS = HttpDigidSession,                        // DigidSession
    IS = HttpIssuanceSession,                     // IssuanceSession
    MDS = DisclosureSession<HttpVpMessageClient, Uuid>, // MdocDisclosureSession
    WIC = WpWteIssuanceClient,                    // WteIssuanceClient
> where
    AKH: AttestedKeyHolder,
{
    config_repository: CR,
    storage: RwLock<S>,
    key_holder: AKH,
    registration:
        Option<WalletRegistration<<AKH as AttestedKeyHolder>::AppleKey, <AKH as AttestedKeyHolder>::GoogleKey>>,
    account_provider_client: APC,
    issuance_session: Option<PidIssuanceSession<DS, IS>>,
    disclosure_session: Option<MDS>,
    wte_issuance_client: WIC,
    lock: WalletLock,
    documents_callback: Option<DocumentsCallback>,
    recent_history_callback: Option<RecentHistoryCallback>,
}
